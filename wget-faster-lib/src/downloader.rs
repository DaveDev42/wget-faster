use crate::{
    output::DownloadedData, parallel, DownloadConfig, Error, HttpClient, Output, ProgressCallback,
    ProgressInfo, Result,
};
use bytes::Bytes;
use futures_util::StreamExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

/// Main downloader for HTTP/HTTPS downloads
///
/// The `Downloader` is the main entry point for performing downloads.
/// It handles parallel downloads, resume functionality, retries, and more.
///
/// # Examples
///
/// ```no_run
/// use wget_faster_lib::{Downloader, DownloadConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let downloader = Downloader::new(DownloadConfig::default())?;
///     let bytes = downloader.download_to_memory("https://example.com/file.txt").await?;
///     println!("Downloaded {} bytes", bytes.len());
///     Ok(())
/// }
/// ```
pub struct Downloader {
    client: HttpClient,
}

impl Downloader {
    /// Create a new downloader with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized (e.g., invalid proxy configuration)
    pub fn new(config: DownloadConfig) -> Result<Self> {
        let client = HttpClient::new(config)?;
        Ok(Self { client })
    }

    /// Get a reference to the HTTP client
    ///
    /// Provides access to the underlying HTTP client for advanced operations
    /// like getting metadata or checking server capabilities.
    pub fn get_client(&self) -> &HttpClient {
        &self.client
    }

    /// Build a request with the configured method, headers, and body
    fn build_request(
        &self,
        url: &str,
        range: Option<&str>,
        if_modified_since: Option<std::time::SystemTime>,
    ) -> Result<reqwest::RequestBuilder> {
        self.build_request_with_auth(url, range, if_modified_since, false)
    }

    /// Build a request with optional auth override
    ///
    /// If `force_preemptive_auth` is true, authentication will be added even if
    /// `auth_no_challenge` is false. This is used when HEAD request succeeded with auth.
    fn build_request_with_auth(
        &self,
        url: &str,
        range: Option<&str>,
        if_modified_since: Option<std::time::SystemTime>,
        force_preemptive_auth: bool,
    ) -> Result<reqwest::RequestBuilder> {
        let config = self.client.config();

        // Check if we've previously authenticated to this host (via HttpClient's authenticated_hosts set)
        // This is automatically managed by the client and implements GNU wget's behavior
        // of remembering successful auth and not waiting for challenge on subsequent requests

        tracing::debug!(
            method = %config.method.as_str(),
            url = %url,
            has_range = range.is_some(),
            has_if_modified_since = if_modified_since.is_some(),
            force_preemptive_auth,
            "Building HTTP request"
        );

        let mut request = match config.method {
            crate::config::HttpMethod::Get => self.client.client().get(url),
            crate::config::HttpMethod::Head => self.client.client().head(url),
            crate::config::HttpMethod::Post => self.client.client().post(url),
            crate::config::HttpMethod::Put => self.client.client().put(url),
            crate::config::HttpMethod::Delete => self.client.client().delete(url),
            crate::config::HttpMethod::Patch => self.client.client().patch(url),
            crate::config::HttpMethod::Options => {
                self.client.client().request(reqwest::Method::OPTIONS, url)
            },
        };

        // Add body data for POST/PUT/PATCH
        if let Some(ref body) = config.body_data {
            request = request.body(body.clone());

            // Add Content-Type if specified
            if let Some(ref content_type) = config.content_type {
                request = request.header(reqwest::header::CONTENT_TYPE, content_type);
            } else if matches!(
                config.method,
                crate::config::HttpMethod::Post
                    | crate::config::HttpMethod::Put
                    | crate::config::HttpMethod::Patch
            ) {
                // Default to application/x-www-form-urlencoded for POST
                request = request
                    .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded");
            }
        }

        // Add Referer header
        if let Some(ref referer) = config.referer {
            request = request.header(reqwest::header::REFERER, referer);
        }

        // Add Range header if provided
        if let Some(range_value) = range {
            tracing::debug!(range = %range_value, "Adding Range header");
            request = request.header(reqwest::header::RANGE, range_value);
        }

        // Add If-Modified-Since header if provided (for timestamping/conditional GET)
        if let Some(time) = if_modified_since {
            let http_date = httpdate::fmt_http_date(time);
            tracing::debug!(if_modified_since = %http_date, "Adding If-Modified-Since header");
            request = request.header(reqwest::header::IF_MODIFIED_SINCE, http_date);
        }

        // Add authentication if configured and either:
        // 1. auth_no_challenge is set (preemptive auth flag), OR
        // 2. force_preemptive_auth is true (from metadata.auth_succeeded), OR
        // 3. We've previously authenticated successfully to this host (via HttpClient's authenticated_hosts set)
        let host_previously_authenticated = url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .map_or(false, |h| self.client.authenticated_hosts_contains(&h));

        if config.auth_no_challenge || force_preemptive_auth || host_previously_authenticated {
            // Get credentials - either from config.auth or from .netrc
            let auth_creds = if let Some(ref auth) = config.auth {
                Some(auth.clone())
            } else if host_previously_authenticated {
                // If we've authenticated before but don't have config.auth, try .netrc
                crate::auth_handler::get_credentials(url, config)
            } else {
                None
            };

            if let Some(auth) = auth_creds {
                tracing::debug!(
                    username = %auth.username,
                    preemptive = force_preemptive_auth,
                    host_authenticated = host_previously_authenticated,
                    "Adding preemptive Basic authentication"
                );
                request = request.basic_auth(&auth.username, Some(&auth.password));
            }
        }

        Ok(request)
    }

    /// Download a URL to memory
    ///
    /// Downloads the entire file into memory and returns it as `Bytes`.
    /// For files larger than 10MB that support Range requests, this will
    /// automatically use parallel downloads for better performance.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download
    ///
    /// # Returns
    ///
    /// The downloaded data as `Bytes`
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails (network error, invalid status, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wget_faster_lib::{Downloader, DownloadConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let downloader = Downloader::new(DownloadConfig::default())?;
    ///     let bytes = downloader.download_to_memory("https://example.com/file.txt").await?;
    ///     println!("Downloaded {} bytes", bytes.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_to_memory(&self, url: &str) -> Result<Bytes> {
        self.download_to_memory_with_progress(url, None).await
    }

    /// Download a URL to memory with progress tracking
    ///
    /// Downloads the entire file into memory with progress callbacks.
    /// The progress callback is called periodically with download statistics.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download
    /// * `progress_callback` - Optional callback function for progress updates
    ///
    /// # Returns
    ///
    /// The downloaded data as `Bytes`
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails (network error, invalid status, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo};
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let downloader = Downloader::new(DownloadConfig::default())?;
    ///
    ///     let progress = Arc::new(|info: ProgressInfo| {
    ///         if let Some(pct) = info.percentage() {
    ///             println!("{:.1}% - {} - ETA: {:?}",
    ///                 pct,
    ///                 info.format_speed(),
    ///                 info.format_eta()
    ///             );
    ///         }
    ///     });
    ///
    ///     let bytes = downloader
    ///         .download_to_memory_with_progress("https://example.com/large.zip", Some(progress))
    ///         .await?;
    ///     println!("Downloaded {} bytes", bytes.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_to_memory_with_progress(
        &self,
        url: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Bytes> {
        tracing::debug!(url = %url, "Starting download to memory");

        // Only send HEAD request if parallel downloads are enabled AND threshold is set
        // This allows us to check file size and Range support
        let should_check_metadata =
            self.client.config().parallel_threshold > 0 && self.client.config().parallel_chunks > 1;

        if !should_check_metadata {
            // Skip HEAD request - go directly to GET
            // This matches GNU wget behavior for simple downloads
            tracing::debug!(
                "Skipping HEAD request - going directly to GET (parallel downloads disabled)"
            );
            return self.download_sequential(url, progress_callback).await;
        }

        // Get metadata (sends HEAD request)
        let metadata = self.client.get_metadata(url).await?;
        tracing::debug!(
            status_code = metadata.status_code,
            content_length = ?metadata.content_length,
            supports_range = metadata.supports_range,
            "Received metadata from HEAD request"
        );

        // Print server response if requested
        if self.client.config().print_server_response {
            eprintln!("{}", metadata.format_headers());
        }

        // Check status code - for HEAD requests, 4xx and 5xx are informational, not fatal
        // We'll let the actual GET request handle the error
        if metadata.status_code >= 400 && metadata.status_code < 600 {
            // For HEAD, we continue to GET which will handle the error properly
            // This allows wget-compatible behavior where some servers respond differently to HEAD vs GET
        }

        // Use parallel download if supported and beneficial
        if metadata.supports_range {
            if let Some(total_size) = metadata.content_length {
                if total_size > self.client.config().parallel_threshold {
                    // Use parallel for files > threshold
                    tracing::info!(
                        total_size,
                        threshold = self.client.config().parallel_threshold,
                        chunks = self.client.config().parallel_chunks,
                        "Using parallel download (file size exceeds threshold)"
                    );
                    return parallel::download_parallel(
                        &self.client,
                        url,
                        total_size,
                        progress_callback,
                    )
                    .await;
                }
                tracing::debug!(
                    total_size,
                    threshold = self.client.config().parallel_threshold,
                    "Using sequential download (file size below threshold)"
                );
            } else {
                tracing::debug!("Using sequential download (no content length)");
            }
        } else {
            tracing::debug!("Using sequential download (server doesn't support Range requests)");
        }

        // Fall back to sequential download
        self.download_sequential(url, progress_callback).await
    }

    /// Download a URL to a file
    ///
    /// Downloads content to the specified file path. Supports resume functionality
    /// if the file already exists and the server supports Range requests.
    /// For large files, automatically uses parallel downloads when beneficial.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download
    /// * `path` - The file path where content will be saved
    ///
    /// # Returns
    ///
    /// A `DownloadResult` containing download metadata and information
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails or file I/O fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wget_faster_lib::{Downloader, DownloadConfig};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let downloader = Downloader::new(DownloadConfig::default())?;
    ///     let result = downloader
    ///         .download_to_file("https://example.com/file.zip", PathBuf::from("file.zip"))
    ///         .await?;
    ///     println!("Downloaded to: {:?}", result.data.path());
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_to_file(&self, url: &str, path: PathBuf) -> Result<DownloadResult> {
        self.download_to_file_with_progress(url, path, None, false)
            .await
    }

    /// Download a URL to a file with progress tracking
    ///
    /// Downloads content to the specified file path with progress callbacks.
    /// Supports resume functionality and parallel downloads.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download
    /// * `path` - The file path where content will be saved
    /// * `progress_callback` - Optional callback function for progress updates
    ///
    /// # Returns
    ///
    /// A `DownloadResult` containing download metadata and information
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails or file I/O fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo};
    /// use std::sync::Arc;
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let downloader = Downloader::new(DownloadConfig::default())?;
    ///
    ///     let progress = Arc::new(|info: ProgressInfo| {
    ///         if let Some(pct) = info.percentage() {
    ///             println!("[{:.1}%] {} at {}", pct, info.format_downloaded(), info.format_speed());
    ///         }
    ///     });
    ///
    ///     let result = downloader
    ///         .download_to_file_with_progress(
    ///             "https://example.com/large.zip",
    ///             PathBuf::from("large.zip"),
    ///             Some(progress)
    ///         )
    ///         .await?;
    ///     println!("Download complete: {:?}", result.data.path());
    ///     Ok(())
    /// }
    /// ```
    pub async fn download_to_file_with_progress(
        &self,
        url: &str,
        path: PathBuf,
        progress_callback: Option<ProgressCallback>,
        is_retry: bool,
    ) -> Result<DownloadResult> {
        // If method is HEAD, send HEAD request and return without downloading
        // This matches GNU wget --method=HEAD behavior: check headers only, no file creation
        if matches!(self.client.config().method, crate::config::HttpMethod::Head) {
            let metadata = self.client.get_metadata(url).await?;
            tracing::info!(url = %url, "HEAD method requested - returning metadata without download");
            return Ok(DownloadResult {
                data: DownloadedData::new_memory(Bytes::new()),
                url: url.to_string(),
                metadata,
            });
        }

        // Skip HEAD request if:
        // 1. Timestamping mode (-N) - use GET with If-Modified-Since instead
        // 2. Simple download without parallel (no need to check Range support)
        // 3. GNU wget compatibility mode (always skip HEAD for wget-compatible behavior)
        // 4. Retry attempt - HEAD was already sent in first attempt, don't repeat
        // 5. Low retry count (< 5) - user wants fast failure, don't waste time on HEAD
        //    This matches GNU wget behavior with --tries=N where N is small
        let skip_head = self.client.config().timestamping
            || self.client.config().gnu_wget_compat
            || (self.client.config().parallel_threshold == 0
                || self.client.config().parallel_chunks <= 1)
            || is_retry
            || self.client.config().retry.max_retries < 5;

        // Get metadata first (unless skipping HEAD)
        // If timestamping is enabled, use GET with If-Modified-Since header instead of HEAD
        let (metadata, if_modified_since) = if skip_head {
            // Timestamping mode: skip HEAD, use GET with If-Modified-Since directly
            // Create dummy metadata for now - actual metadata will come from GET request
            let dummy_metadata = crate::client::ResourceMetadata {
                content_length: None,
                content_type: None,
                supports_range: false,
                status_code: 200, // Assume success, will be validated in GET
                last_modified: None,
                etag: None,
                content_disposition: None,
                headers: reqwest::header::HeaderMap::new(),
                auth_succeeded: false,
            };

            let if_modified_since_time = if path.exists() {
                let local_metadata = tokio::fs::metadata(&path).await?;
                Some(local_metadata.modified()?)
            } else {
                None
            };

            (dummy_metadata, if_modified_since_time)
        } else {
            // Normal mode: use HEAD request to get metadata
            (self.client.get_metadata(url).await?, None)
        };

        // Print server response if requested (skip in timestamping mode since we haven't made request yet)
        if !skip_head && self.client.config().print_server_response {
            eprintln!("{}", metadata.format_headers());
        }

        // Handle special status codes from HEAD request (skip in timestamping mode)
        if !skip_head {
            use crate::response_handler::ResponseStatus;
            let response_status = ResponseStatus::from_status_code(metadata.status_code);

            match response_status {
                ResponseStatus::NoContent => {
                    tracing::info!("HTTP 204 No Content - skipping file creation");
                    return Ok(DownloadResult {
                        data: DownloadedData::new_memory(Bytes::new()),
                        url: url.to_string(),
                        metadata,
                    });
                },
                ResponseStatus::NotModified => {
                    tracing::info!(path = %path.display(), "HTTP 304 Not Modified - file is up to date");
                    // If file exists, return it as-is
                    if path.exists() {
                        let local_metadata = tokio::fs::metadata(&path).await?;
                        let local_size = local_metadata.len();
                        return Ok(DownloadResult {
                            data: DownloadedData::new_file(path.clone(), local_size, false),
                            url: url.to_string(),
                            metadata,
                        });
                    }
                    // If file doesn't exist, treat as success with empty result
                    tracing::warn!("HTTP 304 but file doesn't exist - returning empty result");
                    return Ok(DownloadResult {
                        data: DownloadedData::new_memory(Bytes::new()),
                        url: url.to_string(),
                        metadata,
                    });
                },
                ResponseStatus::RangeNotSatisfiable => {
                    tracing::info!(path = %path.display(), "HTTP 416 Range Not Satisfiable - file already complete");
                    // If file exists, return it as-is (already complete)
                    if path.exists() {
                        let local_metadata = tokio::fs::metadata(&path).await?;
                        let local_size = local_metadata.len();
                        return Ok(DownloadResult {
                            data: DownloadedData::new_file(path.clone(), local_size, false),
                            url: url.to_string(),
                            metadata,
                        });
                    }
                    // If file doesn't exist, this is an error
                    tracing::error!("HTTP 416 but file doesn't exist - this is an error");
                    return Err(Error::InvalidStatus(416));
                },
                ResponseStatus::ClientError => {
                    // 4xx errors from HEAD: check content_on_error setting
                    // If false, return error immediately (don't create file)
                    // Otherwise continue to GET which will handle them properly
                    if !self.client.config().content_on_error {
                        return Err(Error::InvalidStatus(metadata.status_code));
                    }
                    // Continue to GET request to download error page
                },
                ResponseStatus::ServerError => {
                    // 5xx errors from HEAD: always continue to GET
                    // This allows GET request retry logic to handle server errors
                    // Matches GNU wget behavior where --tries applies to actual download attempts (GET), not metadata checks (HEAD)
                    tracing::debug!(
                        status_code = metadata.status_code,
                        "HEAD returned 5xx - will retry with GET requests"
                    );
                    // Continue to GET request which will handle retries
                },
                ResponseStatus::AuthChallenge => {
                    // Auth challenges should have been handled in get_metadata
                    // If we're here, auth failed
                    return Err(Error::InvalidStatus(metadata.status_code));
                },
                _ => {
                    // Success or other - continue normally
                },
            }
        }

        // Check timestamping - skip if local file is newer or delete if we need to re-download
        // In skip_head mode, this check will be done after GET request in download_sequential_to_writer
        let mut should_delete_existing = false;
        if !skip_head && self.client.config().timestamping {
            tracing::debug!(path = %path.display(), "Timestamping enabled - checking local vs remote timestamps");

            let (action, result_data) =
                crate::timestamping::check_timestamp(&path, &metadata).await?;

            use crate::timestamping::TimestampAction;
            match action {
                TimestampAction::Skip => {
                    // Local file is up to date, return it
                    // Safe: check_timestamp always returns Some(DownloadedData) when action is Skip
                    return Ok(DownloadResult {
                        data: result_data
                            .expect("check_timestamp should return data when action is Skip"),
                        url: url.to_string(),
                        metadata,
                    });
                },
                TimestampAction::DeleteAndDownload => {
                    // Need to delete and re-download
                    should_delete_existing = true;
                },
                TimestampAction::Download => {
                    // Just download (file doesn't exist)
                },
            }
        }

        // Delete existing file if timestamping determined we need to re-download
        if should_delete_existing && path.exists() {
            tracing::info!(path = %path.display(), "Deleting existing file for re-download");
            tokio::fs::remove_file(&path).await?;
        }

        // Check if file exists for resume
        // If --start-pos is specified, it overrides automatic resume from file size
        // IMPORTANT: When timestamping (-N) is enabled, don't resume - do conditional GET instead
        let resume_from = if self.client.config().timestamping {
            // Timestamping mode: always start from 0 and use If-Modified-Since header
            tracing::debug!("Timestamping enabled - skipping resume, will use conditional GET");
            0
        } else if let Some(start_pos) = self.client.config().start_pos {
            tracing::debug!(start_pos, "Using --start-pos for resume");
            start_pos
        } else if path.exists() {
            let size = tokio::fs::metadata(&path).await?.len();
            if size > 0 {
                tracing::info!(path = %path.display(), existing_size = size, "Resuming download from existing file");
            }
            size
        } else {
            0
        };

        // In timestamping mode with existing file, download to temp file first
        // Then compare timestamps and decide whether to replace original
        let (mut file, temp_path) = if self.client.config().timestamping && path.exists() {
            // Create temporary file path
            let temp_path = PathBuf::from(format!("{}.wgetf-tmp", path.display()));
            tracing::debug!(
                original = %path.display(),
                temp = %temp_path.display(),
                "Timestamping mode: downloading to temporary file"
            );
            let file = File::create(&temp_path).await?;
            (file, Some(temp_path))
        } else if resume_from > 0 && self.client.config().start_pos.is_none() {
            // Resume mode: append to existing file
            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&path)
                .await?;
            (file, None)
        } else {
            // Normal mode or --start-pos mode or timestamping without existing file: create new file
            (File::create(&path).await?, None)
        };

        // Track which file to potentially clean up on error
        let created_file_path = if temp_path.is_some() {
            temp_path.clone()
        } else if resume_from == 0 {
            // Only clean up if we created a new file (not resuming)
            Some(path.clone())
        } else {
            None
        };

        // Use parallel download if supported and beneficial
        // For sequential downloads, we also capture the actual metadata from the GET response
        let download_result = if metadata.supports_range && resume_from == 0 {
            if let Some(total_size) = metadata.content_length {
                if total_size > self.client.config().parallel_threshold {
                    // Use parallel for files > threshold
                    parallel::download_parallel_to_writer(
                        &self.client,
                        url,
                        total_size,
                        &mut file,
                        progress_callback,
                    )
                    .await
                    .map(|_| (total_size, metadata.clone()))
                } else {
                    self.download_sequential_to_writer(
                        url,
                        &mut file,
                        progress_callback,
                        resume_from,
                        if_modified_since,
                        metadata.auth_succeeded,
                    )
                    .await
                }
            } else {
                self.download_sequential_to_writer(
                    url,
                    &mut file,
                    progress_callback,
                    resume_from,
                    if_modified_since,
                    metadata.auth_succeeded,
                )
                .await
            }
        } else {
            self.download_sequential_to_writer(
                url,
                &mut file,
                progress_callback,
                resume_from,
                if_modified_since,
                metadata.auth_succeeded,
            )
            .await
        };

        // If download failed, clean up the empty file
        let (total_bytes, actual_metadata) = match download_result {
            Ok(result) => result,
            Err(e) => {
                // Drop file handle before deleting
                drop(file);

                // Clean up empty file if download failed
                if let Some(ref cleanup_path) = created_file_path {
                    tracing::debug!(path = %cleanup_path.display(), "Download failed - cleaning up empty file");
                    if let Err(remove_err) = tokio::fs::remove_file(cleanup_path).await {
                        tracing::warn!(
                            path = %cleanup_path.display(),
                            error = %remove_err,
                            "Failed to remove file after download error"
                        );
                    }
                }

                return Err(e);
            },
        };

        // Handle timestamping mode: decide whether to keep new file or original
        // Use Option to safely handle file ownership
        let mut file_option = Some(file);
        if let Some(ref tmp_path) = temp_path {
            // We downloaded to a temporary file - compare timestamps
            // Drop file handle before comparing/moving
            drop(file_option.take().expect("file should be present"));

            // Special case: 304 Not Modified (total_bytes == 0)
            // Delete temp file and keep original
            if total_bytes == 0 {
                tracing::info!("HTTP 304 Not Modified - keeping original file, deleting temp");
                tokio::fs::remove_file(tmp_path).await?;
            } else {
                // We got 200 OK with content - compare timestamps
                // Get original file timestamp
                let original_metadata = tokio::fs::metadata(&path).await?;
                let original_time = original_metadata.modified()?;

                // Compare with remote timestamp
                let should_replace =
                    if let Some(ref remote_modified) = actual_metadata.last_modified {
                        if let Ok(remote_time) = httpdate::parse_http_date(remote_modified) {
                            tracing::debug!(
                                original_time = ?original_time,
                                remote_time = ?remote_time,
                                "Comparing timestamps (post-download)"
                            );

                            match original_time.cmp(&remote_time) {
                                std::cmp::Ordering::Less => {
                                    // Remote is newer - replace
                                    tracing::info!("Remote file is newer - replacing original");
                                    true
                                },
                                std::cmp::Ordering::Greater => {
                                    // Local is newer - keep original
                                    tracing::info!("Local file is newer - keeping original");
                                    false
                                },
                                std::cmp::Ordering::Equal => {
                                    // Same timestamp - check file sizes
                                    // If sizes differ, download (matches GNU wget behavior)
                                    let original_size = original_metadata.len();
                                    if total_bytes != original_size {
                                        tracing::info!(
                                            original_size,
                                            new_size = total_bytes,
                                            "Same timestamp but different size - replacing file"
                                        );
                                        true
                                    } else {
                                        tracing::info!(
                                            "Same timestamp and size - keeping original file"
                                        );
                                        false
                                    }
                                },
                            }
                        } else {
                            // Can't parse remote timestamp - replace anyway
                            tracing::warn!("Failed to parse remote Last-Modified - replacing file");
                            true
                        }
                    } else {
                        // No remote timestamp - replace anyway
                        tracing::info!(
                            last_modified = ?actual_metadata.last_modified,
                            "No remote Last-Modified header - replacing file anyway"
                        );
                        true
                    };

                if should_replace {
                    // Replace original with temp file
                    tracing::debug!(from = %tmp_path.display(), to = %path.display(), "Replacing original file with new version");
                    tokio::fs::rename(tmp_path, &path).await?;
                } else {
                    // Keep original, delete temp file
                    tracing::debug!(temp = %tmp_path.display(), "Deleting temporary file, keeping original");
                    tokio::fs::remove_file(tmp_path).await?;
                }
            }
        }

        // Check if we should create/keep the file
        // Remove empty files for 204 No Content or 0 bytes without resume
        // Skip this check if we used temp_path (timestamping mode) - file handling is done above
        if temp_path.is_none()
            && !skip_head
            && !crate::response_handler::should_create_file(
                metadata.status_code,
                total_bytes,
                resume_from,
            )
        {
            tracing::info!(path = %path.display(), "Removing empty file (should not create)");
            // Drop the file handle before deleting (if not already dropped)
            if let Some(f) = file_option.take() {
                drop(f);
            }

            // Remove the empty file
            if let Err(e) = tokio::fs::remove_file(&path).await {
                // Log error but don't fail if file doesn't exist
                tracing::warn!(path = %path.display(), error = %e, "Failed to remove empty file");
                if self.client.config().verbose {
                    eprintln!("Warning: Failed to remove empty file: {e}");
                }
            }

            // Return empty result without a file
            return Ok(DownloadResult {
                data: DownloadedData::new_memory(Bytes::new()),
                url: url.to_string(),
                metadata,
            });
        }

        // Set file modification time from server if configured and available
        // Use actual_metadata from GET response (which has the real Last-Modified header)
        // instead of dummy metadata from HEAD request
        // IMPORTANT: Always set timestamp in timestamping mode (-N), as it's required for proper operation
        if self.client.config().use_server_timestamps || self.client.config().timestamping {
            crate::timestamping::set_file_timestamp(
                &path,
                &actual_metadata,
                self.client.config().verbose,
            )?;
        }

        // In timestamping mode, if we got 0 bytes (304 Not Modified), use the existing file size
        let final_size = if skip_head && total_bytes == 0 && path.exists() {
            tokio::fs::metadata(&path).await?.len()
        } else {
            total_bytes
        };

        Ok(DownloadResult {
            data: DownloadedData::new_file(path, final_size, resume_from > 0),
            url: url.to_string(),
            metadata,
        })
    }

    /// Download with custom output destination
    ///
    /// Generic download method that supports multiple output types (memory, file, or custom writer).
    /// This provides the most flexibility for different download scenarios.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download
    /// * `output` - The output destination (Memory, File, or `AsyncWrite`)
    /// * `progress_callback` - Optional callback function for progress updates
    ///
    /// # Returns
    ///
    /// A `DownloadResult` containing download metadata and information
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails or output I/O fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wget_faster_lib::{Downloader, DownloadConfig, Output};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let downloader = Downloader::new(DownloadConfig::default())?;
    ///
    ///     // Download to memory
    ///     let result = downloader.download(
    ///         "https://example.com/file.txt",
    ///         Output::Memory,
    ///         None
    ///     ).await?;
    ///
    ///     // Download to file
    ///     let result = downloader.download(
    ///         "https://example.com/file.zip",
    ///         Output::File(PathBuf::from("file.zip")),
    ///         None
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn download(
        &self,
        url: &str,
        output: Output,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<DownloadResult> {
        match output {
            Output::Memory => {
                let bytes = self
                    .download_to_memory_with_progress(url, progress_callback)
                    .await?;

                let metadata = self.client.get_metadata(url).await?;

                Ok(DownloadResult {
                    data: DownloadedData::new_memory(bytes),
                    url: url.to_string(),
                    metadata,
                })
            },

            Output::File(path) => {
                self.download_to_file_with_progress(url, path, progress_callback, false)
                    .await
            },
        }
    }

    /// Sequential download (fallback for servers that don't support Range)
    async fn download_sequential(
        &self,
        url: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Bytes> {
        tracing::debug!(url = %url, "Starting sequential download");
        let request = self.build_request(url, None, None)?;
        let response = request.send().await?;

        let status_code = response.status().as_u16();
        tracing::debug!(status_code, "Received response from GET request");

        // Handle authentication challenges (401/407)
        // If we have credentials but didn't send them preemptively, retry with auth
        if crate::auth_handler::should_retry_auth(status_code, self.client.config()) {
            tracing::info!(
                status_code,
                "Authentication challenge received - retrying with credentials"
            );

            // Get credentials (configured auth or .netrc)
            if let Some(auth) = crate::auth_handler::get_credentials(url, self.client.config()) {
                tracing::debug!(username = %auth.username, "Retrying with authentication");
                // Retry with authentication
                let retry_request = self
                    .client
                    .client()
                    .get(url)
                    .basic_auth(&auth.username, Some(&auth.password));

                let retry_response = retry_request.send().await?;
                let retry_status = retry_response.status().as_u16();
                tracing::debug!(retry_status, "Received retry response with auth");

                // If still unauthorized, return error
                if crate::auth_handler::is_auth_challenge(retry_status) {
                    tracing::error!(retry_status, "Authentication failed even with credentials");
                    return Err(Error::InvalidStatus(retry_status));
                }

                // Success! Continue with retry_response
                tracing::info!("Authentication successful");

                // Remember this host for future preemptive auth (matches HEAD request behavior)
                if let Some(host) = url::Url::parse(url)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.to_string()))
                {
                    self.client.mark_host_authenticated(host.clone());
                    tracing::debug!(host = ?host, "GET request authentication successful - will use preemptive auth for subsequent requests");
                }

                return self
                    .process_sequential_response(retry_response, url, progress_callback)
                    .await;
            }
            // No credentials available
            tracing::warn!("No credentials available for authentication");
            return Err(Error::InvalidStatus(status_code));
        }

        // Check if we should proceed based on status code
        match crate::response_handler::should_proceed_download(status_code, self.client.config()) {
            Ok(true) => {
                // Proceed with download
                self.process_sequential_response(response, url, progress_callback)
                    .await
            },
            Ok(false) => {
                // Skip download (304/416 - should not reach here in sequential download)
                Ok(Bytes::new())
            },
            Err(err_status) => {
                // Return error
                Err(Error::InvalidStatus(err_status))
            },
        }
    }

    /// Helper to process response body for sequential downloads
    async fn process_sequential_response(
        &self,
        response: reqwest::Response,
        url: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Bytes> {
        let status_code = response.status().as_u16();

        // Check if we should proceed based on status code
        match crate::response_handler::should_proceed_download(status_code, self.client.config()) {
            Ok(false) => {
                // Skip download (empty response)
                return Ok(Bytes::new());
            },
            Err(err_status) => {
                // Return error
                return Err(Error::InvalidStatus(err_status));
            },
            Ok(true) => {
                // Proceed with download
            },
        }

        let total_size = response.content_length();
        let mut downloaded = 0u64;
        let start_time = Instant::now();
        let mut last_chunk_time = Instant::now();

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;

            // Apply speed limiting if configured
            if let Some(speed_limit) = self.client.config().speed_limit {
                let chunk_size = chunk.len() as u64;
                let expected_duration =
                    Duration::from_secs_f64(chunk_size as f64 / speed_limit as f64);
                let actual_duration = last_chunk_time.elapsed();

                if actual_duration < expected_duration {
                    sleep(expected_duration - actual_duration).await;
                }
                last_chunk_time = Instant::now();
            }

            if let Some(callback) = &progress_callback {
                let mut progress = ProgressInfo::new(url.to_string());
                progress.total_size = total_size;
                progress.update(chunk.len() as u64, start_time);
                progress.downloaded = downloaded;
                callback(progress);
            }
        }

        Ok(Bytes::from(buffer))
    }

    /// Sequential download to writer
    /// Returns (bytes_downloaded, actual_metadata_from_response)
    async fn download_sequential_to_writer<W>(
        &self,
        url: &str,
        writer: &mut W,
        progress_callback: Option<ProgressCallback>,
        resume_from: u64,
        if_modified_since: Option<std::time::SystemTime>,
        force_preemptive_auth: bool,
    ) -> Result<(u64, crate::client::ResourceMetadata)>
    where
        W: AsyncWriteExt + Unpin + Send,
    {
        let range_header = if resume_from > 0 {
            Some(format!("bytes={resume_from}-"))
        } else {
            None
        };

        let request = self.build_request_with_auth(
            url,
            range_header.as_deref(),
            if_modified_since,
            force_preemptive_auth,
        )?;
        let response = request.send().await?;

        let status_code = response.status().as_u16();

        // Handle authentication challenges (401/407)
        // If we have credentials but didn't send them preemptively, retry with auth
        if crate::auth_handler::should_retry_auth(status_code, self.client.config()) {
            // Get credentials (configured auth or .netrc)
            if let Some(auth) = crate::auth_handler::get_credentials(url, self.client.config()) {
                // Retry with authentication (preserving range header if needed)
                let mut retry_request = self
                    .client
                    .client()
                    .get(url)
                    .basic_auth(&auth.username, Some(&auth.password));

                if let Some(ref range) = range_header {
                    retry_request = retry_request.header(reqwest::header::RANGE, range);
                }

                let retry_response = retry_request.send().await?;
                let retry_status = retry_response.status().as_u16();

                // Extract metadata from retry response before processing
                let retry_metadata =
                    crate::client::HttpClient::extract_metadata_from_response(&retry_response);

                // If still unauthorized, return error
                if crate::auth_handler::is_auth_challenge(retry_status) {
                    return Err(Error::InvalidStatus(retry_status));
                }

                // Success! Continue with retry_response

                // Remember this host for future preemptive auth (matches HEAD request behavior)
                if let Some(host) = url::Url::parse(url)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.to_string()))
                {
                    self.client.mark_host_authenticated(host.clone());
                    tracing::debug!(host = ?host, "GET request authentication successful - will use preemptive auth for subsequent requests");
                }

                let bytes = self
                    .process_writer_response(
                        retry_response,
                        url,
                        writer,
                        progress_callback,
                        resume_from,
                    )
                    .await?;

                return Ok((bytes, retry_metadata));
            }
            // No credentials available
            return Err(Error::InvalidStatus(status_code));
        }

        // Extract metadata from response before consuming it
        let metadata = crate::client::HttpClient::extract_metadata_from_response(&response);

        // Handle special status codes
        use crate::response_handler::ResponseStatus;
        let response_status = ResponseStatus::from_status_code(status_code);

        match response_status {
            ResponseStatus::NoContent => {
                // 204 No Content - don't create file
                return Ok((0, metadata));
            },
            ResponseStatus::NotModified => {
                // 304 Not Modified - file is already up to date
                // In timestamping mode, the file should already exist - return its size
                tracing::info!("HTTP 304 Not Modified on GET - file is up to date");
                // Close the writer without writing anything
                writer.flush().await?;
                // Return 0 to indicate no new bytes were downloaded
                // The caller will handle keeping the existing file
                return Ok((0, metadata));
            },
            ResponseStatus::RangeNotSatisfiable => {
                // 416 Range Not Satisfiable - file is already complete
                return Ok((resume_from, metadata));
            },
            ResponseStatus::Success => {
                // 200 OK or 206 Partial Content - proceed
            },
            ResponseStatus::ClientError | ResponseStatus::ServerError => {
                // Check content_on_error
                if !self.client.config().content_on_error {
                    return Err(Error::InvalidStatus(status_code));
                }
                // Proceed to download error page
            },
            _ => {
                // Other non-success status codes
                return Err(Error::InvalidStatus(status_code));
            },
        }

        self.process_writer_response(response, url, writer, progress_callback, resume_from)
            .await
            .map(|bytes| (bytes, metadata))
    }

    /// Helper to process response body for sequential downloads to writer
    async fn process_writer_response<W>(
        &self,
        response: reqwest::Response,
        url: &str,
        writer: &mut W,
        progress_callback: Option<ProgressCallback>,
        resume_from: u64,
    ) -> Result<u64>
    where
        W: AsyncWriteExt + Unpin + Send,
    {
        let status_code = response.status().as_u16();

        // Handle special status codes
        use crate::response_handler::ResponseStatus;
        let response_status = ResponseStatus::from_status_code(status_code);

        match response_status {
            ResponseStatus::NoContent => {
                // 204 No Content - don't create file
                return Ok(0);
            },
            ResponseStatus::NotModified => {
                // 304 Not Modified - file is already up to date
                tracing::info!("HTTP 304 Not Modified - file is up to date");
                return Ok(resume_from);
            },
            ResponseStatus::RangeNotSatisfiable => {
                // 416 Range Not Satisfiable - file is already complete
                return Ok(resume_from);
            },
            ResponseStatus::Success => {
                // 200 OK or 206 Partial Content - proceed
            },
            ResponseStatus::ClientError | ResponseStatus::ServerError => {
                // Check content_on_error
                if !self.client.config().content_on_error {
                    return Err(Error::InvalidStatus(status_code));
                }
                // Proceed to download error page
            },
            _ => {
                // Other non-success status codes
                return Err(Error::InvalidStatus(status_code));
            },
        }

        let total_size = response.content_length().map(|s| s + resume_from);
        let mut downloaded = resume_from;
        let start_time = Instant::now();
        let mut last_chunk_time = Instant::now();

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            writer.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Apply speed limiting if configured
            if let Some(speed_limit) = self.client.config().speed_limit {
                let chunk_size = chunk.len() as u64;
                let expected_duration =
                    Duration::from_secs_f64(chunk_size as f64 / speed_limit as f64);
                let actual_duration = last_chunk_time.elapsed();

                if actual_duration < expected_duration {
                    sleep(expected_duration - actual_duration).await;
                }
                last_chunk_time = Instant::now();
            }

            if let Some(callback) = &progress_callback {
                let mut progress = ProgressInfo::new(url.to_string());
                progress.total_size = total_size;
                progress.update(chunk.len() as u64, start_time);
                progress.downloaded = downloaded;
                callback(progress);
            }
        }

        writer.flush().await?;

        Ok(downloaded)
    }
}

/// Result of a download operation
///
/// Contains all information about a completed download, including the downloaded data,
/// the original URL, and server metadata.
///
/// # Examples
///
/// ```no_run
/// use wget_faster_lib::{Downloader, DownloadConfig};
/// use std::path::PathBuf;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let downloader = Downloader::new(DownloadConfig::default())?;
///     let result = downloader
///         .download_to_file("https://example.com/file.zip", PathBuf::from("file.zip"))
///         .await?;
///
///     println!("Downloaded from: {}", result.url);
///     println!("Content type: {:?}", result.metadata.content_type);
///     println!("File size: {:?}", result.metadata.content_length);
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct DownloadResult {
    /// Downloaded data (in memory or path to file)
    pub data: DownloadedData,

    /// URL that was downloaded
    pub url: String,

    /// Resource metadata from server (content type, length, etc.)
    pub metadata: crate::client::ResourceMetadata,
}

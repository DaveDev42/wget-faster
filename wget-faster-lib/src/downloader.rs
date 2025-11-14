use crate::{
    Error, Result, DownloadConfig, HttpClient, Output, ProgressCallback,
    ProgressInfo, output::DownloadedData, parallel,
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
    fn build_request(&self, url: &str, range: Option<&str>) -> Result<reqwest::RequestBuilder> {
        let config = self.client.config();

        let mut request = match config.method {
            crate::config::HttpMethod::Get => self.client.client().get(url),
            crate::config::HttpMethod::Head => self.client.client().head(url),
            crate::config::HttpMethod::Post => self.client.client().post(url),
            crate::config::HttpMethod::Put => self.client.client().put(url),
            crate::config::HttpMethod::Delete => self.client.client().delete(url),
            crate::config::HttpMethod::Patch => self.client.client().patch(url),
            crate::config::HttpMethod::Options => {
                self.client.client().request(reqwest::Method::OPTIONS, url)
            }
        };

        // Add body data for POST/PUT/PATCH
        if let Some(ref body) = config.body_data {
            request = request.body(body.clone());

            // Add Content-Type if specified
            if let Some(ref content_type) = config.content_type {
                request = request.header(reqwest::header::CONTENT_TYPE, content_type);
            } else if matches!(config.method, crate::config::HttpMethod::Post | crate::config::HttpMethod::Put | crate::config::HttpMethod::Patch) {
                // Default to application/x-www-form-urlencoded for POST
                request = request.header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded");
            }
        }

        // Add Referer header
        if let Some(ref referer) = config.referer {
            request = request.header(reqwest::header::REFERER, referer);
        }

        // Add Range header if provided
        if let Some(range_value) = range {
            request = request.header(reqwest::header::RANGE, range_value);
        }

        // Add authentication if configured and auth_no_challenge is set
        if config.auth_no_challenge {
            if let Some(ref auth) = config.auth {
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
        // Get metadata
        let metadata = self.client.get_metadata(url).await?;

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
                    return parallel::download_parallel(
                        &self.client,
                        url,
                        total_size,
                        progress_callback,
                    )
                    .await;
                }
            }
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
        self.download_to_file_with_progress(url, path, None).await
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
    ) -> Result<DownloadResult> {
        // Get metadata first
        let metadata = self.client.get_metadata(url).await?;

        // Print server response if requested
        if self.client.config().print_server_response {
            eprintln!("{}", metadata.format_headers());
        }

        // Handle special status codes from HEAD request
        match metadata.status_code {
            // 204 No Content - success but no body, don't create file
            204 => {
                return Ok(DownloadResult {
                    data: DownloadedData::new_memory(Bytes::new()),
                    url: url.to_string(),
                    metadata,
                });
            }
            // 304 Not Modified - file is already up to date
            304 => {
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
                return Ok(DownloadResult {
                    data: DownloadedData::new_memory(Bytes::new()),
                    url: url.to_string(),
                    metadata,
                });
            }
            // 416 Range Not Satisfiable - file is already complete
            416 => {
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
                return Err(Error::InvalidStatus(416));
            }
            // For other status codes 4xx and 5xx, check content_on_error setting
            // If content_on_error is false, return error immediately (don't create file)
            // Otherwise continue to GET which will handle them properly
            // This allows wget-compatible behavior where some servers respond differently to HEAD vs GET
            // Note: 504 Gateway Timeout will be retried by the retry mechanism (TODO: implement retry loop)
            _ if metadata.status_code >= 400 && metadata.status_code < 600 => {
                if !self.client.config().content_on_error {
                    // Don't create file for error responses when content_on_error is false
                    return Err(Error::InvalidStatus(metadata.status_code));
                }
                // Otherwise, continue to GET request to download error page
            }
            _ => {}
        }

        // Check timestamping - skip if local file is newer or delete if we need to re-download
        let mut should_delete_existing = false;
        if self.client.config().timestamping && path.exists() {
            let local_metadata = tokio::fs::metadata(&path).await?;
            let local_size = local_metadata.len();
            let local_time = local_metadata.modified()?;

            if let Some(ref remote_modified) = metadata.last_modified {
                // Parse remote Last-Modified header (RFC 2822 or RFC 3339 format)
                if let Ok(remote_time) = httpdate::parse_http_date(remote_modified) {
                    // Check if local file is older than remote (need to download)
                    if local_time < remote_time {
                        // Local file is older, delete and re-download
                        should_delete_existing = true;
                    } else if local_time > remote_time {
                        // Local file is newer, skip download
                        return Ok(DownloadResult {
                            data: DownloadedData::new_file(path.clone(), local_size, false),
                            url: url.to_string(),
                            metadata,
                        });
                    } else {
                        // Same timestamp - check file size
                        if let Some(remote_size) = metadata.content_length {
                            if local_size == remote_size {
                                // Same timestamp and size, skip download
                                return Ok(DownloadResult {
                                    data: DownloadedData::new_file(path.clone(), local_size, false),
                                    url: url.to_string(),
                                    metadata,
                                });
                            }
                            // Same timestamp but different size - delete and re-download
                            should_delete_existing = true;
                        } else {
                            // No remote size info, skip download (same timestamp)
                            return Ok(DownloadResult {
                                data: DownloadedData::new_file(path.clone(), local_size, false),
                                url: url.to_string(),
                                metadata,
                            });
                        }
                    }
                }
            } else {
                // No Last-Modified header from server
                // wget behavior: download the file (server doesn't provide timestamp info)
                should_delete_existing = true;
            }
        }

        // Delete existing file if timestamping determined we need to re-download
        if should_delete_existing && path.exists() {
            tokio::fs::remove_file(&path).await?;
        }

        // Check if file exists for resume
        // If --start-pos is specified, it overrides automatic resume from file size
        let resume_from = if let Some(start_pos) = self.client.config().start_pos {
            start_pos
        } else if path.exists() {
            tokio::fs::metadata(&path).await?.len()
        } else {
            0
        };

        // Create or open file
        // When --start-pos is used, always create a new file (even if resume_from > 0)
        // because the file numbering logic will have created a new numbered file
        let mut file = if resume_from > 0 && self.client.config().start_pos.is_none() {
            // Resume mode: append to existing file
            tokio::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&path)
                .await?
        } else {
            // Normal mode or --start-pos mode: create new file
            File::create(&path).await?
        };

        // Use parallel download if supported and beneficial
        let total_bytes = if metadata.supports_range && resume_from == 0 {
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
                    .await?;
                    total_size
                } else {
                    self.download_sequential_to_writer(url, &mut file, progress_callback, resume_from)
                        .await?
                }
            } else {
                self.download_sequential_to_writer(url, &mut file, progress_callback, resume_from)
                    .await?
            }
        } else {
            self.download_sequential_to_writer(url, &mut file, progress_callback, resume_from)
                .await?
        };

        // If 204 No Content or no bytes downloaded, remove the empty file
        // This matches wget behavior: don't create files for 204 responses
        if total_bytes == 0 && resume_from == 0 {
            // Drop the file handle before deleting
            drop(file);

            // Remove the empty file
            if let Err(e) = tokio::fs::remove_file(&path).await {
                // Log error but don't fail if file doesn't exist
                if self.client.config().verbose {
                    eprintln!("Warning: Failed to remove empty file: {}", e);
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
        if self.client.config().use_server_timestamps {
            if let Some(ref last_modified_str) = metadata.last_modified {
                if let Ok(remote_time) = httpdate::parse_http_date(last_modified_str) {
                    // Convert SystemTime to FileTime for setting the modification time
                    let file_time = filetime::FileTime::from_system_time(remote_time);
                    // Set the file modification time (atime is set to current time, mtime to server time)
                    if let Err(e) = filetime::set_file_mtime(&path, file_time) {
                        // Log error but don't fail the download
                        if self.client.config().verbose {
                            eprintln!("Warning: Failed to set file modification time: {}", e);
                        }
                    }
                }
            }
        }

        Ok(DownloadResult {
            data: DownloadedData::new_file(path, total_bytes, resume_from > 0),
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
    /// * `output` - The output destination (Memory, File, or AsyncWrite)
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
            }

            Output::File(path) => {
                self.download_to_file_with_progress(url, path, progress_callback)
                    .await
            }
        }
    }

    /// Sequential download (fallback for servers that don't support Range)
    async fn download_sequential(
        &self,
        url: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Bytes> {
        let request = self.build_request(url, None)?;
        let response = request.send().await?;

        let status_code = response.status().as_u16();

        // Handle authentication challenges (401/407)
        // If we have credentials but didn't send them preemptively, retry with auth
        if (status_code == 401 || status_code == 407) && !self.client.config().auth_no_challenge {
            // Try configured auth first, then .netrc
            let auth = if let Some(ref auth) = self.client.config().auth {
                Some(auth.clone())
            } else {
                // Try .netrc file
                match crate::netrc::Netrc::from_default_location() {
                    Ok(Some(netrc)) => {
                        // Extract hostname from URL
                        if let Ok(parsed) = url::Url::parse(url) {
                            if let Some(host) = parsed.host_str() {
                                netrc.get(host)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            };

            if let Some(auth) = auth {
                // Retry with authentication
                let retry_request = self.client.client().get(url)
                    .basic_auth(&auth.username, Some(&auth.password));

                let retry_response = retry_request.send().await?;
                let retry_status = retry_response.status().as_u16();

                // If still unauthorized, return error
                if retry_status == 401 || retry_status == 407 {
                    return Err(Error::InvalidStatus(retry_status));
                }

                // Success! Continue with retry_response
                return self.process_sequential_response(retry_response, url, progress_callback).await;
            } else {
                // No credentials available
                return Err(Error::InvalidStatus(status_code));
            }
        }

        // 204 No Content is a success but has no body
        if status_code == 204 {
            return Ok(Bytes::new());
        }

        // Check for HTTP errors (4xx/5xx)
        if status_code >= 400 {
            // Only save error content if content_on_error is enabled
            if !self.client.config().content_on_error {
                return Err(Error::InvalidStatus(status_code));
            }
            // If content_on_error is enabled, continue downloading the error page
        } else if !response.status().is_success() {
            // Other non-success status codes (e.g., 1xx, 3xx unexpected)
            return Err(Error::InvalidStatus(status_code));
        }

        self.process_sequential_response(response, url, progress_callback).await
    }

    /// Helper to process response body for sequential downloads
    async fn process_sequential_response(
        &self,
        response: reqwest::Response,
        url: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Bytes> {
        let status_code = response.status().as_u16();

        // 204 No Content is a success but has no body
        if status_code == 204 {
            return Ok(Bytes::new());
        }

        // Check for HTTP errors (4xx/5xx)
        if status_code >= 400 {
            // Only save error content if content_on_error is enabled
            if !self.client.config().content_on_error {
                return Err(Error::InvalidStatus(status_code));
            }
            // If content_on_error is enabled, continue downloading the error page
        } else if !response.status().is_success() {
            // Other non-success status codes (e.g., 1xx, 3xx unexpected)
            return Err(Error::InvalidStatus(status_code));
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
                let expected_duration = Duration::from_secs_f64(chunk_size as f64 / speed_limit as f64);
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
    async fn download_sequential_to_writer<W>(
        &self,
        url: &str,
        writer: &mut W,
        progress_callback: Option<ProgressCallback>,
        resume_from: u64,
    ) -> Result<u64>
    where
        W: AsyncWriteExt + Unpin + Send,
    {
        let range_header = if resume_from > 0 {
            Some(format!("bytes={}-", resume_from))
        } else {
            None
        };

        let request = self.build_request(url, range_header.as_deref())?;
        let response = request.send().await?;

        let status_code = response.status().as_u16();

        // Handle authentication challenges (401/407)
        // If we have credentials but didn't send them preemptively, retry with auth
        if (status_code == 401 || status_code == 407) && !self.client.config().auth_no_challenge {
            // Try configured auth first, then .netrc
            let auth = if let Some(ref auth) = self.client.config().auth {
                Some(auth.clone())
            } else {
                // Try .netrc file
                match crate::netrc::Netrc::from_default_location() {
                    Ok(Some(netrc)) => {
                        // Extract hostname from URL
                        if let Ok(parsed) = url::Url::parse(url) {
                            if let Some(host) = parsed.host_str() {
                                netrc.get(host)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            };

            if let Some(auth) = auth {
                // Retry with authentication (preserving range header if needed)
                let mut retry_request = self.client.client().get(url)
                    .basic_auth(&auth.username, Some(&auth.password));

                if let Some(ref range) = range_header {
                    retry_request = retry_request.header(reqwest::header::RANGE, range);
                }

                let retry_response = retry_request.send().await?;
                let retry_status = retry_response.status().as_u16();

                // If still unauthorized, return error
                if retry_status == 401 || retry_status == 407 {
                    return Err(Error::InvalidStatus(retry_status));
                }

                // Success! Continue with retry_response
                return self.process_writer_response(retry_response, url, writer, progress_callback, resume_from).await;
            } else {
                // No credentials available
                return Err(Error::InvalidStatus(status_code));
            }
        }

        // 204 No Content is a success but has no body - don't create file
        if status_code == 204 {
            return Ok(0);
        }

        // 416 Range Not Satisfiable means the file is already complete
        if status_code == 416 {
            // File is already fully downloaded
            return Ok(resume_from);
        }

        // Check for HTTP errors (4xx/5xx)
        if status_code >= 400 {
            // Only save error content if content_on_error is enabled
            if !self.client.config().content_on_error {
                return Err(Error::InvalidStatus(status_code));
            }
            // If content_on_error is enabled, continue downloading the error page
        } else if !response.status().is_success() && status_code != 206 {
            // Other non-success status codes (e.g., 1xx, 3xx unexpected)
            // 206 is acceptable for partial content (resume)
            return Err(Error::InvalidStatus(status_code));
        }

        self.process_writer_response(response, url, writer, progress_callback, resume_from).await
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

        // 204 No Content is a success but has no body - don't create file
        if status_code == 204 {
            return Ok(0);
        }

        // 416 Range Not Satisfiable means the file is already complete
        if status_code == 416 {
            return Ok(resume_from);
        }

        // Check for HTTP errors (4xx/5xx)
        if status_code >= 400 {
            // Only save error content if content_on_error is enabled
            if !self.client.config().content_on_error {
                return Err(Error::InvalidStatus(status_code));
            }
            // If content_on_error is enabled, continue downloading the error page
        } else if !response.status().is_success() && status_code != 206 {
            // Other non-success status codes (e.g., 1xx, 3xx unexpected)
            // 206 is acceptable for partial content (resume)
            return Err(Error::InvalidStatus(status_code));
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
                let expected_duration = Duration::from_secs_f64(chunk_size as f64 / speed_limit as f64);
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

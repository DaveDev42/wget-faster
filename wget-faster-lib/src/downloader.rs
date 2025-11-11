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

        // Check status code - for HEAD requests, 4xx and 5xx are informational, not fatal
        // We'll let the actual GET request handle the error
        if metadata.status_code >= 400 && metadata.status_code < 600 {
            // For HEAD, we continue to GET which will handle the error properly
            // This allows wget-compatible behavior where some servers respond differently to HEAD vs GET
        }

        // Check timestamping - skip if local file is newer
        if self.client.config().timestamping && path.exists() {
            if let Some(ref remote_modified) = metadata.last_modified {
                let local_time = tokio::fs::metadata(&path).await?.modified()?;

                // Parse remote Last-Modified header (RFC 2822 or RFC 3339 format)
                if let Ok(remote_time) = httpdate::parse_http_date(remote_modified) {
                    if local_time >= remote_time {
                        // Local file is newer or same age, skip download
                        return Ok(DownloadResult {
                            data: DownloadedData::new_file(path, 0, false),
                            url: url.to_string(),
                            metadata,
                        });
                    }
                }
            }
        }

        // Check if file exists for resume
        let resume_from = if path.exists() {
            tokio::fs::metadata(&path).await?.len()
        } else {
            0
        };

        // Create or open file
        let mut file = if resume_from > 0 {
            tokio::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(&path)
                .await?
        } else {
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

        // 204 No Content is a success but has no body
        if status_code == 204 {
            return Ok(Bytes::new());
        }

        if !response.status().is_success() {
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

        // 416 Range Not Satisfiable means the file is already complete
        if status_code == 416 {
            // File is already fully downloaded
            return Ok(resume_from);
        }

        if !response.status().is_success() && status_code != 206 {
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

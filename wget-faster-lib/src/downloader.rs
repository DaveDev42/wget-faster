use crate::{
    Error, Result, DownloadConfig, HttpClient, Output, ProgressCallback,
    ProgressInfo, output::DownloadedData, parallel,
};
use bytes::Bytes;
use futures_util::StreamExt;
use std::path::PathBuf;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Main downloader struct
pub struct Downloader {
    client: HttpClient,
}

impl Downloader {
    /// Create a new downloader with the given configuration
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

    /// Download to memory
    pub async fn download_to_memory(&self, url: &str) -> Result<Bytes> {
        self.download_to_memory_with_progress(url, None).await
    }

    /// Download to memory with progress callback
    pub async fn download_to_memory_with_progress(
        &self,
        url: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Bytes> {
        // Get metadata
        let metadata = self.client.get_metadata(url).await?;

        // Use parallel download if supported and beneficial
        if metadata.supports_range {
            if let Some(total_size) = metadata.content_length {
                if total_size > 1024 * 1024 * 10 {
                    // Use parallel for files > 10MB
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

    /// Download to file
    pub async fn download_to_file(&self, url: &str, path: PathBuf) -> Result<DownloadResult> {
        self.download_to_file_with_progress(url, path, None).await
    }

    /// Download to file with progress callback
    pub async fn download_to_file_with_progress(
        &self,
        url: &str,
        path: PathBuf,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<DownloadResult> {
        // Get metadata first
        let metadata = self.client.get_metadata(url).await?;

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
                if total_size > 1024 * 1024 * 10 {
                    // Use parallel for files > 10MB
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

    /// Download with custom output
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

        if !response.status().is_success() {
            return Err(Error::InvalidStatus(response.status().as_u16()));
        }

        let total_size = response.content_length();
        let mut downloaded = 0u64;
        let start_time = Instant::now();

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;

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

        if !response.status().is_success() && response.status().as_u16() != 206 {
            return Err(Error::InvalidStatus(response.status().as_u16()));
        }

        let total_size = response.content_length().map(|s| s + resume_from);
        let mut downloaded = resume_from;
        let start_time = Instant::now();

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            writer.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

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
#[derive(Debug)]
pub struct DownloadResult {
    /// Downloaded data
    pub data: DownloadedData,

    /// URL that was downloaded
    pub url: String,

    /// Resource metadata
    pub metadata: crate::client::ResourceMetadata,
}

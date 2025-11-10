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
        // Check if file exists for resume
        let resume_from = if path.exists() {
            tokio::fs::metadata(&path).await?.len()
        } else {
            0
        };

        // Get metadata
        let metadata = self.client.get_metadata(url).await?;

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
        let response = self.client.client().get(url).send().await?;

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
        let mut request = self.client.client().get(url);

        // Add Range header for resume
        if resume_from > 0 {
            request = request.header(
                reqwest::header::RANGE,
                format!("bytes={}-", resume_from),
            );
        }

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

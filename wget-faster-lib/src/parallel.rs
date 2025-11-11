use crate::{Error, Result, HttpClient, ProgressInfo, ProgressCallback};
use bytes::{Bytes, BytesMut};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;

/// Download a chunk of data using HTTP Range request
pub async fn download_chunk(
    client: &HttpClient,
    url: &str,
    start: u64,
    end: u64,
) -> Result<Bytes> {
    let range_header = format!("bytes={}-{}", start, end);

    let response = client
        .client()
        .get(url)
        .header(reqwest::header::RANGE, range_header)
        .send()
        .await?;

    if !response.status().is_success() && response.status().as_u16() != 206 {
        return Err(Error::InvalidStatus(response.status().as_u16()));
    }

    let bytes = response.bytes().await?;
    Ok(bytes)
}

/// Download file in parallel using multiple Range requests
pub async fn download_parallel(
    client: &HttpClient,
    url: &str,
    total_size: u64,
    progress_callback: Option<ProgressCallback>,
) -> Result<Bytes> {
    let num_chunks = client.config().parallel_chunks;

    // Calculate chunk size
    let chunk_size = if let Some(size) = client.config().chunk_size {
        size
    } else {
        // Auto-determine chunk size (minimum 1MB, maximum total_size / num_chunks)
        std::cmp::max(1024 * 1024, total_size / num_chunks as u64)
    };

    let mut chunks = Vec::new();
    let mut start = 0u64;

    while start < total_size {
        let end = std::cmp::min(start + chunk_size - 1, total_size - 1);
        chunks.push((start, end));
        start = end + 1;
    }

    // Track progress
    let downloaded = Arc::new(Mutex::new(0u64));
    let start_time = Instant::now();

    // Download chunks in parallel
    let mut tasks = Vec::new();

    for (start, end) in chunks {
        let client = client.clone();
        let url = url.to_string();
        let downloaded = Arc::clone(&downloaded);
        let progress_callback = progress_callback.clone();
        let url_for_progress = url.clone();

        let task = tokio::spawn(async move {
            let chunk_data = download_chunk(&client, &url, start, end).await?;

            // Update progress
            if let Some(callback) = progress_callback {
                let mut downloaded_guard = downloaded.lock().await;
                *downloaded_guard += chunk_data.len() as u64;

                let mut progress = ProgressInfo::new(url_for_progress);
                progress.total_size = Some(total_size);
                progress.update(chunk_data.len() as u64, start_time);
                progress.downloaded = *downloaded_guard;

                callback(progress);
            }

            Ok::<_, Error>((start, chunk_data))
        });

        tasks.push(task);
    }

    // Wait for all chunks to complete
    let mut results = Vec::new();
    for task in tasks {
        let result = task.await
            .map_err(|e| Error::ChunkError(format!("Task join error: {}", e)))?
            .map_err(|e| Error::ChunkError(format!("Chunk download failed: {}", e)))?;
        results.push(result);
    }

    // Sort by start position
    results.sort_by_key(|(start, _)| *start);

    // Combine chunks
    let mut combined = BytesMut::with_capacity(total_size as usize);
    for (_, data) in results {
        combined.extend_from_slice(&data);
    }

    Ok(combined.freeze())
}

/// Download to a writer in parallel
pub async fn download_parallel_to_writer<W>(
    client: &HttpClient,
    url: &str,
    total_size: u64,
    writer: &mut W,
    progress_callback: Option<ProgressCallback>,
) -> Result<()>
where
    W: AsyncWriteExt + Unpin + Send,
{
    // For writers, we download sequentially to maintain order
    // In a more advanced implementation, we could use a temp file for random writes

    let num_chunks = client.config().parallel_chunks;
    let chunk_size = if let Some(size) = client.config().chunk_size {
        size
    } else {
        std::cmp::max(1024 * 1024, total_size / num_chunks as u64)
    };

    let downloaded = Arc::new(Mutex::new(0u64));
    let start_time = Instant::now();

    let mut start = 0u64;
    while start < total_size {
        let end = std::cmp::min(start + chunk_size - 1, total_size - 1);

        let chunk_data = download_chunk(client, url, start, end).await?;
        writer.write_all(&chunk_data).await?;

        // Update progress
        if let Some(callback) = &progress_callback {
            let mut downloaded_guard = downloaded.lock().await;
            *downloaded_guard += chunk_data.len() as u64;

            let mut progress = ProgressInfo::new(url.to_string());
            progress.total_size = Some(total_size);
            progress.update(chunk_data.len() as u64, start_time);
            progress.downloaded = *downloaded_guard;

            callback(progress);
        }

        start = end + 1;
    }

    writer.flush().await?;
    Ok(())
}

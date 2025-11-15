/// Adaptive download strategy that automatically adjusts chunk size and connection count
/// based on network conditions and observed performance.
use crate::{Error, HttpClient, ProgressCallback, ProgressInfo, Result};
use bytes::Bytes;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Performance statistics for a chunk download
#[derive(Debug, Clone)]
struct ChunkStats {
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    duration: Duration,
    speed: f64, // bytes per second
}

/// Adaptive download manager
pub struct AdaptiveDownloader {
    client: Arc<HttpClient>,
    min_chunk_size: u64,
    max_chunk_size: u64,
    initial_chunks: usize,
    max_chunks: usize,
}

impl AdaptiveDownloader {
    /// Create a new adaptive downloader
    ///
    /// # Arguments
    ///
    /// * `client` - Arc-wrapped `HttpClient` for making HTTP requests
    ///
    /// # Default Parameters
    ///
    /// - `min_chunk_size`: 256 KB - Minimum chunk size to avoid overhead
    /// - `max_chunk_size`: 10 MB - Maximum chunk size to avoid timeouts
    /// - `initial_chunks`: 4 - Starting number of parallel connections
    /// - `max_chunks`: 32 - Maximum parallel connections to avoid server overload
    ///
    /// # Performance Strategy
    ///
    /// The downloader starts conservatively (4 chunks) and adapts based on:
    /// 1. **Speed variance** - Low variance → larger chunks, fewer connections
    /// 2. **Slow chunks** - >30% slow chunks → smaller chunks, more connections
    /// 3. **Performance trend** - Improving → add connections, degrading → reduce
    pub fn new(client: Arc<HttpClient>) -> Self {
        Self {
            client,
            min_chunk_size: 256 * 1024,       // 256 KB
            max_chunk_size: 10 * 1024 * 1024, // 10 MB
            initial_chunks: 4,
            max_chunks: 32,
        }
    }

    /// Download with adaptive chunk sizing
    pub async fn download_adaptive(
        &self,
        url: &str,
        total_size: u64,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Bytes> {
        // Start with initial chunk count
        let mut chunk_count = self.initial_chunks;
        let mut chunk_size = self.calculate_chunk_size(total_size, chunk_count);

        let start_time = Instant::now();
        let downloaded = Arc::new(Mutex::new(0u64));
        let stats = Arc::new(Mutex::new(Vec::new()));

        // Download first batch of chunks
        let mut position = 0u64;
        let mut result_data = Vec::new();

        while position < total_size {
            // Adjust chunk size based on previous performance
            if !stats.lock().await.is_empty() {
                chunk_size = self.adjust_chunk_size(&stats.lock().await, chunk_size);
                chunk_count = self.adjust_chunk_count(&stats.lock().await, chunk_count);
            }

            // Download next batch of chunks
            let batch_end = std::cmp::min(position + (chunk_size * chunk_count as u64), total_size);
            let batch_chunks = self.create_chunks(position, batch_end, chunk_size);

            let batch_results = self
                .download_chunks(
                    url,
                    batch_chunks,
                    &downloaded,
                    &stats,
                    start_time,
                    total_size,
                    progress_callback.clone(),
                )
                .await?;

            // Merge results
            for chunk_data in batch_results {
                result_data.extend_from_slice(&chunk_data);
            }

            position = batch_end;
        }

        Ok(Bytes::from(result_data))
    }

    /// Calculate initial chunk size
    fn calculate_chunk_size(&self, total_size: u64, chunk_count: usize) -> u64 {
        let size = total_size / chunk_count as u64;
        size.clamp(self.min_chunk_size, self.max_chunk_size)
    }

    /// Adjust chunk size based on observed performance
    fn adjust_chunk_size(&self, stats: &[ChunkStats], current_size: u64) -> u64 {
        if stats.len() < 2 {
            return current_size;
        }

        // Calculate average speed
        let avg_speed: f64 = stats.iter().map(|s| s.speed).sum::<f64>() / stats.len() as f64;

        // Find slow chunks (below 70% of average)
        let slow_threshold = avg_speed * 0.7;
        let slow_count = stats.iter().filter(|s| s.speed < slow_threshold).count();

        // If many chunks are slow, decrease chunk size
        if slow_count as f64 / stats.len() as f64 > 0.3 {
            let new_size = (current_size as f64 * 0.75) as u64;
            return new_size.clamp(self.min_chunk_size, self.max_chunk_size);
        }

        // If all chunks are fast and uniform, increase chunk size
        let speed_variance = self.calculate_variance(stats.iter().map(|s| s.speed).collect());
        if speed_variance < avg_speed * 0.2 {
            let new_size = (current_size as f64 * 1.25) as u64;
            return new_size.clamp(self.min_chunk_size, self.max_chunk_size);
        }

        current_size
    }

    /// Adjust chunk count based on observed performance
    fn adjust_chunk_count(&self, stats: &[ChunkStats], current_count: usize) -> usize {
        if stats.len() < 3 {
            return current_count;
        }

        // Calculate efficiency (did more chunks help?)
        let avg_speed: f64 = stats.iter().map(|s| s.speed).sum::<f64>() / stats.len() as f64;

        // If we have recent stats, compare
        let recent_count = stats.len().min(5);
        let recent_avg: f64 = stats[stats.len() - recent_count..]
            .iter()
            .map(|s| s.speed)
            .sum::<f64>()
            / recent_count as f64;

        // If recent performance is better, we can add more chunks
        if recent_avg > avg_speed * 1.1 && current_count < self.max_chunks {
            return current_count + 2;
        }

        // If performance is degrading, reduce chunks
        if recent_avg < avg_speed * 0.9 && current_count > 2 {
            return current_count - 1;
        }

        current_count
    }

    /// Create chunk ranges
    fn create_chunks(&self, start: u64, end: u64, chunk_size: u64) -> Vec<(u64, u64)> {
        let mut chunks = Vec::new();
        let mut pos = start;

        while pos < end {
            let chunk_end = std::cmp::min(pos + chunk_size, end);
            chunks.push((pos, chunk_end - 1));
            pos = chunk_end;
        }

        chunks
    }

    /// Download multiple chunks in parallel
    async fn download_chunks(
        &self,
        url: &str,
        chunks: Vec<(u64, u64)>,
        downloaded: &Arc<Mutex<u64>>,
        stats: &Arc<Mutex<Vec<ChunkStats>>>,
        start_time: Instant,
        total_size: u64,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Vec<Bytes>> {
        let mut tasks = Vec::new();

        for (start, end) in chunks {
            let client = self.client.clone();
            let url = url.to_string();
            let downloaded = Arc::clone(downloaded);
            let stats = Arc::clone(stats);
            let progress_callback = progress_callback.clone();

            let task = tokio::spawn(async move {
                let chunk_start = Instant::now();
                let size = end - start + 1;

                // Download chunk
                let range_header = format!("bytes={start}-{end}");
                let response = client
                    .client()
                    .get(&url)
                    .header(reqwest::header::RANGE, range_header)
                    .send()
                    .await?;

                if !response.status().is_success() && response.status().as_u16() != 206 {
                    return Err(Error::InvalidStatus(response.status().as_u16()));
                }

                let chunk_data = response.bytes().await?;
                let chunk_duration = chunk_start.elapsed();

                // Record stats
                let speed = size as f64 / chunk_duration.as_secs_f64();
                stats.lock().await.push(ChunkStats {
                    size,
                    duration: chunk_duration,
                    speed,
                });

                // Update progress
                if let Some(callback) = progress_callback {
                    let mut downloaded_guard = downloaded.lock().await;
                    *downloaded_guard += chunk_data.len() as u64;

                    let mut progress = ProgressInfo::new(url);
                    progress.total_size = Some(total_size);
                    progress.update(chunk_data.len() as u64, start_time);
                    progress.downloaded = *downloaded_guard;

                    callback(progress);
                }

                Ok::<_, Error>((start, chunk_data))
            });

            tasks.push(task);
        }

        // Wait for all chunks
        let mut results = Vec::new();
        for task in tasks {
            let result = task
                .await
                .map_err(|e| Error::ChunkError(format!("Task join error: {e}")))?
                .map_err(|e| Error::ChunkError(format!("Chunk download failed: {e}")))?;
            results.push(result);
        }

        // Sort by position
        results.sort_by_key(|(pos, _)| *pos);

        // Extract data in order
        Ok(results.into_iter().map(|(_, data)| data).collect())
    }

    /// Calculate variance of a set of values
    fn calculate_variance(&self, values: Vec<f64>) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance.sqrt()
    }
}

use std::path::PathBuf;
use bytes::Bytes;

/// Output destination for downloaded content
#[derive(Debug)]
pub enum Output {
    /// Store in memory
    Memory,

    /// Write to file
    File(PathBuf),
}


/// Result of a download operation
#[derive(Debug)]
pub struct DownloadedData {
    /// Downloaded content (only for Memory output)
    pub data: Option<Bytes>,

    /// Path to downloaded file (only for File output)
    pub file_path: Option<PathBuf>,

    /// Total bytes downloaded
    pub total_bytes: u64,

    /// Whether the download was resumed
    pub was_resumed: bool,
}

impl DownloadedData {
    pub fn new_memory(data: Bytes) -> Self {
        let total_bytes = data.len() as u64;
        Self {
            data: Some(data),
            file_path: None,
            total_bytes,
            was_resumed: false,
        }
    }

    pub fn new_file(path: PathBuf, total_bytes: u64, was_resumed: bool) -> Self {
        Self {
            data: None,
            file_path: Some(path),
            total_bytes,
            was_resumed,
        }
    }
}

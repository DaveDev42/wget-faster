use std::path::PathBuf;
use bytes::Bytes;

/// Output destination for downloaded content
///
/// Specifies where downloaded data should be written. Choose `Memory` for
/// small files or when you need to process the data immediately. Use `File`
/// for larger downloads or when you want to save directly to disk.
///
/// # Examples
///
/// ```no_run
/// use wget_faster_lib::Output;
/// use std::path::PathBuf;
///
/// // Download to memory
/// let output = Output::Memory;
///
/// // Download to file
/// let output = Output::File(PathBuf::from("download.zip"));
/// ```
#[derive(Debug)]
pub enum Output {
    /// Store downloaded content in memory as `Bytes`
    Memory,

    /// Write downloaded content to a file at the specified path
    File(PathBuf),
}


/// Container for downloaded data
///
/// Holds the result of a download operation, which can be either in-memory data
/// or a file path, along with metadata about the download.
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
///     println!("Downloaded {} bytes", result.data.total_bytes);
///     if result.data.was_resumed {
///         println!("Download was resumed from previous attempt");
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct DownloadedData {
    /// Downloaded content as bytes (only present when using `Output::Memory`)
    pub data: Option<Bytes>,

    /// Path to downloaded file (only present when using `Output::File`)
    pub file_path: Option<PathBuf>,

    /// Total number of bytes downloaded
    pub total_bytes: u64,

    /// Whether this download was resumed from a partial file
    pub was_resumed: bool,
}

impl DownloadedData {
    /// Create a new `DownloadedData` for in-memory downloads
    ///
    /// # Arguments
    ///
    /// * `data` - The downloaded content as `Bytes`
    pub fn new_memory(data: Bytes) -> Self {
        let total_bytes = data.len() as u64;
        Self {
            data: Some(data),
            file_path: None,
            total_bytes,
            was_resumed: false,
        }
    }

    /// Create a new `DownloadedData` for file downloads
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the downloaded file
    /// * `total_bytes` - Total number of bytes downloaded
    /// * `was_resumed` - Whether the download was resumed from a partial file
    pub fn new_file(path: PathBuf, total_bytes: u64, was_resumed: bool) -> Self {
        Self {
            data: None,
            file_path: Some(path),
            total_bytes,
            was_resumed,
        }
    }

    /// Get the file path if this is a file download
    ///
    /// # Returns
    ///
    /// `Some(&PathBuf)` if this is a file download, `None` if it's in memory
    pub fn path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Get the downloaded bytes if this is a memory download
    ///
    /// # Returns
    ///
    /// `Some(&Bytes)` if this is a memory download, `None` if it's a file
    pub fn bytes(&self) -> Option<&Bytes> {
        self.data.as_ref()
    }
}

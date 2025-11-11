//! # wget-faster-lib
//!
//! A high-performance async HTTP downloader library with parallel Range request support.
//!
//! This library provides:
//! - Full async/non-blocking API
//! - Multiple output modes: memory, file, or custom AsyncWrite
//! - Parallel downloads using HTTP Range requests
//! - Progress tracking with callbacks
//! - Resume support for partial downloads
//! - Cookie, authentication, and proxy support
//!
//! ## Example
//!
//! ```no_run
//! use wget_faster_lib::{Downloader, DownloadConfig, Output};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let downloader = Downloader::new(DownloadConfig::default())?;
//!
//!     // Download to memory
//!     let bytes = downloader.download_to_memory("https://example.com/file.txt").await?;
//!
//!     // Download to file
//!     downloader.download_to_file(
//!         "https://example.com/file.txt",
//!         "output.txt".into()
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```

mod error;
mod config;
mod client;
mod progress;
mod output;
mod parallel;
mod downloader;
mod cookies;
mod adaptive;
mod recursive;
mod netrc;

pub use error::{Error, Result};
pub use config::{DownloadConfig, RetryConfig, ProxyConfig, AuthConfig, AuthType, HttpMethod};
pub use client::{HttpClient, ResourceMetadata};
pub use downloader::{Downloader, DownloadResult};
pub use progress::{ProgressInfo, ProgressCallback, format_bytes, format_bytes_per_sec, format_duration};
pub use output::{Output, DownloadedData};
pub use cookies::{Cookie, CookieJar};
pub use adaptive::AdaptiveDownloader;
pub use recursive::{RecursiveDownloader, RecursiveConfig};
pub use netrc::{Netrc, NetrcEntry};

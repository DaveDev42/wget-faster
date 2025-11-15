//! # wget-faster-lib
//!
//! A high-performance async HTTP downloader library with parallel Range request support.
//!
//! This library provides:
//! - Full async/non-blocking API
//! - Multiple output modes: memory, file, or custom `AsyncWrite`
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

mod adaptive;
mod auth_handler;
mod client;
mod config;
mod cookies;
mod downloader;
mod error;
mod link_converter;
mod netrc;
mod output;
mod parallel;
mod progress;
mod recursive;
mod response_handler;
mod timestamping;

pub use adaptive::AdaptiveDownloader;
pub use client::{HttpClient, ResourceMetadata};
pub use config::{
    apply_filename_restrictions, AuthConfig, AuthType, DownloadConfig, FilenameRestriction,
    HttpMethod, ProxyConfig, RetryConfig,
};
pub use cookies::{Cookie, CookieJar};
pub use downloader::{DownloadResult, Downloader};
pub use error::{Error, Result};
pub use link_converter::LinkConverter;
pub use netrc::{Netrc, NetrcEntry};
pub use output::{DownloadedData, Output};
pub use progress::{
    format_bytes, format_bytes_per_sec, format_duration, ProgressCallback, ProgressInfo,
};
pub use recursive::{RecursiveConfig, RecursiveDownloader};

/// robots.txt parsing and handling
pub mod robots;

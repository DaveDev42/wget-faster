use std::io;
use thiserror::Error;

/// Result type alias using the library's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for download operations
///
/// All error types implement the `std::error::Error` trait and can be
/// displayed with user-friendly error messages.
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] http::header::InvalidHeaderValue),

    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    #[error("Server does not support range requests")]
    RangeNotSupported,

    #[error("Content length not available")]
    ContentLengthUnavailable,

    #[error("Download failed after {0} retries")]
    MaxRetriesExceeded(usize),

    #[error("Timeout exceeded")]
    Timeout,

    #[error("Invalid response status: {0}")]
    InvalidStatus(u16),

    #[error("Chunk download failed: {0}")]
    ChunkError(String),

    #[error("Failed to create temporary file: {0}")]
    TempFileError(String),

    #[error("Failed to write to output: {0}")]
    WriteError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Unknown(err.to_string())
    }
}

impl Error {
    /// Format error in wget-style output
    ///
    /// Example: "wget: failed: Connection refused."
    pub fn format_wget_style(&self) -> String {
        match self {
            Error::HttpError(e) => {
                if e.is_timeout() {
                    "Read error (Connection timed out).".to_string()
                } else if e.is_connect() {
                    format!("Unable to establish connection: {}", e)
                } else {
                    format!("HTTP error: {}", e)
                }
            }
            Error::IoError(e) => format!("File write error: {}", e),
            Error::InvalidUrl(_) => "Invalid URL format.".to_string(),
            Error::RangeNotSupported => "Server does not support byte ranges.".to_string(),
            Error::ContentLengthUnavailable => "Content-Length header missing.".to_string(),
            Error::MaxRetriesExceeded(n) => {
                format!("Giving up after {} retries.", n)
            }
            Error::Timeout => "Read error (Connection timed out).".to_string(),
            Error::InvalidStatus(code) => {
                let status_text = match *code {
                    400 => "Bad Request",
                    401 => "Unauthorized",
                    403 => "Forbidden",
                    404 => "Not Found",
                    500 => "Internal Server Error",
                    502 => "Bad Gateway",
                    503 => "Service Unavailable",
                    _ => "Error",
                };
                format!("{}: {} {}", code, status_text, self)
            }
            Error::ChunkError(msg) => format!("Download failed: {}", msg),
            Error::TempFileError(msg) => format!("Cannot create temp file: {}", msg),
            Error::WriteError(msg) => format!("File write error: {}", msg),
            Error::ConfigError(msg) => format!("Configuration error: {}", msg),
            Error::Unknown(msg) => format!("Error: {}", msg),
            _ => self.to_string(),
        }
    }

    /// Format error with URL context (wget-style)
    ///
    /// Example: "wget: https://example.com/file.txt: failed: Connection refused."
    pub fn format_with_url(&self, url: &str) -> String {
        format!("{}: {}", url, self.format_wget_style())
    }
}

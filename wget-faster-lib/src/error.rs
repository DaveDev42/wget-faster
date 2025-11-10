use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

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

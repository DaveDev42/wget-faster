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
    /// Get wget-compatible exit code for this error
    ///
    /// Exit codes:
    /// - 0: Success
    /// - 1: Generic error
    /// - 2: Parse error
    /// - 3: File I/O error
    /// - 4: Network failure (including server errors 5xx)
    /// - 5: SSL verification failure
    /// - 6: Authentication failure
    /// - 7: Protocol error
    /// - 8: Server error response (4xx client errors only)
    pub fn exit_code(&self) -> i32 {
        match self {
            // File I/O errors -> 3
            Error::IoError(_) | Error::TempFileError(_) | Error::WriteError(_) => 3,

            // Network failures -> 4
            Error::Timeout => 4,
            Error::HttpError(e) if e.is_timeout() || e.is_connect() => 4,

            // SSL verification failure -> 5
            Error::HttpError(e) if e.to_string().contains("certificate")
                                || e.to_string().contains("tls")
                                || e.to_string().contains("ssl") => 5,

            // Authentication failure -> 6
            Error::InvalidStatus(401) | Error::InvalidStatus(407) => 6,

            // Client errors (4xx) -> 8
            Error::InvalidStatus(code) if *code >= 400 && *code < 500 => 8,

            // Server errors (5xx) -> 4
            Error::InvalidStatus(code) if *code >= 500 => 4,

            // Protocol errors -> 7
            Error::RangeNotSupported | Error::ContentLengthUnavailable => 7,

            // Parse errors -> 2
            Error::InvalidUrl(_) | Error::InvalidHeader(_) | Error::InvalidHeaderName(_) => 2,
            Error::ConfigError(_) => 2,

            // Generic error -> 1
            _ => 1,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes_client_errors() {
        // Client errors (4xx except 401/407) should return exit code 8
        assert_eq!(Error::InvalidStatus(400).exit_code(), 8, "400 Bad Request");
        assert_eq!(Error::InvalidStatus(403).exit_code(), 8, "403 Forbidden");
        assert_eq!(Error::InvalidStatus(404).exit_code(), 8, "404 Not Found");
        assert_eq!(Error::InvalidStatus(410).exit_code(), 8, "410 Gone");
        assert_eq!(Error::InvalidStatus(429).exit_code(), 8, "429 Too Many Requests");
        assert_eq!(Error::InvalidStatus(499).exit_code(), 8, "499 (last 4xx)");
    }

    #[test]
    fn test_exit_codes_auth_errors() {
        // Authentication errors should return exit code 6
        assert_eq!(Error::InvalidStatus(401).exit_code(), 6, "401 Unauthorized");
        assert_eq!(Error::InvalidStatus(407).exit_code(), 6, "407 Proxy Auth Required");
    }

    #[test]
    fn test_exit_codes_server_errors() {
        // Server errors (5xx) should return exit code 4
        assert_eq!(Error::InvalidStatus(500).exit_code(), 4, "500 Internal Server Error");
        assert_eq!(Error::InvalidStatus(502).exit_code(), 4, "502 Bad Gateway");
        assert_eq!(Error::InvalidStatus(503).exit_code(), 4, "503 Service Unavailable");
        assert_eq!(Error::InvalidStatus(504).exit_code(), 4, "504 Gateway Timeout");
        assert_eq!(Error::InvalidStatus(599).exit_code(), 4, "599 (last 5xx)");
    }

    #[test]
    fn test_exit_codes_io_errors() {
        // File I/O errors should return exit code 3
        assert_eq!(Error::TempFileError("test".to_string()).exit_code(), 3);
        assert_eq!(Error::WriteError("test".to_string()).exit_code(), 3);
    }

    #[test]
    fn test_exit_codes_network_errors() {
        // Network failures should return exit code 4
        assert_eq!(Error::Timeout.exit_code(), 4);
    }

    #[test]
    fn test_exit_codes_protocol_errors() {
        // Protocol errors should return exit code 7
        assert_eq!(Error::RangeNotSupported.exit_code(), 7);
        assert_eq!(Error::ContentLengthUnavailable.exit_code(), 7);
    }
}

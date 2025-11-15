/// HTTP response handling and status code validation
///
/// Consolidates HTTP response logic including:
/// - Status code validation and classification
/// - Special status handling (204, 304, 416)
/// - Error response handling with `content_on_error` support
use crate::DownloadConfig;

/// Response status category for decision making
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStatus {
    /// Successful response (2xx)
    Success,
    /// No content - successful but empty body (204)
    NoContent,
    /// Not modified - file is up to date (304)
    NotModified,
    /// Range not satisfiable - file already complete (416)
    RangeNotSatisfiable,
    /// Client error (4xx) - may save content if `content_on_error` is true
    ClientError,
    /// Server error (5xx) - may save content if `content_on_error` is true
    ServerError,
    /// Authentication challenge (401/407) - handled separately
    AuthChallenge,
    /// Other/unexpected status
    Other,
}

impl ResponseStatus {
    /// Classify an HTTP status code
    ///
    /// # Arguments
    ///
    /// * `status_code` - HTTP status code to classify
    ///
    /// # Returns
    ///
    /// Returns the response status category
    pub fn from_status_code(status_code: u16) -> Self {
        match status_code {
            // Success codes
            200..=203 | 205..=299 => Self::Success,

            // Special success cases
            204 => Self::NoContent,
            304 => Self::NotModified,
            416 => Self::RangeNotSatisfiable,

            // Authentication challenges
            401 | 407 => Self::AuthChallenge,

            // Client errors
            400..=499 => Self::ClientError,

            // Server errors
            500..=599 => Self::ServerError,

            // Other
            _ => Self::Other,
        }
    }

    /// Check if status indicates success (including special success codes)
    pub fn is_success_or_special(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::NoContent | Self::NotModified | Self::RangeNotSatisfiable
        )
    }
}

/// Check if we should create a file for this response
///
/// # Arguments
///
/// * `status_code` - HTTP status code
/// * `downloaded_bytes` - Number of bytes downloaded
/// * `resumed_from` - Starting byte offset (for resume)
///
/// # Returns
///
/// Returns `false` if the file should not be created (204 or 0 bytes without resume)
pub fn should_create_file(status_code: u16, downloaded_bytes: u64, resumed_from: u64) -> bool {
    // Don't create file for 204 No Content
    if status_code == 204 {
        return false;
    }

    // Don't create empty files unless we're resuming
    if downloaded_bytes == 0 && resumed_from == 0 {
        return false;
    }

    true
}

/// Determine if we should proceed with download or return error
///
/// # Arguments
///
/// * `status_code` - HTTP status code
/// * `config` - Download configuration
///
/// # Returns
///
/// Returns `Ok(true)` if download should proceed, `Ok(false)` if should skip (304/416),
/// or `Err(status_code)` if should return error.
pub fn should_proceed_download(status_code: u16, config: &DownloadConfig) -> Result<bool, u16> {
    let status = ResponseStatus::from_status_code(status_code);

    match status {
        // Success - proceed
        ResponseStatus::Success => Ok(true),

        // No content - proceed but will be empty
        ResponseStatus::NoContent => Ok(true),

        // Not modified or range not satisfiable - skip download
        ResponseStatus::NotModified | ResponseStatus::RangeNotSatisfiable => Ok(false),

        // Auth challenge - should be handled before this check
        ResponseStatus::AuthChallenge => Err(status_code),

        // Error responses - check content_on_error
        ResponseStatus::ClientError | ResponseStatus::ServerError => {
            if config.content_on_error {
                // Proceed to download error page
                Ok(true)
            } else {
                // Don't download error content
                Err(status_code)
            }
        },

        // Other - error
        ResponseStatus::Other => Err(status_code),
    }
}

/// Check if status code indicates a special case that needs handling
///
/// # Arguments
///
/// * `status_code` - HTTP status code
///
/// # Returns
///
/// Returns `Some(action)` describing what to do, or `None` if normal processing
pub fn check_special_status(status_code: u16) -> Option<&'static str> {
    match status_code {
        204 => Some("no_content"),
        304 => Some("not_modified"),
        416 => Some("range_not_satisfiable"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_status_classification() {
        assert_eq!(ResponseStatus::from_status_code(200), ResponseStatus::Success);
        assert_eq!(ResponseStatus::from_status_code(201), ResponseStatus::Success);
        assert_eq!(ResponseStatus::from_status_code(204), ResponseStatus::NoContent);
        assert_eq!(ResponseStatus::from_status_code(304), ResponseStatus::NotModified);
        assert_eq!(ResponseStatus::from_status_code(401), ResponseStatus::AuthChallenge);
        assert_eq!(ResponseStatus::from_status_code(404), ResponseStatus::ClientError);
        assert_eq!(ResponseStatus::from_status_code(416), ResponseStatus::RangeNotSatisfiable);
        assert_eq!(ResponseStatus::from_status_code(500), ResponseStatus::ServerError);
    }

    #[test]
    fn test_is_success_or_special() {
        assert!(ResponseStatus::Success.is_success_or_special());
        assert!(ResponseStatus::NoContent.is_success_or_special());
        assert!(ResponseStatus::NotModified.is_success_or_special());
        assert!(ResponseStatus::RangeNotSatisfiable.is_success_or_special());
        assert!(!ResponseStatus::ClientError.is_success_or_special());
        assert!(!ResponseStatus::ServerError.is_success_or_special());
    }

    #[test]
    fn test_should_create_file() {
        // Don't create for 204
        assert!(!should_create_file(204, 0, 0));
        assert!(!should_create_file(204, 100, 0));

        // Don't create empty files without resume
        assert!(!should_create_file(200, 0, 0));

        // Create empty files when resuming
        assert!(should_create_file(200, 0, 100));

        // Create non-empty files
        assert!(should_create_file(200, 100, 0));
    }

    #[test]
    fn test_should_proceed_download() {
        let mut config = DownloadConfig::default();

        // Success - proceed
        assert_eq!(should_proceed_download(200, &config), Ok(true));

        // No content - proceed (but empty)
        assert_eq!(should_proceed_download(204, &config), Ok(true));

        // Not modified - skip
        assert_eq!(should_proceed_download(304, &config), Ok(false));

        // Range not satisfiable - skip
        assert_eq!(should_proceed_download(416, &config), Ok(false));

        // Errors without content_on_error - error
        config.content_on_error = false;
        assert_eq!(should_proceed_download(404, &config), Err(404));
        assert_eq!(should_proceed_download(500, &config), Err(500));

        // Errors with content_on_error - proceed
        config.content_on_error = true;
        assert_eq!(should_proceed_download(404, &config), Ok(true));
        assert_eq!(should_proceed_download(500, &config), Ok(true));
    }

    #[test]
    fn test_check_special_status() {
        assert_eq!(check_special_status(204), Some("no_content"));
        assert_eq!(check_special_status(304), Some("not_modified"));
        assert_eq!(check_special_status(416), Some("range_not_satisfiable"));
        assert_eq!(check_special_status(200), None);
        assert_eq!(check_special_status(404), None);
    }
}

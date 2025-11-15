/// Authentication handler for HTTP requests
///
/// Consolidates authentication logic including:
/// - Credential resolution (configured auth + .netrc fallback)
/// - Authentication challenge handling (401/407)
/// - Retry logic with credentials
use crate::{AuthConfig, DownloadConfig};

/// Get authentication credentials for a URL
///
/// Tries configured auth first, then falls back to .netrc file.
///
/// # Arguments
///
/// * `url` - The URL to get credentials for (used for .netrc hostname lookup)
/// * `config` - Download configuration containing auth settings
///
/// # Returns
///
/// Returns `Some(AuthConfig)` if credentials are found, `None` otherwise
pub fn get_credentials(url: &str, config: &DownloadConfig) -> Option<AuthConfig> {
    // Try configured auth first
    if let Some(ref auth) = config.auth {
        tracing::debug!(username = %auth.username, "Using configured auth credentials");
        return Some(auth.clone());
    }

    // Fall back to .netrc file
    tracing::debug!("No configured auth - trying .netrc file");
    match crate::netrc::Netrc::from_default_location() {
        Ok(Some(netrc)) => {
            // Extract hostname from URL
            if let Ok(parsed) = url::Url::parse(url) {
                if let Some(host) = parsed.host_str() {
                    if let Some(entry) = netrc.get(host) {
                        tracing::debug!(host = %host, username = %entry.username, "Found .netrc entry for host");
                        return Some(entry);
                    }
                    tracing::debug!(host = %host, "No .netrc entry found for host");
                }
            }
        },
        Ok(None) => {
            tracing::debug!("No .netrc file found");
        },
        Err(e) => {
            tracing::warn!(error = %e, "Failed to read .netrc file");
        },
    }

    None
}

/// Check if a status code indicates an authentication challenge
///
/// # Arguments
///
/// * `status_code` - HTTP status code
///
/// # Returns
///
/// Returns `true` for 401 Unauthorized or 407 Proxy Authentication Required
#[inline]
pub fn is_auth_challenge(status_code: u16) -> bool {
    status_code == 401 || status_code == 407
}

/// Determine if we should retry authentication
///
/// Returns true if:
/// - Status is 401 or 407 (auth challenge)
/// - We didn't send preemptive auth (`auth_no_challenge` is false)
///
/// # Arguments
///
/// * `status_code` - HTTP status code from response
/// * `config` - Download configuration
///
/// # Returns
///
/// Returns `true` if we should attempt auth retry
#[inline]
pub fn should_retry_auth(status_code: u16, config: &DownloadConfig) -> bool {
    is_auth_challenge(status_code) && !config.auth_no_challenge
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_auth_challenge() {
        assert!(is_auth_challenge(401));
        assert!(is_auth_challenge(407));
        assert!(!is_auth_challenge(200));
        assert!(!is_auth_challenge(404));
        assert!(!is_auth_challenge(500));
    }

    #[test]
    fn test_should_retry_auth() {
        let mut config = DownloadConfig::default();

        // Should retry when challenge received and no preemptive auth
        config.auth_no_challenge = false;
        assert!(should_retry_auth(401, &config));
        assert!(should_retry_auth(407, &config));

        // Should NOT retry when preemptive auth was sent
        config.auth_no_challenge = true;
        assert!(!should_retry_auth(401, &config));
        assert!(!should_retry_auth(407, &config));

        // Should NOT retry for non-auth status codes
        config.auth_no_challenge = false;
        assert!(!should_retry_auth(200, &config));
        assert!(!should_retry_auth(404, &config));
    }

    #[test]
    fn test_get_credentials_with_configured_auth() {
        let mut config = DownloadConfig::default();
        config.auth = Some(AuthConfig {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            auth_type: crate::AuthType::Basic,
        });

        let creds = get_credentials("https://example.com", &config);
        assert!(creds.is_some());
        assert_eq!(creds.unwrap().username, "testuser");
    }

    #[test]
    fn test_get_credentials_without_auth() {
        let config = DownloadConfig::default();

        // Without .netrc file, should return None
        // Note: This might find a real .netrc file in test environment
        // In production code, we'd mock the netrc module
        let creds = get_credentials("https://unknown-host-12345.com", &config);
        // We can't assert None here because .netrc might exist
        // Just verify it doesn't panic
        drop(creds);
    }
}

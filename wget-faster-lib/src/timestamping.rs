/// Timestamping functionality for conditional downloads
///
/// Implements wget's timestamping (-N flag) behavior:
/// - Compare local and remote file modification times
/// - Skip download if local file is newer or same
/// - Re-download if remote file is newer
/// - Handle edge cases (missing timestamps, size mismatches)
use crate::{client::ResourceMetadata, output::DownloadedData, Result};
use std::path::Path;

/// Result of timestamp comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampAction {
    /// Download the file (local is older or doesn't exist)
    Download,
    /// Skip download (local is newer or same)
    Skip,
    /// Delete existing file and re-download
    DeleteAndDownload,
}

/// Check if we should download based on timestamping rules
///
/// # Arguments
///
/// * `path` - Local file path
/// * `metadata` - Remote resource metadata
///
/// # Returns
///
/// Returns `TimestampAction` indicating what to do
pub async fn check_timestamp(
    path: &Path,
    metadata: &ResourceMetadata,
) -> Result<(TimestampAction, Option<DownloadedData>)> {
    // If file doesn't exist, download
    if !path.exists() {
        tracing::debug!("File doesn't exist - will download");
        return Ok((TimestampAction::Download, None));
    }

    // Get local file metadata
    let local_metadata = tokio::fs::metadata(path).await?;
    let local_size = local_metadata.len();
    let local_time = local_metadata.modified()?;

    // If no remote Last-Modified header, download (server doesn't provide timestamp info)
    // This matches wget behavior
    let Some(ref remote_modified) = metadata.last_modified else {
        tracing::info!("No Last-Modified header from server - will download");
        return Ok((TimestampAction::DeleteAndDownload, None));
    };

    // Parse remote Last-Modified header
    let Ok(remote_time) = httpdate::parse_http_date(remote_modified) else {
        tracing::warn!(last_modified = %remote_modified, "Failed to parse Last-Modified header");
        return Ok((TimestampAction::DeleteAndDownload, None));
    };

    tracing::debug!(
        local_time = ?local_time,
        remote_time = ?remote_time,
        local_size,
        remote_size = ?metadata.content_length,
        "Comparing timestamps"
    );

    // Compare timestamps
    match local_time.cmp(&remote_time) {
        std::cmp::Ordering::Less => {
            // Local file is older, delete and re-download
            tracing::info!("Local file is older than remote - will re-download");
            Ok((TimestampAction::DeleteAndDownload, None))
        },
        std::cmp::Ordering::Greater => {
            // Local file is newer, skip download
            tracing::info!("Local file is newer than remote - skipping download");
            let result = DownloadedData::new_file(path.to_path_buf(), local_size, false);
            Ok((TimestampAction::Skip, Some(result)))
        },
        std::cmp::Ordering::Equal => {
            // Same timestamp - check file size
            tracing::debug!("Same timestamp - checking file size");

            if let Some(remote_size) = metadata.content_length {
                if local_size == remote_size {
                    // Same timestamp and size, skip download
                    tracing::info!("Same timestamp and size - skipping download");
                    let result = DownloadedData::new_file(path.to_path_buf(), local_size, false);
                    return Ok((TimestampAction::Skip, Some(result)));
                }
                // Same timestamp but different size - delete and re-download
                tracing::info!("Same timestamp but different size - will re-download");
                Ok((TimestampAction::DeleteAndDownload, None))
            } else {
                // No remote size info, skip download (same timestamp)
                tracing::info!("Same timestamp, no remote size - skipping download");
                let result = DownloadedData::new_file(path.to_path_buf(), local_size, false);
                Ok((TimestampAction::Skip, Some(result)))
            }
        },
    }
}

/// Set file modification time from server timestamp
///
/// # Arguments
///
/// * `path` - File path to modify
/// * `metadata` - Remote resource metadata containing Last-Modified header
/// * `verbose` - Whether to print warnings on error
///
/// # Returns
///
/// Returns Ok(()) on success, or Ok(()) with warning log on parse/set failure
pub fn set_file_timestamp(path: &Path, metadata: &ResourceMetadata, verbose: bool) -> Result<()> {
    let Some(ref last_modified_str) = metadata.last_modified else {
        tracing::debug!("No Last-Modified header - skipping file timestamp setting");
        return Ok(());
    };

    let Ok(remote_time) = httpdate::parse_http_date(last_modified_str) else {
        tracing::warn!(last_modified = %last_modified_str, "Failed to parse Last-Modified for setting file time");
        return Ok(());
    };

    tracing::debug!(
        path = %path.display(),
        remote_time = ?remote_time,
        "Setting file modification time to server timestamp"
    );

    // Convert SystemTime to FileTime for setting the modification time
    let file_time = filetime::FileTime::from_system_time(remote_time);

    // Set the file modification time (atime is set to current time, mtime to server time)
    if let Err(e) = filetime::set_file_mtime(path, file_time) {
        // Log error but don't fail the download
        tracing::warn!(path = %path.display(), error = %e, "Failed to set file modification time");
        if verbose {
            eprintln!("Warning: Failed to set file modification time: {e}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_timestamp_file_not_exists() {
        let path = Path::new("/nonexistent/file.txt");
        let metadata = ResourceMetadata {
            supports_range: false,
            content_length: Some(1000),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            etag: None,
            content_type: None,
            content_disposition: None,
            status_code: 200,
            headers: reqwest::header::HeaderMap::new(),
            auth_succeeded: false,
        };

        let (action, _) = check_timestamp(path, &metadata)
            .await
            .expect("Failed to check timestamp");
        assert_eq!(action, TimestampAction::Download);
    }

    #[test]
    fn test_timestamp_action_equality() {
        assert_eq!(TimestampAction::Download, TimestampAction::Download);
        assert_eq!(TimestampAction::Skip, TimestampAction::Skip);
        assert_eq!(TimestampAction::DeleteAndDownload, TimestampAction::DeleteAndDownload);
        assert_ne!(TimestampAction::Download, TimestampAction::Skip);
    }
}

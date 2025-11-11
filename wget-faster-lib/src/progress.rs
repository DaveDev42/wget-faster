use std::time::{Duration, Instant};
use std::sync::Arc;

/// Progress information for a download
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Total size in bytes (None if unknown)
    pub total_size: Option<u64>,

    /// Downloaded bytes so far
    pub downloaded: u64,

    /// Download speed in bytes per second
    pub speed: f64,

    /// Estimated time remaining (None if unknown)
    pub eta: Option<Duration>,

    /// Elapsed time since download started
    pub elapsed: Duration,

    /// Current URL being downloaded
    pub url: String,
}

impl ProgressInfo {
    /// Create new progress tracker for a URL
    pub fn new(url: String) -> Self {
        Self {
            total_size: None,
            downloaded: 0,
            speed: 0.0,
            eta: None,
            elapsed: Duration::ZERO,
            url,
        }
    }

    /// Calculate percentage (0-100)
    pub fn percentage(&self) -> Option<f64> {
        self.total_size.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.downloaded as f64 / total as f64) * 100.0
            }
        })
    }

    /// Update progress with new downloaded bytes
    pub fn update(&mut self, new_bytes: u64, start_time: Instant) {
        self.downloaded += new_bytes;
        self.elapsed = start_time.elapsed();

        // Calculate speed (bytes per second)
        if self.elapsed.as_secs_f64() > 0.0 {
            self.speed = self.downloaded as f64 / self.elapsed.as_secs_f64();
        }

        // Calculate ETA
        if let Some(total) = self.total_size {
            if self.speed > 0.0 && self.downloaded < total {
                let remaining_bytes = total - self.downloaded;
                let eta_secs = remaining_bytes as f64 / self.speed;
                self.eta = Some(Duration::from_secs_f64(eta_secs));
            }
        }
    }

    /// Format speed in human-readable format (KB/s, MB/s, etc.)
    pub fn format_speed(&self) -> String {
        format_bytes_per_sec(self.speed)
    }

    /// Format downloaded size in human-readable format
    pub fn format_downloaded(&self) -> String {
        format_bytes(self.downloaded)
    }

    /// Format total size in human-readable format
    pub fn format_total(&self) -> Option<String> {
        self.total_size.map(format_bytes)
    }

    /// Format ETA in human-readable format
    pub fn format_eta(&self) -> Option<String> {
        self.eta.map(format_duration)
    }
}

/// Callback function for progress updates
pub type ProgressCallback = Arc<dyn Fn(ProgressInfo) + Send + Sync>;

/// Format bytes in human-readable format (B, KB, MB, GB, etc.)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{}B", bytes)
    } else {
        format!("{:.2}{}", size, UNITS[unit_idx])
    }
}

/// Format bytes per second in human-readable format
pub fn format_bytes_per_sec(bytes_per_sec: f64) -> String {
    const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s"];
    let mut speed = bytes_per_sec;
    let mut unit_idx = 0;

    while speed >= 1024.0 && unit_idx < UNITS.len() - 1 {
        speed /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:.0}{}", speed, UNITS[unit_idx])
    } else {
        format!("{:.2}{}", speed, UNITS[unit_idx])
    }
}

/// Format duration in human-readable format (e.g., "2h 30m 15s")
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        if secs == 0 {
            format!("{}m", mins)
        } else {
            format!("{}m {}s", mins, secs)
        }
    } else {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        if mins == 0 && secs == 0 {
            format!("{}h", hours)
        } else if secs == 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h {}m {}s", hours, mins, secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0B");
        assert_eq!(format_bytes(512), "512B");
        assert_eq!(format_bytes(1024), "1.00KB");
        assert_eq!(format_bytes(1536), "1.50KB");
        assert_eq!(format_bytes(1048576), "1.00MB");
        assert_eq!(format_bytes(1073741824), "1.00GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(60)), "1m");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }
}

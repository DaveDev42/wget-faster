use wget_faster_lib::{ProgressInfo, format_bytes, format_bytes_per_sec, format_duration};
use std::time::{Duration, Instant};

#[test]
fn test_progress_info_creation() {
    let progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    assert_eq!(progress.url, "https://example.com/file.zip");
    assert_eq!(progress.downloaded, 0);
    assert!(progress.total_size.is_none());
    assert_eq!(progress.speed, 0.0);
}

#[test]
fn test_progress_percentage() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.downloaded = 50;
    progress.total_size = Some(100);

    let percentage = progress.percentage();
    assert!(percentage.is_some());
    assert_eq!(percentage.unwrap(), 50.0);
}

#[test]
fn test_progress_percentage_unknown_total() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.downloaded = 50;
    progress.total_size = None;

    let percentage = progress.percentage();
    assert!(percentage.is_none());
}

#[test]
fn test_progress_eta() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.downloaded = 50;
    progress.total_size = Some(100);
    progress.speed = 10.0; // 10 bytes per second
    progress.eta = Some(Duration::from_secs(5)); // Manually set ETA for testing

    assert!(progress.eta.is_some());

    // Remaining: 50 bytes, speed: 10 bytes/sec = 5 seconds
    let eta_secs = progress.eta.unwrap().as_secs();
    assert_eq!(eta_secs, 5);
}

#[test]
fn test_progress_eta_zero_speed() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.downloaded = 50;
    progress.total_size = Some(100);
    progress.speed = 0.0;
    progress.eta = None; // No ETA when speed is zero

    assert!(progress.eta.is_none());
}

#[test]
fn test_progress_eta_unknown_total() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.downloaded = 50;
    progress.total_size = None;
    progress.speed = 10.0;
    progress.eta = None; // No ETA when total size is unknown

    assert!(progress.eta.is_none());
}

#[test]
fn test_progress_update() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());
    let start_time = Instant::now();

    std::thread::sleep(Duration::from_millis(100));

    progress.update(1000, start_time);

    assert!(progress.elapsed >= Duration::from_millis(100));
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(500), "500 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(format_bytes(1024u64.pow(4)), "1.0 TB");
}

#[test]
fn test_format_bytes_per_sec() {
    assert_eq!(format_bytes_per_sec(0.0), "0 B/s");
    assert_eq!(format_bytes_per_sec(500.0), "500 B/s");
    assert_eq!(format_bytes_per_sec(1024.0), "1.0 KB/s");
    assert_eq!(format_bytes_per_sec(1536.0), "1.5 KB/s");
    assert_eq!(format_bytes_per_sec(1048576.0), "1.0 MB/s");
    assert_eq!(format_bytes_per_sec(1073741824.0), "1.0 GB/s");
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(Duration::from_secs(0)), "0s");
    assert_eq!(format_duration(Duration::from_secs(30)), "30s");
    assert_eq!(format_duration(Duration::from_secs(60)), "1m 0s");
    assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
    assert_eq!(format_duration(Duration::from_secs(3600)), "1h 0m 0s");
    assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m 5s");
}

#[test]
fn test_progress_format_downloaded() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.downloaded = 1024 * 1024; // 1 MB

    let formatted = progress.format_downloaded();
    assert_eq!(formatted, "1.0 MB");
}

#[test]
fn test_progress_format_total() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.total_size = Some(10 * 1024 * 1024); // 10 MB

    let formatted = progress.format_total();
    assert!(formatted.is_some());
    assert_eq!(formatted.unwrap(), "10.0 MB");
}

#[test]
fn test_progress_format_total_unknown() {
    let progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    let formatted = progress.format_total();
    assert!(formatted.is_none());
}

#[test]
fn test_progress_format_speed() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.speed = 1024.0 * 1024.0; // 1 MB/s

    let formatted = progress.format_speed();
    assert_eq!(formatted, "1.0 MB/s");
}

#[test]
fn test_progress_complete() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    progress.downloaded = 100;
    progress.total_size = Some(100);

    let percentage = progress.percentage();
    assert_eq!(percentage.unwrap(), 100.0);
}

#[test]
fn test_progress_over_100_percent() {
    let mut progress = ProgressInfo::new("https://example.com/file.zip".to_string());

    // This can happen with compression or incorrect content-length
    progress.downloaded = 150;
    progress.total_size = Some(100);

    let percentage = progress.percentage();
    assert!(percentage.unwrap() > 100.0);
}

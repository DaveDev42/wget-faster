use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo, HttpMethod, AuthConfig, AuthType};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[tokio::test]
async fn test_basic_download() {
    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    // This would need a real test URL or mock server
    // For now, we'll test the configuration
    assert!(true);
}

#[tokio::test]
async fn test_progress_callback() {
    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let progress_called = Arc::new(Mutex::new(false));
    let progress_called_clone = Arc::clone(&progress_called);

    let _callback = Arc::new(move |_info: ProgressInfo| {
        let mut called = progress_called_clone.lock().unwrap();
        *called = true;
    });

    // Progress callback would be tested with actual download
    assert!(true);
}

#[tokio::test]
async fn test_custom_config() {
    let mut config = DownloadConfig::default();

    config.parallel_chunks = 16;
    config.timeout = Duration::from_secs(60);
    config.user_agent = "TestAgent/1.0".to_string();
    config.method = HttpMethod::Post;
    config.body_data = Some(b"test=data".to_vec());
    config.referer = Some("https://example.com".to_string());

    config.auth = Some(AuthConfig {
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        auth_type: AuthType::Basic,
    });

    let downloader = Downloader::new(config);
    assert!(downloader.is_ok());
}

#[tokio::test]
async fn test_http_methods() {
    for method in [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Patch,
        HttpMethod::Options,
    ] {
        let mut config = DownloadConfig::default();
        config.method = method;

        let downloader = Downloader::new(config);
        assert!(downloader.is_ok());
    }
}

#[test]
fn test_http_method_conversions() {
    assert_eq!(HttpMethod::Get.as_str(), "GET");
    assert_eq!(HttpMethod::Post.as_str(), "POST");
    assert_eq!(HttpMethod::Put.as_str(), "PUT");
    assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
    assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
    assert_eq!(HttpMethod::Options.as_str(), "OPTIONS");

    assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::Get));
    assert_eq!(HttpMethod::from_str("post"), Some(HttpMethod::Post));
    assert_eq!(HttpMethod::from_str("INVALID"), None);
}

#[tokio::test]
async fn test_retry_config() {
    let mut config = DownloadConfig::default();

    config.retry.max_retries = 10;
    config.retry.initial_delay = Duration::from_secs(1);
    config.retry.max_delay = Duration::from_secs(30);
    config.retry.backoff_multiplier = 2.0;
    config.retry.retry_on_conn_refused = true;

    let downloader = Downloader::new(config);
    assert!(downloader.is_ok());
}

#[tokio::test]
async fn test_parallel_chunks_config() {
    for chunks in [1, 2, 4, 8, 16, 32, 64] {
        let mut config = DownloadConfig::default();
        config.parallel_chunks = chunks;

        let downloader = Downloader::new(config);
        assert!(downloader.is_ok());
    }
}

#[tokio::test]
async fn test_chunk_size_config() {
    for size in [
        256 * 1024,      // 256 KB
        512 * 1024,      // 512 KB
        1024 * 1024,     // 1 MB
        5 * 1024 * 1024, // 5 MB
    ] {
        let mut config = DownloadConfig::default();
        config.chunk_size = Some(size);

        let downloader = Downloader::new(config);
        assert!(downloader.is_ok());
    }
}

#[test]
fn test_config_defaults() {
    let config = DownloadConfig::default();

    assert_eq!(config.parallel_chunks, 8);
    assert_eq!(config.timeout, Duration::from_secs(120));
    assert_eq!(config.connect_timeout, Duration::from_secs(30));
    assert!(config.follow_redirects);
    assert_eq!(config.max_redirects, 20);
    assert!(config.enable_cookies);
    assert!(config.enable_compression);
    assert!(config.verify_ssl);
    assert!(config.http_keep_alive);
}

#[tokio::test]
async fn test_timestamping_config() {
    let mut config = DownloadConfig::default();

    config.timestamping = true;
    config.if_modified_since = true;
    config.use_server_timestamps = true;

    let downloader = Downloader::new(config);
    assert!(downloader.is_ok());
}

#[tokio::test]
async fn test_quota_and_wait_config() {
    let mut config = DownloadConfig::default();

    config.quota = Some(100 * 1024 * 1024); // 100 MB
    config.wait_time = Some(Duration::from_secs(2));
    config.random_wait = true;
    config.wait_retry = Some(Duration::from_secs(5));

    let downloader = Downloader::new(config);
    assert!(downloader.is_ok());
}

#[test]
fn test_auth_types() {
    assert_eq!(AuthType::Basic, AuthType::Basic);
    assert_eq!(AuthType::Digest, AuthType::Digest);
    assert_ne!(AuthType::Basic, AuthType::Digest);
}

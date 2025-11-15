use mockito::Server;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wget_faster_lib::{
    AuthConfig, AuthType, DownloadConfig, Downloader, HttpClient, HttpMethod, ProgressInfo,
};

#[tokio::test]
async fn test_basic_http_download() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/test.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("Hello, World!")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/test.txt", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes, "Hello, World!");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_download_with_content_length() {
    let mut server = Server::new_async().await;

    let body = "Test content with known length";
    let mock = server
        .mock("GET", "/file.txt")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_header("content-length", &body.len().to_string())
        .with_body(body)
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/file.txt", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes, body);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_progress_callback() {
    let mut server = Server::new_async().await;

    let body = "x".repeat(1000); // 1KB of data
    let mock = server
        .mock("GET", "/progress.txt")
        .with_status(200)
        .with_header("content-length", &body.len().to_string())
        .with_body(&body)
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let progress_called = Arc::new(Mutex::new(false));
    let progress_called_clone = Arc::clone(&progress_called);

    let callback = Arc::new(move |_info: ProgressInfo| {
        let mut called = progress_called_clone.lock().unwrap();
        *called = true;
    });

    let url = format!("{}/progress.txt", server.url());
    let result = downloader
        .download_to_memory_with_progress(&url, Some(callback))
        .await;

    assert!(result.is_ok());
    assert!(*progress_called.lock().unwrap(), "Progress callback should have been called");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_redirect_following() {
    let mut server = Server::new_async().await;

    let redirect_mock = server
        .mock("GET", "/redirect")
        .with_status(302)
        .with_header("location", &format!("{}/final", server.url()))
        .create_async()
        .await;

    let final_mock = server
        .mock("GET", "/final")
        .with_status(200)
        .with_body("Redirected content")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/redirect", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes, "Redirected content");

    redirect_mock.assert_async().await;
    final_mock.assert_async().await;
}

#[tokio::test]
async fn test_404_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/notfound")
        .with_status(404)
        .with_body("Not Found")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/notfound", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_custom_headers() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/custom-headers")
        .match_header("X-Custom-Header", "test-value")
        .with_status(200)
        .with_body("OK")
        .create_async()
        .await;

    let mut config = DownloadConfig::default();
    config
        .headers
        .insert("X-Custom-Header".to_string(), "test-value".to_string());

    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/custom-headers", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_ok());

    mock.assert_async().await;
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

#[tokio::test]
async fn test_post_request() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/data")
        .match_body("key=value&foo=bar")
        .with_status(200)
        .with_body("Success")
        .create_async()
        .await;

    let mut config = DownloadConfig::default();
    config.method = HttpMethod::Post;
    config.body_data = Some(b"key=value&foo=bar".to_vec());

    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/api/data", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_download_to_file() {
    let mut server = Server::new_async().await;

    let body = "File content to download";

    // Add HEAD request mock (downloader checks metadata first)
    let head_mock = server
        .mock("HEAD", "/downloadable.txt")
        .with_status(200)
        .with_header("content-length", &body.len().to_string())
        .create_async()
        .await;

    let mock = server
        .mock("GET", "/downloadable.txt")
        .with_status(200)
        .with_header("content-length", &body.len().to_string())
        .with_body(body)
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    // Create temp file
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");

    let url = format!("{}/downloadable.txt", server.url());
    let result = downloader.download_to_file(&url, file_path.clone()).await;

    if let Err(e) = &result {
        eprintln!("Download to file failed: {e:?}");
    }
    assert!(result.is_ok(), "Download failed: {:?}", result.err());
    assert!(file_path.exists());

    // Verify file contents
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, body);

    head_mock.assert_async().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn test_range_request_support() {
    let mut server = Server::new_async().await;

    // HEAD request to check capabilities
    let head_mock = server
        .mock("HEAD", "/large-file.bin")
        .with_status(200)
        .with_header("accept-ranges", "bytes")
        .with_header("content-length", "1000")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/large-file.bin", server.url());

    // Check if server supports range
    let client = HttpClient::new(DownloadConfig::default()).unwrap();
    let supports_range = client.supports_range(&url).await;

    assert!(supports_range.is_ok());
    assert!(supports_range.unwrap());

    head_mock.assert_async().await;
}

#[tokio::test]
async fn test_user_agent() {
    let mut server = Server::new_async().await;

    let custom_ua = "WgetFaster/0.1.0 Test";
    let mock = server
        .mock("GET", "/check-ua")
        .match_header("user-agent", custom_ua)
        .with_status(200)
        .with_body("OK")
        .create_async()
        .await;

    let mut config = DownloadConfig::default();
    config.user_agent = custom_ua.to_string();

    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/check-ua", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_ok());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_500_server_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/server-error")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/server-error", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_server_response_display() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("HEAD", "/file")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_header("content-length", "100")
        .with_header("accept-ranges", "bytes")
        .with_header("last-modified", "Wed, 21 Oct 2015 07:28:00 GMT")
        .create_async()
        .await;

    let mock_get = server
        .mock("GET", "/file")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("test content for server response")
        .create_async()
        .await;

    let config = DownloadConfig {
        print_server_response: true,
        ..Default::default()
    };

    let downloader = Downloader::new(config).unwrap();
    let url = format!("{}/file", server.url());

    // This will print headers to stderr
    let result = downloader.download_to_memory(&url).await;

    mock.assert_async().await;
    mock_get.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_metadata_contains_headers() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("HEAD", "/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("content-length", "42")
        .with_header("x-custom-header", "custom-value")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let client = crate::HttpClient::new(config).unwrap();
    let url = format!("{}/test", server.url());

    let metadata = client.get_metadata(&url).await.unwrap();

    // Verify headers are captured
    assert_eq!(metadata.status_code, 200);
    assert!(metadata.headers.contains_key("content-type"));
    assert!(metadata.headers.contains_key("content-length"));
    assert!(metadata.headers.contains_key("x-custom-header"));

    // Test format_headers method
    let formatted = metadata.format_headers();
    assert!(formatted.contains("HTTP/1.1 200 OK"));
    assert!(formatted.contains("content-type"));
    assert!(formatted.contains("application/json"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_speed_limiting() {
    let mut server = Server::new_async().await;

    // 100KB of data
    let data_size = 100 * 1024;
    let data = vec![0u8; data_size];

    let mock = server
        .mock("GET", "/large-file")
        .with_status(200)
        .with_header("content-length", &data_size.to_string())
        .with_body(&data)
        .create_async()
        .await;

    // Limit to 50KB/s
    let speed_limit = 50 * 1024;
    let config = DownloadConfig {
        speed_limit: Some(speed_limit),
        ..Default::default()
    };

    let downloader = Downloader::new(config).unwrap();
    let url = format!("{}/large-file", server.url());

    let start = std::time::Instant::now();
    let result = downloader.download_to_memory(&url).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes.len(), data_size);

    // Should take at least 2 seconds (100KB at 50KB/s)
    // Allow some margin for overhead
    assert!(duration.as_secs_f64() >= 1.8, "Download was too fast: {duration:?}");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_no_speed_limit() {
    let mut server = Server::new_async().await;

    let data_size = 50 * 1024; // 50KB
    let data = vec![0u8; data_size];

    let mock = server
        .mock("GET", "/file")
        .with_status(200)
        .with_body(&data)
        .create_async()
        .await;

    let config = DownloadConfig {
        speed_limit: None,
        ..Default::default()
    };

    let downloader = Downloader::new(config).unwrap();
    let url = format!("{}/file", server.url());

    let start = std::time::Instant::now();
    let result = downloader.download_to_memory(&url).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    // Without speed limit, should be much faster (< 1 second for local mock)
    assert!(duration.as_secs_f64() < 1.0);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_if_modified_since_header() {
    use std::time::SystemTime;

    let mut server = Server::new_async().await;

    // Create a time in the past
    let past_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1420070400); // Jan 1, 2015
    let http_date = httpdate::fmt_http_date(past_time);

    // Mock HEAD request that should receive If-Modified-Since header
    let head_mock = server
        .mock("HEAD", "/timestamped-file.txt")
        .match_header("If-Modified-Since", http_date.as_str())
        .with_status(304) // Not Modified
        .with_header("Last-Modified", &http_date)
        .create_async()
        .await;

    // Create a temporary file with the past modification time
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("timestamped-file.txt");
    std::fs::write(&file_path, "old content").unwrap();

    // Set file mtime to the past time
    let file_time = filetime::FileTime::from_system_time(past_time);
    filetime::set_file_mtime(&file_path, file_time).unwrap();

    // Configure downloader with timestamping enabled
    let config = DownloadConfig {
        timestamping: true,
        use_server_timestamps: true,
        ..Default::default()
    };
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/timestamped-file.txt", server.url());
    let result = downloader.download_to_file(&url, file_path.clone()).await;

    // Should succeed and not download (304 Not Modified)
    assert!(result.is_ok());

    // File should still contain old content (not re-downloaded)
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "old content");

    head_mock.assert_async().await;
}

#[tokio::test]
async fn test_if_modified_since_with_newer_remote() {
    use std::time::SystemTime;

    let mut server = Server::new_async().await;

    // Local file time (old)
    let old_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1420070400); // Jan 1, 2015
    let old_http_date = httpdate::fmt_http_date(old_time);

    // Remote file time (newer)
    let new_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1483228800); // Jan 1, 2017
    let new_http_date = httpdate::fmt_http_date(new_time);

    // Mock HEAD request
    let head_mock = server
        .mock("HEAD", "/updated-file.txt")
        .match_header("If-Modified-Since", old_http_date.as_str())
        .with_status(200) // Modified, proceed with download
        .with_header("Last-Modified", &new_http_date)
        .with_header("Content-Length", "11")
        .create_async()
        .await;

    // Mock GET request for the actual download
    let get_mock = server
        .mock("GET", "/updated-file.txt")
        .with_status(200)
        .with_header("Last-Modified", &new_http_date)
        .with_body("new content")
        .create_async()
        .await;

    // Create a temporary file with old modification time
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("updated-file.txt");
    std::fs::write(&file_path, "old content").unwrap();

    // Set file mtime to old time
    let file_time = filetime::FileTime::from_system_time(old_time);
    filetime::set_file_mtime(&file_path, file_time).unwrap();

    // Configure downloader with timestamping
    let config = DownloadConfig {
        timestamping: true,
        use_server_timestamps: true,
        ..Default::default()
    };
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/updated-file.txt", server.url());
    let result = downloader.download_to_file(&url, file_path.clone()).await;

    // Should succeed and re-download
    assert!(result.is_ok());

    // File should now contain new content
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "new content");

    head_mock.assert_async().await;
    get_mock.assert_async().await;
}

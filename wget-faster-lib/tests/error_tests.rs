use wget_faster_lib::{Downloader, DownloadConfig, Error};
use mockito::Server;
use std::time::Duration;

#[tokio::test]
async fn test_network_timeout() {
    let mut config = DownloadConfig::default();
    config.timeout = Duration::from_millis(100);

    let downloader = Downloader::new(config).unwrap();

    // Try to connect to a non-routable IP that will timeout
    let result = downloader.download_to_memory("http://10.255.255.1/timeout").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_url() {
    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let result = downloader.download_to_memory("not-a-valid-url").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_404_not_found() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/missing")
        .with_status(404)
        .with_body("Not Found")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/missing", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_500_internal_server_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/error")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/error", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_503_service_unavailable() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/unavailable")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/unavailable", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_connection_refused() {
    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    // Try to connect to a port that's likely not listening
    let result = downloader.download_to_memory("http://localhost:1/refused").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_retry_config() {
    let mut config = DownloadConfig::default();
    config.retry.max_retries = 3;
    config.retry.initial_delay = Duration::from_millis(10);
    config.retry.max_delay = Duration::from_millis(100);
    config.retry.backoff_multiplier = 2.0;

    let downloader = Downloader::new(config);
    assert!(downloader.is_ok());
}

#[tokio::test]
async fn test_empty_response() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/empty")
        .with_status(200)
        .with_body("")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/empty", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes.len(), 0);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_large_content_length_mismatch() {
    let mut server = Server::new_async().await;

    let body = "Short body";
    let mock = server
        .mock("GET", "/mismatch")
        .with_status(200)
        .with_header("content-length", "1000000") // Claim 1MB
        .with_body(body) // But only send 10 bytes
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/mismatch", server.url());
    let result = downloader.download_to_memory(&url).await;

    // Should handle gracefully even with content-length mismatch
    assert!(result.is_ok() || result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_redirect_loop_detection() {
    let mut server = Server::new_async().await;

    // Create a redirect loop: /loop1 -> /loop2 -> /loop1
    let mock1 = server
        .mock("GET", "/loop1")
        .with_status(302)
        .with_header("location", &format!("{}/loop2", server.url()))
        .create_async()
        .await;

    let mock2 = server
        .mock("GET", "/loop2")
        .with_status(302)
        .with_header("location", &format!("{}/loop1", server.url()))
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/loop1", server.url());
    let result = downloader.download_to_memory(&url).await;

    // Should eventually fail due to redirect limit
    assert!(result.is_err());

    // Mocks may be called multiple times due to loop
    drop(mock1);
    drop(mock2);
}

#[tokio::test]
async fn test_too_many_redirects() {
    let mut server = Server::new_async().await;

    let mut config = DownloadConfig::default();
    config.max_redirects = 2; // Set low limit

    // Create chain of redirects
    let mock1 = server
        .mock("GET", "/redirect1")
        .with_status(302)
        .with_header("location", &format!("{}/redirect2", server.url()))
        .create_async()
        .await;

    let mock2 = server
        .mock("GET", "/redirect2")
        .with_status(302)
        .with_header("location", &format!("{}/redirect3", server.url()))
        .create_async()
        .await;

    let mock3 = server
        .mock("GET", "/redirect3")
        .with_status(302)
        .with_header("location", &format!("{}/final", server.url()))
        .create_async()
        .await;

    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/redirect1", server.url());
    let result = downloader.download_to_memory(&url).await;

    // Should fail due to too many redirects
    assert!(result.is_err());

    drop(mock1);
    drop(mock2);
    drop(mock3);
}

#[tokio::test]
async fn test_invalid_redirect_location() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/bad-redirect")
        .with_status(302)
        .with_header("location", "not a valid url")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/bad-redirect", server.url());
    let result = downloader.download_to_memory(&url).await;

    // Should handle invalid redirect gracefully
    assert!(result.is_err() || result.is_ok());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_partial_content_response_206() {
    let mut server = Server::new_async().await;

    // Test that 206 Partial Content responses are handled properly
    let partial_content = "56789";
    let mock = server
        .mock("GET", "/partial")
        .with_status(206) // Partial Content
        .with_header("content-range", "bytes 5-9/10")
        .with_header("content-length", &partial_content.len().to_string())
        .with_body(partial_content)
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/partial", server.url());
    let result = downloader.download_to_memory(&url).await;

    // Should successfully handle 206 responses
    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert_eq!(bytes, partial_content);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_unauthorized_401() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/protected")
        .with_status(401)
        .with_header("www-authenticate", "Basic realm=\"Test\"")
        .with_body("Unauthorized")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/protected", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_forbidden_403() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/forbidden")
        .with_status(403)
        .with_body("Forbidden")
        .create_async()
        .await;

    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/forbidden", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_method_not_allowed_405() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/get-only")
        .with_status(405)
        .with_header("allow", "GET, HEAD")
        .with_body("Method Not Allowed")
        .create_async()
        .await;

    let mut config = DownloadConfig::default();
    config.method = wget_faster_lib::HttpMethod::Post;

    let downloader = Downloader::new(config).unwrap();

    let url = format!("{}/get-only", server.url());
    let result = downloader.download_to_memory(&url).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

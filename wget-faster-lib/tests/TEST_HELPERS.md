# Test Helper Patterns

This document describes common patterns and helper functions used across tests.

## Mock Server Setup

### Basic Download Mock (HEAD + GET)

Most download tests require both HEAD and GET mocks because the downloader checks metadata first:

```rust
let mut server = Server::new_async().await;
let body = "Test content";

// HEAD request mock for metadata
let head_mock = server
    .mock("HEAD", "/file.txt")
    .with_status(200)
    .with_header("content-length", &body.len().to_string())
    .with_header("content-type", "text/plain")
    .create_async()
    .await;

// GET request mock for download
let get_mock = server
    .mock("GET", "/file.txt")
    .with_status(200)
    .with_header("content-type", "text/plain")
    .with_body(body)
    .create_async()
    .await;
```

### HTML Page Mock

For recursive download tests:

```rust
let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <a href="/page2.html">Link</a>
    </body>
    </html>
"#;

let head_mock = server
    .mock("HEAD", "/")
    .with_status(200)
    .with_header("content-type", "text/html")
    .with_header("content-length", &html.len().to_string())
    .create_async()
    .await;

let get_mock = server
    .mock("GET", "/")
    .with_status(200)
    .with_header("content-type", "text/html")
    .with_body(html)
    .create_async()
    .await;
```

## File Assertions

### Check File Exists and Content

```rust
use std::fs;

// Assert file exists
assert!(file_path.exists(), "File should exist at {:?}", file_path);

// Assert file content
let content = fs::read_to_string(&file_path)
    .expect("Failed to read file");
assert_eq!(content, expected);

// Assert file size
let metadata = fs::metadata(&file_path)
    .expect("Failed to get metadata");
assert_eq!(metadata.len(), expected_size);
```

### Temporary Directories

Always use `tempfile` for test output:

```rust
use tempfile::TempDir;

let temp_dir = TempDir::new().unwrap();
let file_path = temp_dir.path().join("output.txt");

// File will be automatically cleaned up when temp_dir is dropped
```

## Progress Tracking

### Progress Callback

```rust
use std::sync::{Arc, Mutex};

let progress_called = Arc::new(Mutex::new(false));
let progress_called_clone = Arc::clone(&progress_called);

let callback = Arc::new(move |info: ProgressInfo| {
    let mut called = progress_called_clone.lock().unwrap();
    *called = true;

    // Optional: check progress values
    assert!(info.downloaded > 0);
});

downloader.download_to_memory_with_progress(&url, Some(callback)).await?;

assert!(*progress_called.lock().unwrap(), "Progress callback should be called");
```

## Common Test Patterns

### Error Testing

```rust
let result = downloader.download_to_memory(&url).await;

assert!(result.is_err());

// Check specific error type
match result {
    Err(Error::InvalidStatus(404)) => {}, // Expected
    other => panic!("Expected 404, got: {:?}", other),
}
```

### Timestamping Tests

```rust
use std::time::SystemTime;
use filetime::{FileTime, set_file_mtime};

// Create file with specific mtime
let past_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1420070400);
let file_time = FileTime::from_system_time(past_time);
set_file_mtime(&file_path, file_time).unwrap();

// Configure timestamping
let config = DownloadConfig {
    timestamping: true,
    use_server_timestamps: true,
    ..Default::default()
};
```

### Authentication Tests

```rust
// Mock with auth challenge
let mock = server
    .mock("GET", "/protected")
    .match_header("authorization", "Basic dXNlcjpwYXNz")  // user:pass in base64
    .with_status(200)
    .with_body("Protected content")
    .create_async()
    .await;

// Configure auth
let config = DownloadConfig {
    auth: Some(AuthConfig {
        username: "user".to_string(),
        password: "pass".to_string(),
        auth_type: AuthType::Basic,
    }),
    ..Default::default()
};
```

## Best Practices

1. **Always use HEAD + GET mocks** - The downloader checks metadata first
2. **Use tempfile::TempDir** - Automatic cleanup
3. **Be specific with assertions** - Include descriptive messages
4. **Test error cases** - Not just success paths
5. **Use expect_at_least(0)** - For optional requests (like recursive downloads)
6. **Check mock assertions** - Always call `mock.assert_async().await`

## Example: Complete Download Test

```rust
#[tokio::test]
async fn test_complete_download() {
    // 1. Setup server
    let mut server = Server::new_async().await;
    let body = "Test content";

    // 2. Create mocks (HEAD + GET)
    let head_mock = server
        .mock("HEAD", "/file.txt")
        .with_status(200)
        .with_header("content-length", &body.len().to_string())
        .create_async()
        .await;

    let get_mock = server
        .mock("GET", "/file.txt")
        .with_status(200)
        .with_body(body)
        .create_async()
        .await;

    // 3. Create downloader
    let config = DownloadConfig::default();
    let downloader = Downloader::new(config).unwrap();

    // 4. Create temp output
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("file.txt");

    // 5. Download
    let url = format!("{}/file.txt", server.url());
    let result = downloader.download_to_file(&url, file_path.clone()).await;

    // 6. Assert success
    assert!(result.is_ok());
    assert!(file_path.exists());

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, body);

    // 7. Verify mocks called
    head_mock.assert_async().await;
    get_mock.assert_async().await;
}
```

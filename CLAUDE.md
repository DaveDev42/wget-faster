# CLAUDE.md - AI/LLM Context for wget-faster

This document provides implementation details optimized for AI assistants (like Claude) to understand and work with the wget-faster codebase.

## Project Overview

**wget-faster** is a high-performance HTTP downloader written in Rust that aims to be a drop-in replacement for GNU wget. The project is in active development with core library functionality complete and CLI implementation in progress.

## Architecture

### Core Components

1. **wget-faster-lib** - Core async library
   - `downloader.rs` - Main downloader engine with parallel support
   - `client.rs` - HTTP client wrapper (reqwest-based)
   - `parallel.rs` - Range request based parallel chunk downloads
   - `progress.rs` - Real-time progress tracking with speed/ETA
   - `config.rs` - Configuration types (DownloadConfig, AuthConfig, etc.)
   - `output.rs` - Output modes (Memory/File)
   - `error.rs` - Error types (13 variants)

2. **wget-faster-cli** - Command-line interface
   - `main.rs` - Entry point and main logic
   - `args.rs` - Argument parsing (150+ wget options via clap)
   - `output.rs` - wget-style output formatting

### Key Design Patterns

**Async/Non-blocking**:
- All I/O operations use tokio
- Main API: `async fn download_to_memory(url) -> Result<Bytes>`
- Progress callbacks: `Arc<dyn Fn(ProgressInfo) + Send + Sync>`

**Parallel Downloads**:
- Automatic for files >10MB (configurable threshold)
- HTTP Range requests to split into chunks
- Default: 8 parallel connections (configurable)
- Chunk size auto-calculated: `total_size / parallel_chunks`
- Individual chunk retry on failure

**Output Modes**:
```rust
pub enum OutputMode {
    Memory,                    // Returns Bytes
    File(PathBuf),            // Writes to file
}
```

**Configuration**:
```rust
pub struct DownloadConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub user_agent: Option<String>,
    pub headers: HashMap<String, String>,
    pub auth: Option<AuthConfig>,
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub verify_ssl: bool,
    pub cookies: Option<CookieJar>,
    pub speed_limit: Option<u64>,
    pub retry_attempts: usize,
    pub retry_delay: Duration,
    pub parallel_chunks: usize,
    pub parallel_threshold: u64,
}
```

## Current Implementation Status

### ✅ Implemented Features

**Library (wget-faster-lib)**:
- Fully async API (tokio-based)
- Multiple output modes (memory/file)
- HTTP Range request parallel downloads
- Progress tracking with callbacks
- Resume support for partial downloads
- Retry logic with exponential backoff
- Cookie support
- HTTP authentication (Basic/Digest)
- Proxy support
- Custom headers
- SSL/TLS with certificate verification
- Redirect following
- Content compression (gzip, deflate, brotli)
- Speed limiting
- Configurable timeouts

**CLI (wget-faster-cli)**:
- 150+ option parsing (clap-based)
- Basic download functionality
- wget-style output formatting
- Progress display
- File output (-O)
- Resume support (-c)
- Authentication options
- SSL/TLS options
- HTTP header options

### ⚠️ Partially Implemented

- Recursive downloads (args parsed, execution incomplete)
- POST requests (args parsed, execution incomplete)
- Cookie file I/O (structure ready, file format incomplete)

### 📋 Planned Features

- FTP/FTPS support
- HTML parsing for recursive downloads
- robots.txt compliance
- WARC archiving
- Metalink support

## Testing Strategy

### License Separation

**Important**: wget-faster uses a two-repository strategy for testing:

1. **wget-faster** (this repo): BSD-3-Clause
   - Core library and CLI
   - Independent implementation
   - No GPL code

2. **wget-faster-test** (separate repo): GPL-3.0
   - wget test suite integration
   - wget as git submodule
   - Test runner scripts

**Why**: To maintain complete license separation while enabling wget compatibility testing. We test the compiled binary against wget's test suite, similar to how browsers are tested against W3C test suites.

### Testing Approach

1. **External testing**: Run wget's ~100 Python tests against wget-faster binary via `WGET_PATH` environment variable
2. **Independent tests**: Our own test suite in `wget-faster-lib/tests/`
3. **No code copying**: Never copy GPL test code into our repository
4. **Reference only**: wget tests verify compatibility, not part of codebase

## Code Examples

### Basic Download (Library)

```rust
use wget_faster_lib::{Downloader, DownloadConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::new(DownloadConfig::default())?;
    let bytes = downloader
        .download_to_memory("https://example.com/file.txt")
        .await?;
    println!("Downloaded {} bytes", bytes.len());
    Ok(())
}
```

### Download with Progress

```rust
use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo};
use std::sync::Arc;

let progress_callback = Arc::new(|progress: ProgressInfo| {
    println!(
        "Downloaded: {} / {} ({:.1}%) - Speed: {}",
        progress.format_downloaded(),
        progress.format_total().unwrap_or_else(|| "Unknown".to_string()),
        progress.percentage().unwrap_or(0.0),
        progress.format_speed()
    );
});

downloader
    .download_to_file_with_progress(
        "https://example.com/file.zip",
        "output.zip".into(),
        Some(progress_callback),
    )
    .await?;
```

### Custom Configuration

```rust
let mut config = DownloadConfig::default();
config.parallel_chunks = 16;
config.timeout = Duration::from_secs(60);
config.auth = Some(AuthConfig {
    username: "user".to_string(),
    password: "pass".to_string(),
    auth_type: AuthType::Basic,
});
config.headers.insert("X-API-Key".to_string(), "secret".to_string());
config.speed_limit = Some(1024 * 1024); // 1 MB/s

let downloader = Downloader::new(config)?;
```

## Common Tasks

### Adding a New wget Option

1. Add to `args.rs` in the appropriate clap group
2. Add to `WgetArgs` struct
3. Map to `DownloadConfig` in `main.rs`
4. If needed, extend `DownloadConfig` in `lib/src/config.rs`
5. Implement in `lib/src/downloader.rs`

### Implementing a New Feature

1. Define error types in `lib/src/error.rs` if needed
2. Add configuration to `config.rs`
3. Implement in appropriate module (`downloader.rs`, `client.rs`, etc.)
4. Add CLI argument in `cli/src/args.rs`
5. Wire up in `cli/src/main.rs`
6. Add tests

### Debugging Parallel Downloads

- Parallel downloads triggered when: `content_length >= config.parallel_threshold` (default: 10MB)
- Check server support: Response must include `Accept-Ranges: bytes` header
- Chunk calculation: `chunk_size = total_size / parallel_chunks`
- Monitor: Each chunk download logged separately

## Performance Characteristics

**Parallel Downloads**:
- 8x faster for large files (>10MB) on fast connections
- Automatically disabled if server doesn't support Range requests
- Memory usage: ~8MB for 8 parallel chunks (1MB buffer per chunk)

**Async I/O**:
- Non-blocking throughout
- Efficient CPU and network utilization
- Can handle multiple concurrent downloads

**Memory Efficiency**:
- Streaming downloads with fixed buffer sizes
- Large files don't load entirely into memory
- Parallel chunks use separate fixed-size buffers

## CLI Compatibility with GNU wget

### Target

100% compatibility with GNU wget's 150+ options

### Major Option Categories

- **Startup**: `-V`, `-h`, `-b`, `-e`
- **Logging**: `-o`, `-a`, `-d`, `-q`, `-v`, `-nv`
- **Download**: `-t`, `-O`, `-nc`, `-c`, `-N`, `-S`, `-T`, `-w`, `--limit-rate`
- **Directories**: `-nd`, `-x`, `-nH`, `-P`, `--cut-dirs`
- **HTTP**: `--http-user`, `--header`, `--post-data`, `--user-agent`, `--cookies`
- **HTTPS/TLS**: `--no-check-certificate`, `--certificate`, `--ca-certificate`
- **FTP**: `--ftp-user`, `--ftp-password`, `--no-glob` (planned)
- **Recursive**: `-r`, `-l`, `-k`, `-p`, `-A`, `-R`, `-D`

### Output Format Compatibility

wget-faster mimics wget's output format:
```
--2024-01-15 10:30:00--  https://example.com/file.zip
Resolving example.com (example.com)... 93.184.216.34
Connecting to example.com (example.com)|93.184.216.34|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 1048576 (1.0M) [application/zip]
Saving to: 'file.zip'

file.zip         100%[===============>]   1.00M  1.50MB/s    in 0.7s

2024-01-15 10:30:01 (1.50 MB/s) - 'file.zip' saved [1048576/1048576]
```

## Development Notes

### Tech Stack

- **Language**: Rust (edition 2021)
- **Async Runtime**: tokio 1.42
- **HTTP Client**: reqwest 0.12
- **CLI Parsing**: clap 4.5
- **Progress Bars**: indicatif 0.17
- **Error Handling**: thiserror, anyhow

### Code Style

- Use `async fn` for all I/O operations
- Prefer `Result<T, DownloadError>` over unwrap/expect
- Document public APIs with rustdoc comments
- Follow Rust naming conventions (snake_case for functions/variables)
- Use descriptive variable names

### Error Handling

Error types in `lib/src/error.rs`:
- `NetworkError` - Connection/network issues
- `HttpError` - HTTP protocol errors
- `InvalidUrl` - URL parsing failures
- `FileSystemError` - File I/O errors
- `AuthenticationError` - Auth failures
- `SslError` - TLS/SSL issues
- etc. (13 total variants)

### Next Steps for Development

1. **Complete CLI implementation**
   - Wire up all parsed options to library calls
   - Implement POST request handling
   - Complete cookie file I/O

2. **Improve wget test suite compatibility**
   - Set up wget-faster-test repository
   - Run wget tests and analyze failures
   - Fix compatibility issues

3. **Add remaining features**
   - FTP support
   - Recursive downloads with HTML parsing
   - robots.txt compliance

4. **Performance optimization**
   - Benchmark against GNU wget
   - Tune parallel download parameters
   - Profile and optimize hot paths

## License

**BSD-3-Clause** - This project is completely independent of GNU wget. No GPL code is included or copied.

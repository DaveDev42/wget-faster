# wget-faster

[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)

A high-performance, drop-in replacement for GNU wget written in Rust with async support and parallel downloads.

## Features

### Performance
- **Fully async/non-blocking** - Built on tokio for efficient I/O and minimal overhead
- **Parallel downloads** - Automatic HTTP Range request-based parallel chunk downloads (4-32 connections)
- **Adaptive chunk sizing** - Automatically adjusts chunk size and connection count based on network performance
- **HTTP/2 support** - Connection multiplexing for faster small file downloads
- **Smart connection pooling** - Reuses connections across multiple downloads
- **Memory efficient** - Streaming downloads with constant ~10MB memory usage regardless of file size

### Functionality
- **Progress tracking** - Real-time download speed, progress percentage, and ETA
- **Resume support** - Continue interrupted downloads with `-c`
- **Recursive downloads** - Download entire websites with `-r` (HTML parsing, page requisites)
- **Timestamping** - Only download if remote file is newer (`-N`)
- **HTTP/HTTPS** - Full support including authentication (Basic/Digest), cookies, custom headers
- **POST requests** - Send data with `--post-data` or `--post-file`
- **SSL/TLS** - Certificate verification, custom CA certificates, client certificates
- **Input files** - Read URLs from file or parse HTML for links (`-i, -F`)
- **Spider mode** - Check URLs without downloading (`--spider`)
- **Download quota** - Limit total downloaded bytes (`-Q`)
- **Rate limiting** - Control download speed (`--limit-rate`)
- **Configurable** - Timeouts, retry logic with exponential backoff, wait times

## Quick Start

### Installation

#### From crates.io (Recommended)

```bash
# Install the CLI tool
cargo install wget-faster-cli

# Or add the library to your project
cargo add wget-faster-lib
```

#### From source

```bash
# Clone the repository
git clone https://github.com/wget-faster/wget-faster.git
cd wget-faster

# Build release binary
cargo build --release

# Install to ~/.cargo/bin
cargo install --path wget-faster-cli
```

#### Pre-built binaries

Download pre-built binaries from [GitHub Releases](https://github.com/wget-faster/wget-faster/releases)

### Basic Usage

```bash
# Simple download
wget-faster https://example.com/file.zip

# Download to specific file
wget-faster -O output.zip https://example.com/file.zip

# Resume interrupted download
wget-faster -c https://example.com/large-file.iso

# With authentication
wget-faster --http-user=username --http-password=password https://example.com/protected/file.zip

# Limit download speed to 1 MB/s
wget-faster --limit-rate=1m https://example.com/large-file.iso

# Recursive download (mirror a website)
wget-faster -r -l 3 -p -k https://example.com/

# Download only if remote is newer (timestamping)
wget-faster -N https://example.com/file.zip

# POST request with data
wget-faster --post-data="user=admin&pass=secret" https://example.com/api

# Download multiple URLs from a file
wget-faster -i urls.txt

# Spider mode (check without downloading)
wget-faster --spider https://example.com/file.zip

# With custom headers
wget-faster --header="Authorization: Bearer token123" https://api.example.com/data

# Download with quota limit (stop after 100MB)
wget-faster -Q 100m -i urls.txt

# Random wait between downloads (0.5-1.5 seconds)
wget-faster -w 1 --random-wait -i urls.txt
```

### Library Usage

#### Basic Download

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

#### Download with Progress Tracking

```rust
use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::new(DownloadConfig::default())?;

    let progress = Arc::new(|info: ProgressInfo| {
        if let Some(pct) = info.percentage() {
            println!(
                "{:.1}% - {} - {} - ETA: {}",
                pct,
                info.format_downloaded(),
                info.format_speed(),
                info.eta()
                    .map(|d| format!("{:.0}s", d.as_secs_f64()))
                    .unwrap_or_else(|| "?".to_string())
            );
        }
    });

    downloader
        .download_to_file_with_progress(
            "https://example.com/large-file.zip",
            "output.zip".into(),
            Some(progress),
        )
        .await?;

    Ok(())
}
```

#### Recursive Download

```rust
use wget_faster_lib::{DownloadConfig, RecursiveDownloader, RecursiveConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let download_config = DownloadConfig::default();
    let mut recursive_config = RecursiveConfig::default();

    recursive_config.max_depth = 3;
    recursive_config.page_requisites = true;
    recursive_config.span_hosts = false;

    let mut downloader = RecursiveDownloader::new(download_config, recursive_config)?;

    let files = downloader
        .download_recursive("https://example.com/", "./download")
        .await?;

    println!("Downloaded {} files", files.len());
    Ok(())
}
```

#### Advanced Configuration

```rust
use wget_faster_lib::{Downloader, DownloadConfig, AuthConfig, AuthType, HttpMethod};
use std::time::Duration;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = DownloadConfig::default();

    // Performance settings
    config.parallel_chunks = 16;
    config.chunk_size = Some(1024 * 1024); // 1 MB chunks
    config.timeout = Duration::from_secs(300);

    // Authentication
    config.auth = Some(AuthConfig {
        username: "user".to_string(),
        password: "pass".to_string(),
        auth_type: AuthType::Basic,
    });

    // Custom headers
    config.headers.insert(
        "User-Agent".to_string(),
        "MyApp/1.0".to_string(),
    );
    config.referer = Some("https://example.com".to_string());

    // POST request
    config.method = HttpMethod::Post;
    config.body_data = Some(b"key=value".to_vec());

    // Retry configuration
    config.retry.max_retries = 5;
    config.retry.initial_delay = Duration::from_secs(2);

    let downloader = Downloader::new(config)?;

    let result = downloader
        .download_to_file("https://api.example.com/data", "output.json".into())
        .await?;

    println!("Downloaded {} bytes", result.data.total_bytes);
    Ok(())
}
```

## Project Structure

```
wget-faster/
├── Cargo.toml                 # Workspace configuration
├── wget-faster-lib/          # Core library
│   └── src/
│       ├── lib.rs           # Public API
│       ├── downloader.rs    # Main downloader
│       ├── client.rs        # HTTP client wrapper
│       ├── parallel.rs      # Parallel Range downloads
│       ├── progress.rs      # Progress tracking
│       ├── config.rs        # Configuration types
│       ├── output.rs        # Output modes
│       └── error.rs         # Error types
└── wget-faster-cli/          # CLI application
    └── src/
        ├── main.rs          # Entry point
        ├── args.rs          # Argument parsing (150+ options)
        └── output.rs        # wget-style output formatting
```

## Comparison with GNU wget

| Feature | GNU wget | wget-faster |
|---------|----------|-------------|
| Language | C | Rust |
| Async/Non-blocking | No | Yes (tokio) |
| Parallel Downloads | No | Yes (Range requests) |
| Memory Safety | Manual | Guaranteed by Rust |
| Performance | Good | Better (parallel + async) |
| API Library | Limited | Full async Rust API |

## Development Status

### ✅ Completed (v0.1.0)
- [x] Core library with async support (tokio-based)
- [x] Parallel downloads via HTTP Range requests
- [x] Adaptive chunk sizing - automatically adjusts based on network performance
- [x] Progress tracking with real-time speed and ETA
- [x] Resume support (`-c, --continue`)
- [x] HTTP/HTTPS with authentication (Basic/Digest)
- [x] SSL/TLS configuration (custom CA certs, client certs)
- [x] Cookie support (in-memory and Netscape file format)
- [x] POST requests (`--post-data`, `--post-file`)
- [x] Custom HTTP methods and headers
- [x] Referer support
- [x] Proxy with authentication
- [x] Timestamping (`-N`) - only download if remote is newer
- [x] Input file handling (`-i, -F`) - read URLs from file or HTML
- [x] Wait/retry options (`-w`, `--waitretry`, `--random-wait`)
- [x] Download quota (`-Q`)
- [x] Spider mode (`--spider`) - check if files exist without downloading
- [x] Content-Disposition header support
- [x] Recursive downloads with HTML parsing (`-r, --recursive`)
- [x] Page requisites (`-p`) - download images, CSS, JS
- [x] 150+ CLI options parsed (wget-compatible)
- [x] Performance benchmarks framework

### 🚧 In Progress
- [ ] Link conversion (`-k, --convert-links`)
- [ ] Complete wget test suite integration
- [ ] Unit tests (target: 60%+ coverage)
- [ ] HTTP/3 (QUIC) support for additional performance gains

### 📋 Planned
- [ ] FTP/FTPS support
- [ ] WARC format support
- [ ] Metalink support
- [ ] robots.txt compliance
- [ ] .netrc file support

## Documentation

- [CLAUDE.md](CLAUDE.md) - Implementation details for AI/LLM context
- [SPEC.md](SPEC.md) - Technical specifications and architecture
- [TODO.md](TODO.md) - Pending tasks and future improvements

## License

This project is licensed under the **BSD-3-Clause License** - see the [LICENSE](LICENSE) file for details.

### License Independence

wget-faster is completely independent of GNU wget. We do not include, copy, or modify any GPL-licensed code from wget.

For compatibility testing, we maintain a **separate repository** (wget-faster-test) that is GPL-3.0 licensed and includes wget test suite integration. This ensures complete license separation while enabling comprehensive wget compatibility testing.

## Contributing

Contributions welcome! The main priorities are:
1. Implementing remaining CLI options for wget compatibility
2. Running wget test suite and fixing failures
3. Adding comprehensive tests
4. Performance optimization

## Acknowledgments

- GNU wget team for the excellent test suite
- reqwest, tokio, and the Rust async ecosystem

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

There are three ways to install wget-faster:

#### 1. From crates.io (Recommended)

If you have Rust installed, this is the easiest method:

```bash
# Install the wgetf CLI tool
cargo install wget-faster-cli

# Verify installation
wgetf --version
```

The binary will be installed to `~/.cargo/bin/wgetf` (make sure `~/.cargo/bin` is in your PATH).

To use wget-faster as a library in your Rust project:

```bash
# Add to your project
cargo add wget-faster-lib
```

#### 2. From GitHub Releases (Pre-built binaries)

Download pre-built binaries for your platform from [GitHub Releases](https://github.com/wget-faster/wget-faster/releases):

**Linux/macOS:**
```bash
# Download the latest release (replace VERSION and PLATFORM)
curl -LO https://github.com/wget-faster/wget-faster/releases/latest/download/wgetf-VERSION-PLATFORM.tar.gz

# Extract
tar xzf wgetf-*.tar.gz

# Move to PATH
sudo mv wgetf /usr/local/bin/

# Verify installation
wgetf --version
```

**Windows:**
- Download `wgetf-VERSION-windows.zip` from releases
- Extract the `.exe` file
- Add the directory to your PATH or move `wgetf.exe` to a directory in your PATH

#### 3. Build from Source

For the latest development version or custom builds:

```bash
# Clone the repository
git clone https://github.com/wget-faster/wget-faster.git
cd wget-faster

# Build release binary
cargo build --release

# The binary is now at ./target/release/wgetf
# Test it:
./target/release/wgetf --version

# Install to ~/.cargo/bin (optional)
cargo install --path wget-faster-cli

# Or manually copy to a directory in your PATH:
sudo cp ./target/release/wgetf /usr/local/bin/
```

**Requirements:**
- Rust 1.70 or later (install from [rustup.rs](https://rustup.rs))
- OpenSSL development libraries (Linux only):
  - Ubuntu/Debian: `sudo apt install pkg-config libssl-dev`
  - Fedora/RHEL: `sudo dnf install pkg-config openssl-devel`
  - macOS: Already included

### Basic Usage

```bash
# Simple download
wgetf https://example.com/file.zip

# Download to specific file
wgetf -O output.zip https://example.com/file.zip

# Resume interrupted download
wgetf -c https://example.com/large-file.iso

# With authentication
wgetf --http-user=username --http-password=password https://example.com/protected/file.zip

# Limit download speed to 1 MB/s
wgetf --limit-rate=1m https://example.com/large-file.iso

# Recursive download (mirror a website)
wgetf -r -l 3 -p -k https://example.com/

# Download only if remote is newer (timestamping)
wgetf -N https://example.com/file.zip

# POST request with data
wgetf --post-data="user=admin&pass=secret" https://example.com/api

# Download multiple URLs from a file
wgetf -i urls.txt

# Spider mode (check without downloading)
wgetf --spider https://example.com/file.zip

# With custom headers
wgetf --header="Authorization: Bearer token123" https://api.example.com/data

# Download with quota limit (stop after 100MB)
wgetf -Q 100m -i urls.txt

# Random wait between downloads (0.5-1.5 seconds)
wgetf -w 1 --random-wait -i urls.txt
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

### Current Version: 0.1.0

wget-faster v0.1.0 is a working, high-performance HTTP/HTTPS downloader with core wget compatibility.

### ✅ Implemented Features
- **Core Downloads**
  - Async/await architecture with tokio
  - HTTP/HTTPS with HTTP/2 support
  - Streaming downloads (constant ~10MB memory)
  - Resume support (`-c, --continue`)
  - Redirect following (configurable max)

- **Performance**
  - Parallel downloads via HTTP Range requests (4-32 connections)
  - Adaptive chunk sizing (256KB-10MB, automatic optimization)
  - Speed variance analysis and slow chunk re-splitting
  - Connection pooling

- **HTTP Features**
  - All major HTTP methods (GET, POST, PUT, DELETE, etc.)
  - Authentication (Basic, Digest)
  - Cookies (Netscape format, `--load-cookies`, `--save-cookies`)
  - Custom headers (`--header`, `-U`, `--referer`)
  - POST data (`--post-data`, `--post-file`)

- **Advanced Features**
  - Recursive downloads (`-r`) with HTML parsing
  - Page requisites (`-p`) - CSS, JS, images
  - Timestamping (`-N`) - download only if newer
  - Spider mode (`--spider`)
  - Input files (`-i`, `-F`)
  - Download quota (`-Q`)
  - Wait controls (`-w`, `--waitretry`, `--random-wait`)

- **SSL/TLS**
  - Certificate verification (rustls-tls)
  - Custom CA certificates
  - Client certificate authentication
  - `--no-check-certificate` option

- **CLI**
  - 150+ wget-compatible options parsed
  - Progress tracking with speed and ETA
  - Multiple output modes

### ⚠️ Known Limitations
- **Not Implemented**
  - `-S, --server-response` (parsed but not implemented)
  - `-k, --convert-links` (parsed but not implemented)
  - Directory control options (`-nd`, `-x`, `-nH`, `--cut-dirs`)
  - FTP/FTPS protocols
  - HTTP/3 (QUIC) - planned for v0.2.0
  - `.wgetrc` configuration file
  - WARC format output

- **Testing**
  - Current test coverage: ~10% (unit tests only)
  - wget test suite not yet integrated
  - Need more integration tests

- **Output**
  - Progress bar format differs slightly from wget
  - Some status messages have different wording

See [CHANGELOG.md](CHANGELOG.md) for complete details and [TODO.md](TODO.md) for roadmap.

### 📋 Planned for v0.1.1
- [ ] Comprehensive integration tests (target: 60%+ coverage)
- [ ] Server response display (`-S` option)
- [ ] Improved wget-style output formatting
- [ ] Fix all compiler warnings
- [ ] rustdoc for all public APIs

### 📋 Planned for v0.2.0
- [ ] HTTP/3 (QUIC) support
- [ ] Performance benchmarks vs GNU wget
- [ ] Zero-copy optimizations
- [ ] Memory profiling validation

### 📋 Planned for v0.3.0+
- [ ] Link conversion (`-k`)
- [ ] FTP/FTPS support
- [ ] wget test suite integration (60%+ pass rate)
- [ ] WARC format
- [ ] .netrc support
- [ ] robots.txt compliance

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - Version history and release notes
- [TODO.md](TODO.md) - Development roadmap and future plans
- [CLAUDE.md](CLAUDE.md) - Implementation details for AI/LLM context
- [SPEC.md](SPEC.md) - Technical specifications and architecture

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

# wget-faster

[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)

A high-performance, drop-in replacement for GNU wget written in Rust with async support and parallel downloads.

## Features

- **Fully async/non-blocking** - Built on tokio for efficient I/O
- **Parallel downloads** - Automatic Range request based parallel chunk downloads (8 connections by default for files >10MB)
- **Progress tracking** - Real-time download speed, progress, and ETA
- **Resume support** - Continue interrupted downloads with `-c`
- **HTTP/HTTPS** - Full support including authentication, cookies, custom headers
- **SSL/TLS** - Certificate verification, custom CA certificates, client certificates
- **Configurable** - Timeouts, retry logic with exponential backoff, speed limiting
- **Memory efficient** - Streaming downloads with low memory footprint

## Quick Start

### Installation

```bash
cargo build --release
```

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

# Limit download speed
wget-faster --limit-rate=1m https://example.com/large-file.iso
```

### Library Usage

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

- [x] Core library with async support
- [x] Parallel downloads via Range requests
- [x] Progress tracking
- [x] Resume support
- [x] HTTP/HTTPS with authentication
- [x] SSL/TLS configuration
- [x] Cookie support
- [ ] Complete CLI with all wget options
- [ ] wget test suite integration
- [ ] FTP/FTPS support
- [ ] Recursive download with HTML parsing
- [ ] WARC support
- [ ] Performance benchmarks

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

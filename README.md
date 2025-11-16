# wget-faster

[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
[![CI](https://github.com/wget-faster/wget-faster/actions/workflows/ci.yml/badge.svg)](https://github.com/wget-faster/wget-faster/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/wget-faster-cli.svg)](https://crates.io/crates/wget-faster-cli)

High-performance wget replacement in Rust with async I/O and parallel downloads.

**Performance**: 3-8x faster on large files • **Memory**: Constant ~10MB usage • **Compatibility**: 36% wget test suite passing (v0.0.4)

## Installation

```bash
# From crates.io
cargo install wget-faster-cli

# From source
git clone https://github.com/wget-faster/wget-faster.git
cd wget-faster
cargo install --path wget-faster-cli
```

## Quick Start

```bash
# Basic download
wgetf https://example.com/file.zip

# Resume download
wgetf -c https://example.com/large.iso

# Recursive download
wgetf -r -l 2 https://example.com/

# With authentication
wgetf --http-user=admin --http-password=secret https://example.com/file

# Disable parallel mode (GNU wget compatible)
wgetf --no-parallel https://example.com/file.zip
```

## Features

- **Parallel downloads** - Automatic chunked downloads via HTTP Range requests
- **Async I/O** - Non-blocking downloads with tokio
- **Resume support** - Continue interrupted downloads with `-c`
- **Recursive mode** - Download entire sites with `-r`
- **Authentication** - HTTP Basic/Digest, .netrc support
- **Cookies** - Netscape format compatible
- **Timestamping** - Download only if newer with `-N`
- **150+ wget options** - Drop-in replacement for most use cases

## Library Usage

```rust
use wget_faster_lib::{Downloader, DownloadConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::new(DownloadConfig::default())?;
    let bytes = downloader.download_to_memory("https://example.com/file.txt").await?;
    println!("Downloaded {} bytes", bytes.len());
    Ok(())
}
```

See [docs.rs](https://docs.rs/wget-faster-lib) for complete API documentation.

## Documentation

- [TODO.md](TODO.md) - Development roadmap
- [CLAUDE.md](CLAUDE.md) - Technical details for AI/LLM

## License

BSD-3-Clause - Clean-room implementation, no GPL code.

# CLAUDE.md - AI/LLM Context for wget-faster

Context document for AI assistants working with the wget-faster codebase.

## Project Overview

**wget-faster** is a high-performance HTTP downloader in Rust that exceeds GNU wget's performance through:
1. **HTTP/3 (QUIC)** - Lower latency, better congestion (planned)
2. **Parallel Downloads** - Chunked parallel downloads with adaptive optimization
3. **Adaptive Chunking** - Dynamic sizing based on network conditions
4. **Async I/O** - Non-blocking I/O with tokio

**Status**: v0.3.0 - Core features complete, ~4,250 lines of Rust, 30+ tests

## Project Structure

```
wget-faster/
├── wget-faster-lib/              # Core library
│   ├── lib.rs                    # Public API
│   ├── downloader.rs             # Main orchestrator
│   ├── client.rs                 # HTTP client (reqwest wrapper)
│   ├── parallel.rs               # Parallel download engine
│   ├── adaptive.rs               # Adaptive performance tuning
│   ├── recursive.rs              # Recursive downloads & HTML parsing
│   ├── cookies.rs                # Cookie management (Netscape format)
│   ├── progress.rs               # Progress tracking
│   ├── config.rs                 # Configuration types
│   ├── output.rs                 # Output abstraction
│   └── error.rs                  # Error types
└── wget-faster-cli/              # CLI
    ├── main.rs                   # Entry point
    ├── args.rs                   # 150+ wget-compatible options
    └── output.rs                 # wget-style formatting
```

## Architecture

### Module Responsibilities

**wget-faster-cli:**
- `args.rs` - Argument parsing with clap (150+ options)
- `main.rs` - Parse args → DownloadConfig → Downloader → execute
- `output.rs` - wget-style terminal output with indicatif

**wget-faster-lib:**
- `downloader.rs` - Strategy selection (sequential vs parallel), resume, retry, progress
- `client.rs` - HTTP/1.1, HTTP/2, auth, cookies, compression, TLS (reqwest + rustls)
- `parallel.rs` - Range detection, chunk splitting, parallel fetch, assembly
- `adaptive.rs` - Dynamic chunk sizing (256KB-10MB), connection tuning (4-32), slow chunk re-splitting
- `recursive.rs` - HTML parsing (scraper), link extraction, depth control, domain filtering
- `cookies.rs` - Netscape format I/O, domain/path/secure matching
- `progress.rs` - Speed calculation, ETA, thread-safe updates
- `config.rs` - DownloadConfig, RetryConfig, AuthConfig, HttpMethod
- `output.rs` - Output enum (Memory, File, AsyncWrite)
- `error.rs` - Error types with thiserror

### Key Decision Logic

**Downloader strategy:**
- Parallel if: file ≥ `parallel_threshold` (10MB) AND server supports Range requests
- Sequential if: file < threshold OR no Range support OR resume in progress

**Adaptive optimization:**
- Monitor per-chunk speed variance (coefficient of variation)
- High variance (>0.3): decrease chunk size, increase connections
- Low variance (<0.1): increase chunk size, decrease connections
- Re-split slow chunks dynamically

**Recursive downloads:**
- BFS queue with depth tracking
- Extract links from HTML (scraper with CSS selectors)
- Domain/regex filtering
- Download page requisites (CSS, JS, images) if `-p` enabled

## Core Types

### Public API
```rust
// Main entry point
pub struct Downloader { /* ... */ }
impl Downloader {
    pub async fn download_to_memory(&self, url: &str) -> Result<Bytes>
    pub async fn download_to_file(&self, url: &str, path: PathBuf) -> Result<()>
    pub async fn download_to_file_with_progress(/* ... */) -> Result<()>
}

// Configuration
pub struct DownloadConfig {
    pub timeout: Duration,                 // Default: 120s
    pub parallel_chunks: usize,            // Default: 8
    pub parallel_threshold: u64,           // Default: 10MB
    pub retry: RetryConfig,
    pub method: HttpMethod,
    pub auth: Option<AuthConfig>,
    pub enable_cookies: bool,
    pub timestamping: bool,
    pub quota: Option<u64>,
    pub wait_time: Option<Duration>,
    // ... ~30 more fields
}

// Progress tracking
pub struct ProgressInfo {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: f64,          // Bytes/sec (moving average)
    pub elapsed: Duration,
}

// Output modes
pub enum Output {
    Memory,
    File(PathBuf),
    AsyncWrite(Box<dyn AsyncWrite + Unpin + Send>),
}
```

## Implemented Features (v0.3.0)

### Core Engine
- ✅ Async/await with tokio
- ✅ Parallel downloads via HTTP Range requests
- ✅ Resume functionality (`-c`)
- ✅ Streaming (constant ~10MB memory)
- ✅ Exponential backoff retry

### Performance
- ✅ Adaptive chunk sizing (256KB-10MB)
- ✅ Dynamic connection count (4-32)
- ✅ HTTP/2 multiplexing (via reqwest)
- ✅ Speed variance analysis
- ✅ Slow chunk detection & re-splitting

### HTTP Features
- ✅ Methods: GET, HEAD, POST, PUT, DELETE, PATCH, OPTIONS
- ✅ Auth: Basic, Digest
- ✅ Cookies: Netscape format I/O
- ✅ Compression: gzip, deflate, brotli
- ✅ Redirects (max 20)
- ✅ Custom headers, User-Agent
- ✅ POST data (`--post-data`, `--post-file`)

### Advanced Features
- ✅ Timestamping (`-N`, If-Modified-Since)
- ✅ Recursive downloads (`-r`)
- ✅ Page requisites (`-p`) - CSS, JS, images
- ✅ Domain/regex filtering
- ✅ Quota (`-Q`) & wait controls (`-w`)
- ✅ Spider mode (`--spider`)
- ✅ Input files (`-i`, `-F`)

### Not Yet Implemented
- ❌ HTTP/3 (QUIC) - planned v0.4.0
- ❌ Link conversion (`-k`)
- ❌ FTP/FTPS
- ❌ .netrc authentication
- ❌ WARC format

## Common Development Tasks

### Adding a New wget Option

1. Add to `wget-faster-cli/src/args.rs`:
   ```rust
   #[arg(long = "new-option")]
   pub new_option: Option<String>,
   ```

2. Map to config in `wget-faster-cli/src/main.rs`

3. Add field to `DownloadConfig` if needed (in `wget-faster-lib/src/config.rs`)

4. Implement in appropriate library module

5. Add tests in `wget-faster-lib/tests/`

### Debugging Parallel Downloads

**Enable logging:**
```rust
// In downloader.rs
tracing::debug!("File size: {}, using parallel: {}", size, is_parallel);

// In parallel.rs
tracing::debug!("Chunk {}/{}: {} bytes", chunk_num, total, bytes);
```

**Common issues:**
- Server lacks Range support → Check `Accept-Ranges` header
- File too small → Check `parallel_threshold` (default 10MB)
- Slow performance → Increase chunk count or check network latency

### Adding HTTP/3 Support (Future)

1. Add dependencies: `quinn`, `h3` (as optional features)
2. Create `client_h3.rs` with QUIC connection logic
3. Detect HTTP/3 via Alt-Svc header in `downloader.rs`
4. Fallback to HTTP/2 if unavailable
5. Benchmark vs HTTP/2

## Code Style

- **Edition:** 2021
- **Formatting:** rustfmt, clippy pedantic mode
- **Async:** Use `async fn` for all I/O
- **Errors:** `Result<T, Error>` with `?` operator, thiserror
- **Naming:** `snake_case` functions, `PascalCase` types, descriptive names
- **Documentation:** rustdoc comments for all public APIs

**Error handling:**
```rust
// Good: Propagate with ?
pub async fn download(&self, url: &str) -> Result<Bytes> {
    let response = self.client.get(url).await?;
    Ok(response.bytes().await?)
}

// Avoid: unwrap/expect in library code
```

## Testing

**Unit tests:** `cargo test --lib` (wget-faster-lib/tests/)
- Integration tests, cookie tests, progress tests

**CLI tests:** `cargo test --bin wget-faster`

**wget compatibility:** Separate GPL-3.0 repo (binary-only testing)

## Quick Reference: Finding Implementations

| Feature | File | Key Function/Type |
|---------|------|-------------------|
| Adaptive chunking | `adaptive.rs` | `AdaptiveDownloader::download_adaptive()` |
| Cookie management | `cookies.rs` | `CookieJar::load_from_file()`, `to_cookie_header()` |
| Recursive downloads | `recursive.rs` | `RecursiveDownloader::download_recursive()` |
| HTML parsing | `recursive.rs` | `extract_links()`, `extract_requisites()` |
| Parallel downloads | `parallel.rs` | `download_parallel()` |
| Progress tracking | `progress.rs` | `ProgressInfo` struct |
| HTTP methods | `config.rs` | `HttpMethod` enum |
| Timestamping | `downloader.rs` | If-Modified-Since logic |
| Retry logic | `downloader.rs` | Exponential backoff loop |

## Basic API Example

```rust
use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let downloader = Downloader::new(DownloadConfig::default())?;

    // Simple download
    let bytes = downloader.download_to_memory("https://example.com/file.txt").await?;

    // With progress
    let progress = |info: ProgressInfo| {
        if let Some(pct) = info.percentage() {
            println!("{:.1}% - {}", pct, info.format_speed());
        }
    };
    downloader.download_to_file_with_progress(
        "https://example.com/large.zip",
        "large.zip".into(),
        Some(Arc::new(progress)),
    ).await?;

    Ok(())
}
```

## Roadmap

**Current (v0.3.0):** Core features complete
- ✅ Parallel downloads, adaptive chunking, recursive downloads
- ✅ 40+ wget options, 30+ tests, CI/CD

**Next (v0.4.0):** Advanced performance
- ❌ HTTP/3 (QUIC) support
- ❌ Zero-copy chunk assembly (io_uring on Linux)
- ❌ Real benchmarks vs GNU wget
- ❌ Predictive prefetching for recursive downloads

**Long-term (v1.0.0):** Production ready
- Full wget compatibility (currently ~80%)
- Comprehensive documentation
- 60%+ test coverage
- Man pages

## Architecture Decisions

**Why Tokio?** Industry-standard async runtime, excellent I/O performance, rich ecosystem

**Why reqwest?** High-level HTTP client, built-in HTTP/2, compression, cookies, rustls-tls

**Why Netscape cookie format?** Standard format, wget/curl compatible, simple text

**Why scraper for HTML?** CSS selectors, fast html5ever parser, robust

## Dependencies

- **Core:** tokio, reqwest, rustls-tls
- **Parsing:** scraper, url, httpdate
- **CLI:** clap, indicatif
- **Testing:** tokio::test

## License

BSD-3-Clause - Clean-room implementation, independent from GNU wget (no GPL code)

---

**Performance Philosophy:** wget-faster is a next-generation downloader built for speed. Every feature should consider performance impact. Expected gains: 3-8x faster for large files via parallel downloads.

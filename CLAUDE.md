# CLAUDE.md - AI/LLM Context for wget-faster

Context document for AI assistants working with the wget-faster codebase.

## Project Overview

**wget-faster** is a high-performance HTTP downloader in Rust that exceeds GNU wget's performance through:
1. **HTTP/3 (QUIC)** - Lower latency, better congestion (planned)
2. **Parallel Downloads** - Chunked parallel downloads with adaptive optimization
3. **Adaptive Chunking** - Dynamic sizing based on network conditions
4. **Async I/O** - Non-blocking I/O with tokio

**Status**: v0.0.1 - Core features complete, ~4,250 lines of Rust, 30+ tests

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

## Implemented Features (v0.0.1)

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
- ❌ HTTP/3 (QUIC) - planned v0.1.0
- ❌ Link conversion (`-k`) - planned v0.2.0
- ❌ Server response display (`-S`) - planned v0.0.2
- ❌ FTP/FTPS - planned v0.2.0
- ❌ .netrc authentication - planned v0.2.0
- ❌ WARC format - planned v0.2.0

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

### Unit and Integration Tests

**Unit tests:** `cargo test --lib` (wget-faster-lib/tests/)
- Integration tests, cookie tests, progress tests

**CLI tests:** `cargo test --bin wget-faster`

### wget Compatibility Testing

**Test Repository:** [wget-faster-test](https://github.com/daveDev42/wget-faster-test) (GPL-3.0, separate repo)
- Binary-only testing (no code linking to maintain BSD license)
- Runs GNU wget's official test suite against wget-faster

**Running wget compatibility tests:**

```bash
# Navigate to the test repository (sibling directory)
cd ../wget-faster-test

# Build wget-faster first
cd ../wget-faster
cargo build --release

# Run all tests
cd ../wget-faster-test
./run_tests.sh

# Run specific test suite
./run_tests.sh --perl-only      # Only Perl tests
./run_tests.sh --python-only    # Only Python tests (if available)

# Specify wget-faster binary path
./run_tests.sh --wget-faster /path/to/wgetf
```

**Viewing test results:**

```bash
# View the latest test report (markdown format)
cd ../wget-faster-test/reports
ls -t report_*.md | head -1 | xargs cat

# View the latest test results (JSON format)
ls -t test_results_*.json | head -1 | xargs cat

# Or use the report file directly
cat reports/report_$(ls -t reports/ | grep report | head -1)
```

**Current test status (v0.0.1):**
- **Pass rate:** ~18% (16/87 Perl tests)
- **Passing tests:** Basic HTTP downloads, cookies, resume functionality, Content-Disposition
- **Common failures:**
  - FTP tests (not implemented)
  - IRI/IDN tests (internationalization not implemented)
  - HTTPS advanced features (client certs, CRL)
  - CLI argument parsing issues (`-n` option, `--debug` duplication)
  - Relative file path handling

**Test report format:**
- Summary table (passed/total/pass rate by suite)
- Failed test details (exit code, stderr, execution time)
- Complete test list with status and timing

**Directory structure:**
```
wget-faster-test/
├── runner/              # Python test execution framework
│   ├── test_runner.py
│   └── report_generator.py
├── reports/             # Generated test results and reports
│   ├── test_results_*.json
│   └── report_*.md
├── wget-repo/           # GNU wget source (git clone)
│   └── tests/           # Perl test suite
├── config/              # Test configuration
└── run_tests.sh         # Main test script
```

**Interpreting test failures:**

1. **Exit code 77:** Test skipped (feature not detected via `wget --version`)
2. **Exit code 1:** Test failed (behavior differs from wget)
3. **"missing URL":** CLI parsing issue, wget feature detection failed
4. **"unexpected argument":** Option not implemented or parsed incorrectly
5. **"builder error for url (ftp://...)":** FTP not supported (HTTP-only client)

**Improving test pass rate:**

Priority fixes to increase pass rate quickly:
1. Implement `-n` / `--no-directories` option
2. Allow `--debug` to be specified multiple times
3. Fix relative file path handling for `-i` and `--post-file`
4. Improve HTTP error status code handling (400, 401, 404)
5. Fix Content-Disposition with timestamping (`-N`)

Long-term improvements (v0.2.0+):
- FTP/FTPS support (17 tests)
- IRI/IDN support (14 tests)
- HTTPS advanced features (8 tests)

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

**Current (v0.0.1):** Core features complete
- ✅ Parallel downloads, adaptive chunking, recursive downloads
- ✅ 150+ wget options, 30+ tests, CI/CD
- ⚠️ Test coverage ~10%, needs expansion to 60%+

**Next (v0.0.2):** Testing & quality
- Comprehensive integration tests with mockito
- Server response display (`-S`)
- Improved output formatting
- Fix compiler warnings
- rustdoc for all public APIs

**v0.1.0:** Performance & HTTP/3
- HTTP/3 (QUIC) support
- Zero-copy chunk assembly (io_uring on Linux)
- Real benchmarks vs GNU wget
- Memory profiling validation

**v0.2.0:** Advanced features
- Link conversion (`-k`)
- FTP/FTPS support
- wget test suite integration (60%+ pass rate)
- Directory control options

**v1.0.0:** Production ready
- Full wget compatibility (95%+)
- Comprehensive documentation
- Man pages and shell completions
- Package distribution

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

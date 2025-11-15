# CLAUDE.md - AI/LLM Context for wget-faster

Context document for AI assistants working with the wget-faster codebase.

## Project Overview

**wget-faster** is a high-performance HTTP downloader in Rust that exceeds GNU wget's performance through:
1. **HTTP/3 (QUIC)** - Lower latency, better congestion (planned)
2. **Parallel Downloads** - Chunked parallel downloads with adaptive optimization
3. **Adaptive Chunking** - Dynamic sizing based on network conditions
4. **Async I/O** - Non-blocking I/O with tokio

**Status**: v0.0.4 - Core features complete, ~4,250 lines of Rust, 30+ tests, 36.1% wget test compatibility

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
- ✅ .netrc authentication (automatic credentials)

### Not Yet Implemented
- ❌ HTTP/3 (QUIC) - planned v0.1.0
- ❌ Link conversion (`-k`) - planned v0.2.0
- ❌ Server response display (`-S`) - planned v0.0.4
- ❌ FTP/FTPS - planned v0.2.0
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
# Standard test workflow (RECOMMENDED)
cd ../wget-faster-test
rm -rf reports  # Clean previous test reports (recommended)
./run_tests.sh --wget-faster $(which wgetf) -v

# Alternative: Manual steps
# 1. Build wget-faster first
cd ../wget-faster
cargo build --release
cargo install --path wget-faster-cli  # Installs 'wgetf' binary

# 2. Run all tests
cd ../wget-faster-test
./run_tests.sh

# Run specific test suite
./run_tests.sh --perl-only      # Only Perl tests
./run_tests.sh --python-only    # Only Python tests (if available)

# Specify custom binary path
./run_tests.sh --wget-faster /path/to/wgetf -v
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

### Test Failure Analysis Workflow

**After running tests, analyze and document failures:**

```bash
# 1. Run tests and get results
cd ../wget-faster-test
./run_tests.sh --wget-faster $(which wgetf) -v

# 2. Analyze failures and generate documentation
cd ../wget-faster
python3 scripts/analyze_tests.py ../wget-faster-test/reports/test_results_*.json

# 3. Review generated documentation
ls todo/               # Individual test failure docs
cat todo/README.md     # Category summary and index

# 4. Investigate specific test
# Open todo/${test_name}.md and add implementation notes
# Example: todo/Test-condget_py.md

# 5. After fixing issues, re-run tests and update analysis
cd ../wget-faster-test
./run_tests.sh --wget-faster $(which wgetf) -v
cd ../wget-faster
python3 scripts/analyze_tests.py ../wget-faster-test/reports/test_results_*.json
```

**Test documentation structure:**

```
wget-faster/
├── scripts/
│   └── analyze_tests.py      # Test analysis script
├── todo/
│   ├── README.md              # Index of all failed tests by category
│   ├── Test-*.md              # Individual test documentation
│   └── ...                    # One file per failed test
└── TODO.md                    # Links to todo/README.md
```

**Analysis script features:**

1. **Automatic categorization** - Groups failures by type:
   - `missing_feature_metalink` - Metalink not implemented (32 tests)
   - `missing_feature_ftp` - FTP not implemented (14 tests)
   - `skipped_ssl_tls` - Advanced SSL/TLS features (10 tests)
   - `test_framework_missing_file` - Expected files not created (7 tests)
   - `test_framework_content_mismatch` - Content doesn't match (6 tests)
   - `test_framework_crawl_mismatch` - Files crawled incorrectly (5 tests)
   - `timeout` - Tests timing out (3 tests)
   - `unknown` - Needs manual investigation

2. **Detailed documentation** - Each test file includes:
   - Test description and metadata
   - Full error messages and output
   - Automatic analysis of likely causes
   - Suggested investigation steps
   - Space for implementation notes

3. **Progress tracking** - Update documentation as you fix issues:
   ```markdown
   ## Implementation Notes

   **2025-11-15**: Investigated conditional GET logic
   - HEAD returns 304 correctly
   - Issue: GET request still sent after 304
   - Need to add early return in downloader.rs:350

   **2025-11-16**: Fixed
   - Added early return when HEAD returns 304
   - Test now passing
   ```

**Quick win identification:**

The analysis script highlights high-impact, low-effort fixes:
- Tests failing due to bugs rather than missing features
- Grouped by category for batch fixing
- Estimated improvement potential

See `TODO.md` "Test Failure Analysis" section for latest results.

**Current test status (v0.0.4 - 2025-11-15):**
- **Pass rate:** 36.1% (61/169 total tests) ⬆️ **+14.8%** from v0.0.3
  - Perl: 50.6% (44/87 tests) ⬆️ **+21.9%** (+19 tests)
  - Python: 20.7% (17/82 tests) ⬆️ **+7.3%** (+6 tests)
- **Passing tests:** Basic HTTP downloads, cookies, resume functionality (-c), Content-Disposition (all variants), recursive downloads (significantly improved), spider mode (basic + fail detection), timestamping (-N with all Content-Disposition variants), output handling (-O), meta-robots, filename restrictions (ASCII)
- **Recent improvements (v0.0.4):**
  - ✅ **HEAD request optimization** - Major refactoring for wget compatibility (+25 tests)
    - Skip HEAD requests for simple downloads when parallel is disabled
    - Optimized `is_html_url()` to check extension first (eliminates hundreds of HEAD requests)
    - Only send HEAD when needed: parallel downloads, timestamping, or uncertain content-type
  - ✅ **Recursive download improvements** - Massive improvement in recursive crawling
    - ~50% reduction in HTTP requests during recursive downloads
    - Better wget compatibility for link extraction
- **Previous improvements (v0.0.3):**
  - ✅ HTTP 401/407 authentication retry with credentials
  - ✅ .netrc file support for automatic authentication
  - ✅ Exit codes (3 for I/O, 6 for auth, 8 for HTTP errors)
  - ✅ Spider mode (--spider and --spider-fail working)
  - ✅ Timestamping (-N) with file mtime setting → 5/8 tests passing
  - ✅ Content-Disposition header parsing (basic + advanced) → 6/7 tests passing
  - ✅ Resume/continue functionality (-c)
- **Common failures:**
  - HTTP 204 No Content handling (creates empty file, should not create file - 1 test)
  - Recursive downloads with link conversion (-r -E -k - 2 tests: needs -k implementation)
  - Content-Disposition edge case (duplicate filename numbering .1, .2, .3 - 1 test)
  - Timestamping edge cases (old files, size changes, missing Last-Modified - 3 tests)
  - FTP tests (not implemented - 18 tests)
  - IRI/IDN tests (internationalization not implemented - 11 tests)
  - Advanced HTTPS/TLS features (client certs, CRL - 8 tests)

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

**Improving test pass rate (v0.0.3 priorities):**

Recently completed fixes (+6 tests):
1. ✅ **HTTP auth retry** - Implemented 401/407 authentication retry with credentials
2. ✅ **.netrc support** - Automatic authentication from .netrc file
3. ✅ **Timestamping (-N)** - File mtime setting (5/8 tests passing, 3 edge cases remain)
4. ✅ **Content-Disposition** - Header parsing (6/7 tests passing, 1 edge case remains)
5. ✅ **Spider mode** - Basic spider and fail detection working

Remaining quick wins (can increase pass rate to ~25-30%):
1. **HTTP 204 handling** - Don't create empty files for 204 No Content (1 test)
2. **Timestamping edge cases** - Handle old files, size changes, missing headers (3 tests)
3. **Content-Disposition numbering** - Add .1, .2, .3 suffix for duplicates (1 test)
4. **CLI argument parsing** - Allow `--no-directories` multiple times (1 test)
5. **Recursive link extraction** - Fix link extraction for -r -E -k (2 tests)

See TODO.md for complete list of 19 prioritized tasks.

Long-term improvements (v0.2.0+):
- FTP/FTPS support (18 tests)
- IRI/IDN support (11 tests)
- Advanced HTTPS/TLS features (8 tests)

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

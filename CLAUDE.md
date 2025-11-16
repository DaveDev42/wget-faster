# CLAUDE.md - AI/LLM Context for wget-faster

## Documentation Policy

**Keep project root clean**: Only essential markdown files in root

**Core docs** (root): README.md, CLAUDE.md, TODO.md, LICENSE

**Never create**: New markdown files in root without explicit request. Consolidate findings into TODO.md.

**Test reports**: Always in `../wget-faster-test/reports/`, never in main repo.

## Project Overview

High-performance HTTP downloader in Rust. **v0.0.4**: 36.1% wget test compatibility, ~4,250 lines Rust, 30+ tests.

**Key features**: Parallel downloads (Range requests), adaptive chunking, async I/O (tokio), recursive downloads, cookies, auth.

## Structure

```
wget-faster/
├── wget-faster-lib/     # Core library
│   ├── downloader.rs    # Main orchestrator, strategy selection, retry
│   ├── client.rs        # HTTP client (reqwest/rustls)
│   ├── parallel.rs      # Range requests, chunking
│   ├── adaptive.rs      # Dynamic chunk sizing
│   ├── recursive.rs     # HTML parsing, link extraction
│   ├── cookies.rs       # Netscape format
│   ├── progress.rs      # Speed/ETA tracking
│   └── config.rs        # Configuration types
└── wget-faster-cli/     # CLI (150+ wget options)
    ├── args.rs          # Argument parsing (clap)
    └── main.rs          # Entry point
```

## Architecture

**Downloader strategy**:
- Parallel: file ≥10MB AND server supports Range requests
- Sequential: file <10MB OR no Range support OR `--no-parallel`

**HEAD request optimization** (v0.0.4):
- Skip HEAD when `--no-parallel` (GNU wget compatibility)
- Send HEAD only for: parallel downloads, timestamping, uncertain content-type
- `is_html_url()` checks extension first before HEAD

**Adaptive chunking**:
- Monitor chunk speed variance
- High variance (>0.3): smaller chunks, more connections
- Low variance (<0.1): larger chunks, fewer connections

## Core API

```rust
pub struct Downloader {
    pub async fn download_to_memory(&self, url: &str) -> Result<Bytes>
    pub async fn download_to_file(&self, url: &str, path: PathBuf) -> Result<()>
    pub async fn download_to_file_with_progress(/* ... */) -> Result<()>
}

pub struct DownloadConfig {
    pub timeout: Duration,           // 120s
    pub parallel_chunks: usize,      // 8
    pub parallel_threshold: u64,     // 10MB
    pub method: HttpMethod,
    pub auth: Option<AuthConfig>,
    // ... ~30 fields total
}
```

## Common Tasks

### Adding wget Option

1. `wget-faster-cli/src/args.rs`: Add `#[arg(long = "new-option")]`
2. `wget-faster-cli/src/main.rs`: Map to DownloadConfig
3. `wget-faster-lib/src/config.rs`: Add field if needed
4. Implement in appropriate module
5. Add tests

### Debugging Parallel Downloads

**Check**:
- Server Range support: `Accept-Ranges: bytes` header
- File size ≥ `parallel_threshold` (10MB)
- Not using `--no-parallel`

**Logging**: `tracing::debug!()` in downloader.rs, parallel.rs

### Parallel vs GNU wget Compatibility

- **Default**: HEAD → GET (parallel if ≥10MB)
- **`--no-parallel`**: GET only (no HEAD, sequential) - sets `parallel_chunks=1`, `parallel_threshold=0`

## Testing

### Unit Tests

```bash
cargo test --lib              # Library tests
cargo test --bin wget-faster  # CLI tests
```

### wget Test Suite

**Repo**: [wget-faster-test](https://github.com/daveDev42/wget-faster-test) (GPL-3.0, separate)

```bash
# Run tests
cd ../wget-faster-test
./run_tests.sh --wget-faster $(which wgetf) -v

# View results
cd reports && ls -t report_*.md | head -1 | xargs cat

# Analyze failures
cd ../wget-faster
python3 scripts/analyze_tests.py ../wget-faster-test/reports/test_results_*.json
```

**Current**: 36.1% pass rate (61/169 tests)
- Perl: 50.6% (44/87)
- Python: 20.7% (17/82)

**Passing**: Basic HTTP, cookies, resume, Content-Disposition, recursive, spider, timestamping, .netrc

**Common failures**: FTP (not impl), IRI/IDN (not impl), advanced TLS, HTTP 204 handling, link conversion (-k)

### Test Failure Analysis

Script `scripts/analyze_tests.py` auto-categorizes failures:
- `missing_feature_metalink`, `missing_feature_ftp` - Not implemented
- `test_framework_*` - Test bugs or edge cases
- `timeout` - Performance issues

Generates `todo/*.md` with details and investigation steps.

## Development

### Local CI Checks

```bash
cargo fmt --all                              # Format
cargo clippy --all-targets --all-features    # Lint
cargo test --all-features                    # Test
cargo llvm-cov --workspace --html            # Coverage
cargo bench                                  # Benchmark
cargo audit                                  # Security
```

### Code Style

- **rustfmt**: 100 char lines, Unix newlines
- **clippy**: Pedantic, complexity ≤20, max 6 args, no unwrap in lib
- **Pre-commit**: Auto-runs format, clippy, tests

### Release

```bash
git checkout -b release/v0.1.0
# Update Cargo.toml version, merge PR
# Auto-tags, builds, publishes to crates.io
```

## Quick Reference

| Feature | File | Function |
|---------|------|----------|
| Adaptive chunking | adaptive.rs | `download_adaptive()` |
| Cookies | cookies.rs | `CookieJar::load_from_file()` |
| Recursive | recursive.rs | `download_recursive()` |
| Parallel | parallel.rs | `download_parallel()` |
| Progress | progress.rs | `ProgressInfo` |
| Retry | downloader.rs | Exponential backoff |

## Dependencies

- **Core**: tokio (async), reqwest (HTTP), rustls-tls
- **Parse**: scraper (HTML), url, httpdate
- **CLI**: clap (args), indicatif (progress)
- **Test**: criterion (bench), mockito (mocks)

---

**Status**: v0.0.4 | **License**: BSD-3-Clause | **Updated**: 2025-11-16

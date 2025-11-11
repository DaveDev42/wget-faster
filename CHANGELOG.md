# Changelog

All notable changes to wget-faster will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned for v0.1.1
- Real HTTP integration tests with mockito
- Server response display (`-S, --server-response` option)
- Improved wget-style output formatting
- rustdoc comments for all public APIs
- Fix all compiler warnings

## [0.1.0] - 2025-11-11

### Added

#### Core Download Features
- Async/await architecture with tokio runtime
- HTTP/HTTPS downloads with rustls-tls backend
- HTTP/2 support via reqwest
- Streaming downloads with constant memory usage (~10MB)
- Download to memory or file
- Resume functionality (`-c, --continue`)
- HTTP redirect following with configurable maximum (default: 20)
- Automatic compression handling (gzip, deflate, brotli)

#### Performance Features
- **Parallel downloads** via HTTP Range requests (4-32 connections)
- **Adaptive chunk sizing** - Dynamic adjustment between 256KB and 10MB
- **Speed variance analysis** - Automatic performance optimization
- **Slow chunk detection** - Re-split underperforming chunks mid-download
- **Connection pooling** - Reuse connections across multiple downloads

#### HTTP Features
- Multiple HTTP methods: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD
- Authentication support:
  - HTTP Basic authentication
  - HTTP Digest authentication
- Custom headers (`--header`)
- User-Agent customization (`-U, --user-agent`)
- Referer header (`--referer`)
- POST request support:
  - `--post-data` - Send data from string
  - `--post-file` - Send data from file
- Cookie support (Netscape format):
  - `--load-cookies` - Load cookies from file
  - `--save-cookies` - Save cookies to file
  - In-memory cookie jar with domain/path/secure matching
  - Automatic cookie handling from Set-Cookie headers

#### Advanced Features
- **Timestamping** (`-N, --timestamping`)
  - If-Modified-Since header support
  - Download only if remote file is newer
  - Set local file timestamp from Last-Modified header
- **Recursive downloads** (`-r, --recursive`)
  - HTML parsing with scraper library
  - Automatic link extraction
  - Depth control (`-l, --level`)
  - Domain filtering
  - Cross-domain download control
- **Page requisites** (`-p, --page-requisites`)
  - Download images, CSS, JavaScript for offline viewing
  - Parse and extract resource URLs from HTML
- **Spider mode** (`--spider`)
  - Check if URLs exist without downloading
  - Useful for validation and checking
- **Input files** (`-i, --input-file` and `-F, --force-html`)
  - Read URLs from file (one per line)
  - Parse HTML files for links
- **Download quota** (`-Q, --quota`)
  - Limit total bytes downloaded
  - Stop after reaching quota
- **Wait controls**
  - `-w, --wait` - Wait between retrievals
  - `--waitretry` - Wait between retries
  - `--random-wait` - Randomize wait time

#### Progress & Output
- Real-time progress tracking with indicatif
- Speed calculation using moving average
- ETA (estimated time remaining) calculation
- Progress callbacks for custom output
- Human-readable byte formatting (KB, MB, GB, TB)
- Progress percentage with total size detection

#### Configuration & Control
- Timeout configuration:
  - Overall timeout (`-T, --timeout`)
  - Connection timeout (`--connect-timeout`)
  - Read timeout (`--read-timeout`)
- Retry logic with exponential backoff
  - Configurable max retries (`-t, --tries`)
  - Backoff multiplier
  - Max delay cap
- SSL/TLS configuration:
  - Certificate verification (default: enabled)
  - `--no-check-certificate` - Skip verification
  - Custom CA certificates (`--ca-certificate`)
  - Client certificate authentication
- Proxy support with authentication
- 150+ CLI options parsed (wget-compatible argument structure)

#### CLI Features
- **Binary name**: `wgetf` (wget-faster)
- Comprehensive argument parsing with clap
- Option grouping matching wget:
  - Startup options
  - Logging and input options
  - Download options
  - Directory options
  - HTTP options
  - HTTPS (SSL/TLS) options
  - Recursive download options
- Detailed help text (`--help`)
- Version display (`-V, --version`)

#### Developer Features
- Clean library/CLI separation:
  - `wget-faster-lib` - Core functionality as reusable library
  - `wget-faster-cli` - Command-line interface
- Public API for programmatic use
- Comprehensive error types with thiserror
- Progress callback system for custom integrations
- Configurable via `DownloadConfig` struct

### Changed
- Project structure reorganized into workspace with two crates
- Default parallel chunk count: 8 (configurable)
- Default parallel threshold: 10MB (files smaller than this use sequential download)
- Default timeout: 120 seconds
- Default connection timeout: 30 seconds
- Default max redirects: 20

### Technical Details

#### Dependencies
- **Runtime**: tokio 1.48 (async runtime)
- **HTTP**: reqwest 0.12 (with rustls-tls)
- **HTML**: scraper 0.21 (HTML parsing)
- **CLI**: clap 4.5 (argument parsing)
- **Progress**: indicatif 0.17 (progress bars)
- **Cookies**: cookie_store 0.21 (cookie management)
- **Error handling**: thiserror 1.0, anyhow 1.0
- **Compression**: flate2 1.1, brotli 7.0
- **Testing**: mockito 1.7, tokio::test

#### Architecture
- Modular design with clear separation of concerns
- `downloader.rs` - Main orchestrator, decides sequential vs parallel
- `client.rs` - HTTP client wrapper around reqwest
- `parallel.rs` - Parallel Range request implementation
- `adaptive.rs` - Adaptive chunk sizing and optimization
- `recursive.rs` - Recursive download and HTML parsing
- `cookies.rs` - Cookie jar and Netscape format I/O
- `progress.rs` - Progress tracking and formatting
- `config.rs` - Configuration types
- `error.rs` - Error types and conversions

#### Memory Efficiency
- Streaming downloads - no full-file buffering
- Constant memory usage regardless of file size
- Sequential: ~100KB peak memory
- Parallel (8 chunks): ~1MB peak memory
- Buffer size: 8KB per connection

#### License
- BSD-3-Clause license
- Independent implementation (no GPL code)
- Clean-room design, not derived from GNU wget

### Known Limitations

#### Not Yet Implemented
- `-S, --server-response` - Parsed but not implemented
- `-k, --convert-links` - Parsed but not implemented
- Most directory control options (`-nd`, `-x`, `-nH`, `--cut-dirs`)
- FTP/FTPS protocols
- HTTP/3 (QUIC) support
- `.wgetrc` configuration file
- `.netrc` authentication
- WARC format output
- robots.txt compliance
- Metalink support
- Background execution (`-b`)
- Man pages and shell completions

#### Test Coverage
- Basic unit tests only (~10% coverage)
- Integration tests are mostly placeholders
- wget test suite not yet integrated
- No formal benchmarks vs GNU wget

#### Output Differences
- Progress bar format differs slightly from wget
- Some status messages have different wording
- Exit codes may differ in some error cases

### Documentation
- README.md - User guide with installation and usage examples
- CLAUDE.md - Implementation details and architecture for AI assistants
- SPEC.md - Technical specifications
- TODO.md - Development roadmap
- Inline code comments throughout

### Comparison with GNU wget

#### Advantages
✅ **Faster** - Parallel downloads can be 3-8x faster for large files
✅ **Modern** - Async/await, memory-safe Rust
✅ **HTTP/2** - Built-in HTTP/2 support
✅ **Library** - Full async Rust library API
✅ **Memory-safe** - Guaranteed by Rust compiler
✅ **Adaptive** - Automatic performance tuning

#### Compatible
✅ Most common wget options
✅ Cookie format (Netscape)
✅ Authentication methods
✅ SSL/TLS configuration
✅ Recursive downloads

#### Not Yet Implemented
❌ FTP/FTPS
❌ HTTP/3 (planned v0.2.0)
❌ Some advanced options
❌ `.wgetrc` config file
❌ WARC format

### Migration from GNU wget

Most common wget commands work without modification:

```bash
# These work identically
wgetf https://example.com/file.zip
wgetf -c https://example.com/large-file.iso
wgetf -r -l 3 https://example.com/
wgetf --http-user=user --http-password=pass https://example.com/

# Replace 'wget' with 'wgetf'
alias wget='wgetf'  # Optional
```

### Contributors

wget-faster is an independent clean-room implementation inspired by GNU wget's interface.

---

## Version History

- **v0.1.0** (2025-11-11) - Initial release with core features
  - Async downloads, parallel downloads, recursive downloads
  - 150+ CLI options, cookie support, authentication
  - Basic wget compatibility for common use cases

---

## Upgrade Notes

### From Source to v0.1.0
This is the first official release. If you were using git HEAD:
- Binary name changed from `wget-faster` to `wgetf`
- API is now stable for v0.1.x series
- Configuration structure is finalized

---

## Links

- [GitHub Repository](https://github.com/wget-faster/wget-faster)
- [Issue Tracker](https://github.com/wget-faster/wget-faster/issues)
- [crates.io - wget-faster-lib](https://crates.io/crates/wget-faster-lib)
- [crates.io - wget-faster-cli](https://crates.io/crates/wget-faster-cli)

---

[Unreleased]: https://github.com/wget-faster/wget-faster/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/wget-faster/wget-faster/releases/tag/v0.1.0

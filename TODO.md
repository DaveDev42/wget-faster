# TODO - wget-faster Development Roadmap

**Current Version**: v0.0.1
**Last Updated**: 2025-11-11

---

## Overview

wget-faster is a high-performance HTTP downloader in Rust that aims to be a drop-in replacement for GNU wget with better performance through async I/O and parallel downloads.

### Version Strategy

- **v0.0.x** - Initial development and core features (current)
- **v0.1.x** - Performance optimizations and HTTP/3
- **v0.2.x** - Advanced features and full wget compatibility
- **v1.0.0** - Production-ready release

---

## v0.0.1 Status - Completed ✅

### Core Download Features
- [x] Async/await with tokio runtime
- [x] HTTP/HTTPS downloads with rustls-tls
- [x] HTTP/2 support (via reqwest)
- [x] Streaming downloads (constant memory usage)
- [x] Download to memory or file
- [x] Resume support (`-c, --continue`)
- [x] Redirects with max limit (default: 20)
- [x] Compression (gzip, deflate, brotli)

### Performance Features
- [x] Parallel downloads via HTTP Range requests
- [x] Adaptive chunk sizing (256KB-10MB)
- [x] Dynamic connection count (4-32)
- [x] Speed variance analysis
- [x] Slow chunk detection and re-splitting
- [x] Connection pooling

### HTTP Features
- [x] Multiple HTTP methods (GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD)
- [x] Authentication (Basic, Digest)
- [x] Custom headers (`--header`)
- [x] User-Agent customization (`-U`)
- [x] Referer header (`--referer`)
- [x] POST data (`--post-data`, `--post-file`)
- [x] Cookie support (Netscape format)
  - [x] Load cookies (`--load-cookies`)
  - [x] Save cookies (`--save-cookies`)
  - [x] In-memory cookie jar
  - [x] Domain/path/secure matching

### Advanced Features
- [x] Timestamping (`-N`, If-Modified-Since)
- [x] Recursive downloads (`-r, --recursive`)
- [x] Page requisites (`-p`) - download CSS, JS, images
- [x] HTML parsing with scraper
- [x] Link extraction
- [x] Domain filtering
- [x] Depth control
- [x] Spider mode (`--spider`)
- [x] Input files (`-i`, `-F`)
- [x] Download quota (`-Q`)
- [x] Wait controls (`-w`, `--waitretry`, `--random-wait`)

### Progress & Output
- [x] Real-time progress tracking
- [x] Speed calculation (moving average)
- [x] ETA estimation
- [x] Progress callbacks
- [x] Byte formatting (KB, MB, GB, TB)

### Configuration
- [x] Timeout configuration (connect, read, overall)
- [x] Retry with exponential backoff
- [x] SSL/TLS configuration
  - [x] Certificate verification
  - [x] Custom CA certificates
  - [x] Client certificates
  - [x] `--no-check-certificate` option
- [x] Proxy support with authentication
- [x] 150+ CLI options parsed (wget-compatible)

### Testing
- [x] Basic unit tests (cookie, progress, config)
- [x] 30+ tests across 3 test files
- [x] Integration test framework

### Documentation
- [x] README.md with examples
- [x] CLAUDE.md for AI context
- [x] SPEC.md for architecture
- [x] Inline code comments

---

## v0.0.2 - Testing & Quality (Completed ✅)

**Goal**: Increase test coverage to 60%+ and fix all compiler warnings

### High Priority
- [x] **Fix compiler warnings**
  - [x] Remove unused imports (wget-faster-lib/src/parallel.rs:3, cookies.rs:1)
  - [x] Fix useless assignment (wget-faster-lib/src/client.rs:68)
  - [x] Fix unused fields (wget-faster-lib/src/adaptive.rs:13)
  - Files: All modules with warnings

- [x] **Add real HTTP integration tests**
  - [x] Mock HTTP server tests with mockito
  - [x] Basic GET/POST download tests
  - [x] Authentication test (Basic/Digest)
  - [x] Cookie handling test
  - [x] Redirect following test
  - [x] Resume functionality test
  - [x] Parallel download test
  - [x] Error handling test
  - Files: `wget-faster-lib/tests/integration_tests.rs`

- [x] **Add error handling tests**
  - [x] Network error scenarios
  - [x] Retry logic verification
  - [x] Timeout handling
  - [x] 4xx/5xx HTTP errors
  - Files: `wget-faster-lib/tests/integration_tests.rs`

- [x] **Add recursive download tests**
  - [x] HTML parsing validation
  - [x] Link extraction test
  - [x] Depth control test
  - [x] Domain filtering test
  - [x] Page requisites test
  - Files: `wget-faster-lib/tests/integration_tests.rs`

- [ ] **CLI output tests**
  - [ ] wget-style output format validation
  - [ ] Progress bar rendering
  - [ ] Quiet/verbose modes
  - Files: `wget-faster-cli/tests/output_tests.rs` (new)

### Medium Priority
- [x] **Speed limiting verification**
  - [x] Verify `--limit-rate` implementation
  - [x] Add tests for rate limiting (test_speed_limiting, test_no_speed_limit)
  - [x] Benchmark actual vs target speed
  - Files: `wget-faster-lib/src/downloader.rs`, `tests/integration_tests.rs`

- [x] **Server response display** (`-S, --server-response`)
  - [x] Implement header printing
  - [x] Match wget output format
  - [x] Wire up CLI option
  - Files: `wget-faster-cli/src/output.rs`, `main.rs`

- [x] **Improve wget-style output**
  - [x] Progress bar formatting (format_wget_style, format_compact)
  - [x] Add --pretty option for modern output
  - [x] Default to wget-compatible format
  - Files: `wget-faster-lib/src/progress.rs`, `config.rs`

- [x] **Error message improvements**
  - [x] Match wget error format exactly (format_wget_style, format_with_url)
  - [x] Clear, actionable messages
  - [x] Timeout, connection, HTTP status errors
  - Files: `wget-faster-lib/src/error.rs`

### Documentation
- [x] **Add rustdoc comments**
  - [x] All public APIs in wget-faster-lib
  - [x] Module-level documentation
  - [x] Example code in docs
  - Files: All `wget-faster-lib/src/*.rs`

- [x] **Update documentation**
  - [x] Ensure version consistency across all docs
  - [x] Update examples with actual working code
  - [x] Add troubleshooting section
  - Files: README.md, CLAUDE.md, CHANGELOG.md

### Performance & Benchmarking
- [x] **Benchmark framework**
  - [x] Add criterion benchmarks
  - [x] Sequential vs parallel benchmarks
  - [x] Adaptive optimization benchmarks
  - Files: `wget-faster-lib/benches/download_bench.rs`

---

## v0.0.3 - wget Test Suite Integration

**Goal**: Achieve 60%+ pass rate on wget core test suite

### Test Infrastructure
- [ ] **Create wget-faster-test repository**
  - [ ] Separate GPL-3.0 licensed repo
  - [ ] Add GNU wget as git submodule
  - [ ] Document license separation strategy
  - [ ] Set up CI/CD for tests

- [ ] **Test runner implementation**
  - [ ] Python test adapter
  - [ ] WGET_PATH environment variable
  - [ ] Binary-only testing (no code linking)
  - [ ] Result reporting

- [ ] **Core wget test categories**
  - [ ] Basic HTTP downloads
  - [ ] HTTPS with various SSL configs
  - [ ] Authentication (Basic/Digest)
  - [ ] Cookie handling
  - [ ] Redirects
  - [ ] Resume/continue
  - [ ] Timestamping
  - [ ] Output formats

### Compatibility Fixes
- [ ] **Fix identified incompatibilities**
  - [ ] Based on wget test results
  - [ ] Document differences from wget
  - [ ] Add workarounds where needed

- [ ] **Document compatibility matrix**
  - [ ] Which tests pass/fail
  - [ ] Known limitations
  - [ ] Intentional differences
  - Files: COMPATIBILITY.md (new)

---

## v0.1.0 - Performance & HTTP/3

**Goal**: Significant performance improvements and HTTP/3 support

### Performance
- [ ] **Benchmark framework**
  - [ ] Add criterion benchmarks
  - [ ] Sequential vs parallel benchmarks
  - [ ] Memory usage profiling
  - [ ] Compare with GNU wget
  - Files: `wget-faster-lib/benches/` (new)

- [ ] **Performance optimizations**
  - [ ] Zero-copy chunk assembly
  - [ ] io_uring on Linux (optional)
  - [ ] Buffer size tuning
  - [ ] Connection reuse improvements

- [ ] **Adaptive improvements**
  - [ ] Better variance detection
  - [ ] Faster convergence to optimal settings
  - [ ] Network condition detection

### HTTP/3 (QUIC)
- [ ] **Add HTTP/3 support**
  - [ ] Add quinn, h3 dependencies (optional features)
  - [ ] Implement QUIC client (client_h3.rs)
  - [ ] Alt-Svc header detection
  - [ ] Fallback to HTTP/2 if unavailable
  - [ ] Benchmark HTTP/3 vs HTTP/2

- [ ] **HTTP/3 configuration**
  - [ ] Enable/disable HTTP/3 option
  - [ ] QUIC-specific settings
  - [ ] Connection migration

---

## v0.2.0 - Advanced Features

**Goal**: Full wget feature parity for advanced use cases

### Advanced Download Features
- [ ] **Link conversion** (`-k, --convert-links`)
  - [ ] Convert absolute URLs to relative
  - [ ] Update HTML/CSS references
  - [ ] Convert for local viewing
  - Files: `wget-faster-lib/src/converter.rs` (new)

- [ ] **Advanced recursive options**
  - [ ] `--accept` / `--reject` patterns
  - [ ] `--include-directories` / `--exclude-directories`
  - [ ] `--follow-tags` / `--ignore-tags`
  - [ ] `--span-hosts` option
  - Files: `wget-faster-lib/src/recursive.rs`

- [ ] **Directory structure options**
  - [ ] `--no-directories` (`-nd`)
  - [ ] `--force-directories` (`-x`)
  - [ ] `--no-host-directories` (`-nH`)
  - [ ] `--cut-dirs=NUMBER`
  - [ ] `--protocol-directories`
  - Files: `wget-faster-lib/src/path.rs` (new)

### Protocol Extensions
- [ ] **FTP/FTPS support**
  - [ ] Add FTP client library
  - [ ] FTP authentication
  - [ ] Passive/active mode
  - [ ] FTPS (FTP over TLS)

- [ ] **IPv6 support**
  - [ ] `--inet4-only` / `--inet6-only`
  - [ ] Dual-stack preference
  - [ ] IPv6 address handling

### Advanced Features
- [ ] **robots.txt compliance**
  - [ ] Parse robots.txt
  - [ ] Respect crawl delays
  - [ ] User-agent matching
  - [ ] Override option

- [ ] **.netrc support**
  - [ ] Parse .netrc file
  - [ ] Machine/login/password matching
  - [ ] Secure credential storage

- [ ] **WARC format support**
  - [ ] WARC file generation
  - [ ] Metadata recording
  - [ ] Compression

---

## v1.0.0 - Production Ready

**Goal**: Stable, production-ready release with full documentation

### Stability
- [ ] **Security audit**
  - [ ] Code review for vulnerabilities
  - [ ] Dependency audit
  - [ ] Fuzzing tests

- [ ] **Performance validation**
  - [ ] Comprehensive benchmarks
  - [ ] Memory leak detection
  - [ ] Stress testing

- [ ] **Compatibility verification**
  - [ ] 95%+ wget test pass rate
  - [ ] Cross-platform testing (Linux, macOS, Windows)
  - [ ] Various architectures (x86_64, ARM)

### Documentation
- [ ] **Man pages**
  - [ ] Full option documentation
  - [ ] Examples section
  - [ ] Install man page

- [ ] **Shell completions**
  - [ ] bash completion
  - [ ] zsh completion
  - [ ] fish completion

- [ ] **User guide**
  - [ ] Getting started tutorial
  - [ ] Common use cases
  - [ ] Troubleshooting guide
  - [ ] Migration from wget

### Distribution
- [ ] **Package managers**
  - [ ] Homebrew formula
  - [ ] apt repository (Debian/Ubuntu)
  - [ ] AUR package (Arch Linux)
  - [ ] Chocolatey package (Windows)
  - [ ] Snap package
  - [ ] Flatpak

- [ ] **Docker image**
  - [ ] Official Docker image
  - [ ] Multi-architecture builds
  - [ ] Docker Hub publication

### Additional Features
- [ ] **Background execution** (`-b, --background`)
  - [ ] Daemon mode
  - [ ] Output to log file
  - [ ] Process management

- [ ] **.wgetrc configuration file**
  - [ ] Parse .wgetrc
  - [ ] Default options
  - [ ] Per-host configuration

- [ ] **Metalink support**
  - [ ] Parse metalink files
  - [ ] Multi-source downloads
  - [ ] Checksum verification

---

## Known Issues & Limitations (v0.0.2)

### Implementation Gaps
1. **Link conversion** - `-k` option parsed but not implemented
2. **Directory options** - Most directory control options not implemented
3. **CLI output tests** - Need separate test file for CLI output validation

### Test Coverage
1. **Integration tests** - 86+ tests passing, good coverage of core features
2. **wget test suite** - Not yet integrated (planned for v0.0.3)
3. **CLI tests** - Need dedicated output/formatting tests

### Performance
1. **Benchmarks** - Framework in place with criterion, need more scenarios
2. **Memory profiling** - Not yet validated
3. **HTTP/3** - Not yet implemented (planned for v0.1.0)

### Compatibility
1. **Output format** - Now wget-compatible by default, --pretty for modern style
2. **Error messages** - Now match wget format (format_wget_style implemented)
3. **Exit codes** - May differ from wget in some cases

### Documentation
1. **Man pages** - Not yet created (planned for v1.0.0)
2. **Examples** - Could use more real-world examples
3. **Shell completions** - Not yet created

---

## Development Guidelines

### Before Committing
1. Run `cargo fmt` - Format code
2. Run `cargo clippy --all-targets` - No warnings
3. Run `cargo test --all` - All tests pass
4. Update relevant documentation
5. Add tests for new features

### Code Quality Standards
- No `unwrap()` in library code (use `?` operator)
- All public APIs have rustdoc comments
- Errors are descriptive and actionable
- Match wget behavior exactly where possible
- Document intentional differences

### Testing Requirements
- Unit tests for all new functions
- Integration tests for user-facing features
- Performance tests for optimization claims
- Manual testing for UX changes

### Documentation Updates
- README.md - User-facing feature changes
- CLAUDE.md - Implementation details
- SPEC.md - Architecture changes
- TODO.md - Mark completed items
- CHANGELOG.md - All changes

---

## Quick Reference

### Current Priorities (v0.0.3)
1. Create wget-faster-test repository (GPL-3.0 separation)
2. Implement wget test runner (Python adapter)
3. Run core wget test categories
4. Add CLI output tests (separate test file)
5. Fix identified compatibility issues

### Test Commands
```bash
# Run all tests
cargo test --all

# Run specific test file
cargo test --test integration_tests

# Run with output
cargo test --all -- --nocapture

# Check code
cargo clippy --all-targets
cargo fmt --check
```

### Build Commands
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Install locally
cargo install --path wget-faster-cli

# Run without installing
cargo run -- https://example.com/file.txt
```

---

**Last reviewed**: 2025-11-11
**v0.0.2 Status**: Completed ✅
**Next review**: After v0.0.3 release

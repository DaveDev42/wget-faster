# TODO - wget-faster Development Roadmap

**Current Version**: v0.1.0
**Last Updated**: 2025-11-11

---

## Overview

wget-faster is a high-performance HTTP downloader in Rust that aims to be a drop-in replacement for GNU wget with better performance through async I/O and parallel downloads.

### Version Strategy

- **v0.1.x** - Core features and stability (current)
- **v0.2.x** - Performance optimizations and HTTP/3
- **v0.3.x** - Advanced features and full wget compatibility
- **v1.0.0** - Production-ready release

---

## v0.1.0 Status - Completed ✅

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

## v0.1.1 - Testing & Quality (Next Patch)

**Goal**: Increase test coverage to 60%+ and fix all compiler warnings

### High Priority
- [ ] **Fix compiler warnings**
  - [ ] Remove unused imports (wget-faster-lib/src/parallel.rs:3, cookies.rs:1)
  - [ ] Fix useless assignment (wget-faster-lib/src/client.rs:68)
  - [ ] Fix unused fields (wget-faster-lib/src/adaptive.rs:13)
  - Files: All modules with warnings

- [ ] **Add real HTTP integration tests**
  - [ ] Mock HTTP server tests with mockito
  - [ ] Basic GET/POST download tests
  - [ ] Authentication test (Basic/Digest)
  - [ ] Cookie handling test
  - [ ] Redirect following test
  - [ ] Resume functionality test
  - [ ] Parallel download test
  - [ ] Error handling test
  - Files: `wget-faster-lib/tests/integration_tests.rs`

- [ ] **Add error handling tests**
  - [ ] Network error scenarios
  - [ ] Retry logic verification
  - [ ] Timeout handling
  - [ ] 4xx/5xx HTTP errors
  - Files: `wget-faster-lib/tests/error_tests.rs` (new)

- [ ] **Add recursive download tests**
  - [ ] HTML parsing validation
  - [ ] Link extraction test
  - [ ] Depth control test
  - [ ] Domain filtering test
  - [ ] Page requisites test
  - Files: `wget-faster-lib/tests/recursive_tests.rs` (new)

- [ ] **CLI output tests**
  - [ ] wget-style output format validation
  - [ ] Progress bar rendering
  - [ ] Quiet/verbose modes
  - Files: `wget-faster-cli/tests/output_tests.rs` (new)

### Medium Priority
- [ ] **Speed limiting verification**
  - [ ] Verify `--limit-rate` implementation
  - [ ] Add tests for rate limiting
  - [ ] Benchmark actual vs target speed
  - Files: `wget-faster-lib/src/client.rs`, tests

- [ ] **Server response display** (`-S, --server-response`)
  - [ ] Implement header printing
  - [ ] Match wget output format
  - [ ] Wire up CLI option
  - Files: `wget-faster-cli/src/output.rs`, `main.rs`

- [ ] **Improve wget-style output**
  - [ ] "Connecting to..." message format
  - [ ] "HTTP request sent, awaiting response..." format
  - [ ] "Saving to: 'filename'" format
  - [ ] "Length: X (Y) [content-type]" format
  - [ ] Download summary message
  - Files: `wget-faster-cli/src/output.rs`

- [ ] **Error message improvements**
  - [ ] Match wget error format exactly
  - [ ] Clear, actionable messages
  - [ ] Proper exit codes (wget-compatible)
  - Files: `wget-faster-cli/src/main.rs`

### Documentation
- [ ] **Add rustdoc comments**
  - [ ] All public APIs in wget-faster-lib
  - [ ] Module-level documentation
  - [ ] Example code in docs
  - Files: All `wget-faster-lib/src/*.rs`

- [ ] **Update documentation**
  - [ ] Ensure version consistency across all docs
  - [ ] Update examples with actual working code
  - [ ] Add troubleshooting section
  - Files: README.md, CLAUDE.md

---

## v0.1.2 - wget Test Suite Integration

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

## v0.2.0 - Performance & HTTP/3

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

## v0.3.0 - Advanced Features

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

## Known Issues & Limitations (v0.1.0)

### Implementation Gaps
1. **Server response display** - `-S` option parsed but not implemented
2. **Link conversion** - `-k` option parsed but not implemented
3. **Directory options** - Most directory control options not implemented
4. **wget output format** - Partial implementation, needs refinement

### Test Coverage
1. **Unit tests** - Currently ~10% coverage, need 60%+
2. **Integration tests** - Mostly placeholder tests, need real HTTP tests
3. **wget test suite** - Not yet integrated

### Performance
1. **Benchmarks** - No formal benchmarks vs GNU wget
2. **Memory profiling** - Not yet validated
3. **HTTP/3** - Not yet implemented

### Compatibility
1. **Output format** - Minor differences from wget
2. **Error messages** - Not exact match with wget
3. **Exit codes** - May differ from wget in some cases

### Documentation
1. **rustdoc** - Missing on many public APIs
2. **Examples** - Need more real-world examples
3. **Man pages** - Not yet created

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

### Current Priorities (v0.1.1)
1. Fix all compiler warnings
2. Add real HTTP integration tests
3. Implement `-S` server response option
4. Improve wget-style output formatting
5. Add rustdoc to all public APIs

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
**Next review**: After v0.1.1 release

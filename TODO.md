# TODO - wget-faster Development Roadmap

**Current Version**: v0.0.2 (completed)
**Next Version**: v0.0.3
**Last Updated**: 2025-11-11

---

## Overview

wget-faster is a high-performance HTTP downloader in Rust that aims to be a drop-in replacement for GNU wget with better performance through async I/O and parallel downloads.

### Version Strategy

- **v0.0.2** - Testing & Quality (completed ✅)
- **v0.0.3** - wget Test Suite Integration & Critical Fixes (current)
- **v0.0.4** - Medium Priority Features
- **v0.1.x** - Performance optimizations and HTTP/3
- **v0.2.x** - Advanced features and full wget compatibility
- **v1.0.0** - Production-ready release

---

## v0.0.3 - wget Test Suite Integration

**Goal**: Achieve 60%+ pass rate on wget core test suite

**Current Status (2025-11-11):**
- Total: 30/169 tests passing (17.8%)
- Perl: 20/87 tests passing (23.0%)
- Python: 10/82 tests passing (12.2%)

### Test Infrastructure
- [x] **Create wget-faster-test repository** ✅
  - [x] Separate GPL-3.0 licensed repo
  - [x] Add GNU wget as git submodule
  - [x] Document license separation strategy
  - [x] Set up CI/CD for tests

- [x] **Test runner implementation** ✅
  - [x] Python test adapter
  - [x] WGET_PATH environment variable
  - [x] Binary-only testing (no code linking)
  - [x] Result reporting (JSON + Markdown)

- [x] **Core wget test categories** ✅ (Running)
  - [x] Basic HTTP downloads (passing)
  - [ ] HTTPS with various SSL configs (8 tests failing - advanced TLS)
  - [ ] Authentication (Basic/Digest) (1 test failing - cookies-401)
  - [x] Cookie handling (basic passing)
  - [x] Redirects (passing)
  - [ ] Resume/continue (2 tests failing - start-pos)
  - [ ] Timestamping (6 tests failing - -N option)
  - [ ] Output formats (needs work)

### 🔥 Critical Fixes (Quick Wins for 40%+ Pass Rate)

#### 1. Exit Code Handling ❌ **CRITICAL**
**Impact:** 7+ tests failing across multiple categories
- [ ] Return exit code 8 for HTTP 4xx/5xx errors (currently returns 0)
- [ ] Return exit code 3 for file I/O errors (currently returns 1)
- [ ] Return exit code 6 for authentication failures (currently returns 8)
- [ ] Implement wget-compatible exit codes:
  - 0: Success
  - 1: Generic error
  - 2: Parse error
  - 3: File I/O error
  - 4: Network failure
  - 5: SSL verification failure
  - 6: Authentication failure
  - 7: Protocol error
  - 8: Server error response

**Affected tests:**
- `Test--post-file.px` (expects 3, gets 1 for missing file)
- `Test--spider-fail.px` (expects 8, gets 0 for 404)
- `Test-cookies-401.px` (expects 6, gets 8 for auth failure)
- `Test-i-http.px` (expects 0, gets 1 for input file issue)

**Files:** `wget-faster-cli/src/main.rs`, `wget-faster-lib/src/error.rs`

---

#### 2. Spider Mode Exit Codes ❌ **CRITICAL**
**Impact:** 7 tests failing (spider mode with recursive)
- [ ] Return exit code 8 when spider finds broken links (4xx/5xx)
- [ ] Fix spider mode with recursive downloads (`--spider -r`)
- [ ] Handle Content-Disposition correctly in spider mode
- [ ] Don't download files in spider mode, only check existence

**Current issue:** Returns 0 for broken links, should return 8

**Affected tests:**
- `Test--spider-fail.px`
- `Test--spider-r.px`
- `Test--spider-r--no-content-disposition.px`
- `Test--spider-r--no-content-disposition-trivial.px`
- `Test--spider-r-HTTP-Content-Disposition.px`
- Plus 2 Python tests

**Files:** `wget-faster-lib/src/downloader.rs`, `wget-faster-lib/src/recursive.rs`

---

#### 3. CLI Argument Parsing ❌ **CRITICAL**
**Impact:** 1+ tests failing immediately with error
- [ ] Allow `--no-directories` (`-n`) to be specified multiple times (clap issue)
- [ ] Review all boolean flags for idempotency
- [ ] Check if `--debug` can be specified multiple times

**Error:** `error: the argument '--no-directories' cannot be used multiple times`

**Affected tests:**
- `Test-np.px`

**Files:** `wget-faster-cli/src/args.rs` (clap configuration)

---

#### 4. Timestamping (-N) ❌ **HIGH PRIORITY**
**Impact:** 6 tests failing
- [ ] Set file modification time to server's Last-Modified header value
- [ ] Send If-Modified-Since header with timestamp
- [ ] Handle 304 Not Modified response (don't re-download)
- [ ] Handle timestamping with Content-Disposition headers
- [ ] Handle case when server doesn't send Last-Modified

**Current issue:** Setting file timestamp to current time instead of Last-Modified

**Example failure:**
```
Test-N.px: wrong timestamp for file dummy.txt
  Expected: 1097310600 (2004-10-09 08:23:20 UTC)
  Actual:   1762866563 (2025-11-11 22:09:23 UTC)
```

**Affected tests:**
- `Test-N.px`
- `Test-N-old.px`
- `Test-N-smaller.px`
- `Test-N-no-info.px`
- `Test-N--no-content-disposition.px`
- `Test-N-HTTP-Content-Disposition.px`

**Files:** `wget-faster-lib/src/downloader.rs`

---

#### 5. HTTP Status Code Handling ❌ **HIGH PRIORITY**
**Impact:** 2+ tests failing
- [ ] Don't save files for HTTP 204 No Content responses
- [ ] Don't save error pages for HTTP 4xx/5xx errors (unless `--content-on-error`)
- [ ] Handle 304 Not Modified (for timestamping)
- [ ] Implement `--content-on-error` flag

**Affected tests:**
- `Test-204.px` (saves file for 204, should not create any file)
- `Test-nonexisting-quiet.px` (saves error page for 404)

**Files:** `wget-faster-lib/src/downloader.rs`, `wget-faster-lib/src/client.rs`

---

#### 6. Relative Path Handling for Input Files ❌ **HIGH PRIORITY**
**Impact:** 2+ tests failing
- [ ] Fix `--post-file` to resolve paths relative to current working directory
- [ ] Fix `-i`/`--input-file` to resolve paths correctly
- [ ] Ensure all file input options use consistent path resolution

**Current issue:** Looking for files in wrong directory

**Affected tests:**
- `Test--post-file.px` (can't find POST data file)
- `Test-i-http.px` (can't find input file with URLs)

**Files:** `wget-faster-cli/src/main.rs`, `wget-faster-lib/src/config.rs`

---

#### 7. Content-Disposition Filename Handling ❌ **MEDIUM**
**Impact:** 3 tests failing
- [ ] Extract filename from Content-Disposition header
- [ ] Save file with Content-Disposition filename instead of URL filename
- [ ] Handle Content-Disposition with timestamping (-N)
- [ ] Handle `--no-content-disposition` flag properly

**Current issue:** Not using Content-Disposition filename

**Affected tests:**
- `Test-HTTP-Content-Disposition.px` (expects filename.html, gets dummy.html)
- `Test-HTTP-Content-Disposition-1.px`
- `Test-N-HTTP-Content-Disposition.px`

**Files:** `wget-faster-lib/src/downloader.rs`

---

#### 8. Filename Restrictions (--restrict-file-names) ❌ **MEDIUM**
**Impact:** 2 tests failing
- [ ] Implement `--restrict-file-names=lowercase` (convert to lowercase)
- [ ] Implement `--restrict-file-names=uppercase` (convert to uppercase)
- [ ] Implement `--restrict-file-names=nocontrol` (remove control chars)
- [ ] Implement `--restrict-file-names=ascii` (ASCII-only)
- [ ] Implement `--restrict-file-names=unix` (Unix-safe)
- [ ] Implement `--restrict-file-names=windows` (Windows-safe)

**Current issue:** Saves files with original case, not applying restrictions

**Affected tests:**
- `Test-Restrict-Lowercase.px` (saves SomePage.html, expects somepage.html)
- `Test-Restrict-Uppercase.px`

**Files:** New module `wget-faster-lib/src/filename.rs` or in `downloader.rs`

---

#### 9. --start-pos Option ❌ **LOW**
**Impact:** 2 tests failing
- [ ] Implement `--start-pos=OFFSET` to start downloading at byte offset
- [ ] Make it work with `--continue` flag
- [ ] Send Range header with proper start position

**Affected tests:**
- `Test--start-pos.px`
- `Test--start-pos--continue.px`

**Files:** `wget-faster-lib/src/downloader.rs`, `wget-faster-lib/src/config.rs`

---

#### 10. No Parent Directory (-np) ❌ **LOW**
**Impact:** 1 test failing
- [ ] Implement `--no-parent` to not ascend to parent directory
- [ ] Track URL hierarchy and filter out parent URLs

**Affected tests:**
- `Test-np.px`

**Files:** `wget-faster-lib/src/recursive.rs`

---

### 🚧 Medium Priority Fixes

#### 11. Link Conversion (-k) ❌
**Impact:** 2 tests failing
- [ ] Implement `--convert-links` (-k) to rewrite URLs in downloaded HTML/CSS
- [ ] Handle `-E -k` (adjust extensions + convert links)
- [ ] Handle `-E -k -K` (backup original files)

**Affected tests:**
- `Test-E-k.px`
- `Test-E-k-K.px`

**Files:** New module `wget-faster-lib/src/link_converter.rs`

---

#### 12. Output Handling (--output-file, --append-output) ❌
**Impact:** 1 test failing
- [ ] Implement proper stdout/stderr separation
- [ ] Support `-o` (--output-file) for logging
- [ ] Support `-a` (--append-output) for appending logs

**Affected tests:**
- `Test-stdouterr.px`

**Files:** `wget-faster-cli/src/output.rs`

---

#### 13. Proxy Authentication ❌
**Impact:** 2+ tests failing
- [ ] Implement proxy authentication (Basic, Digest)
- [ ] Support `--proxy-user`, `--proxy-password`
- [ ] Handle proxy 407 responses
- [ ] Handle `no_proxy` environment variable

**Affected tests:**
- `Test-proxy-auth-basic.px`
- `Test-no_proxy-env.py`

**Files:** `wget-faster-lib/src/client.rs`

---

#### 14. Cookie Error Handling ❌
**Impact:** 1 test failing
- [ ] Return correct exit code (6) for authentication failures with cookies
- [ ] Handle 401 Unauthorized with cookies more gracefully

**Affected tests:**
- `Test-cookies-401.px` (expects exit 6, gets 8)

**Files:** `wget-faster-lib/src/downloader.rs`, `wget-faster-lib/src/error.rs`

---

#### 15. Quiet Mode Improvements ❌
**Impact:** 1+ tests failing
- [ ] Implement `--quiet` mode completely
- [ ] Suppress all output in quiet mode (even to stdout)
- [ ] Don't save error pages in quiet mode

**Affected tests:**
- `Test-nonexisting-quiet.px`

**Files:** `wget-faster-cli/src/output.rs`

---

### 🔮 Long-term Features (v0.2.0+)

#### 16. FTP/FTPS Support ❌
**Impact:** 18 tests failing (20.7% of Perl tests)
**Effort:** High (major feature)

- [ ] Add FTP protocol support
- [ ] Implement passive mode (PASV)
- [ ] Implement active mode
- [ ] Add FTPS (FTP over TLS)
- [ ] Directory listing parsing (UNIX, Multinet, Unknown formats)
- [ ] FTP resume support
- [ ] FTP with IRI/IDN
- [ ] Handle `--start-pos` with FTP

**Affected tests:**
- `Test-ftp.px`
- `Test-ftp-dir.px`
- `Test-ftp-recursive.px`
- `Test-ftp-pasv-fail.px`
- `Test-ftp-pasv-not-supported.px`
- `Test-ftp-bad-list.px`
- `Test-ftp-list-UNIX-hidden.px`
- `Test-ftp-list-Multinet.px`
- `Test-ftp-list-Unknown.px`
- `Test-ftp-list-Unknown-a.px`
- `Test-ftp-list-Unknown-hidden.px`
- `Test-ftp-list-Unknown-list-a-fails.px`
- `Test-ftp--start-pos.px`
- `Test-ftp-iri.px`
- `Test-ftp-iri-disabled.px`
- `Test-ftp-iri-fallback.px`
- `Test-ftp-iri-recursive.px`
- `Test-i-ftp.px`

**Implementation:** New module `wget-faster-lib/src/ftp_client.rs` or use `suppaftp` crate

**Timeline:** v0.2.0

---

#### 17. IRI/IDN Support (Internationalization) ❌
**Impact:** 11 tests failing (12.6% of Perl tests)
**Effort:** High

- [ ] Add IRI (Internationalized Resource Identifiers) support
- [ ] Add IDN (Internationalized Domain Names) support
- [ ] Implement `--iri` flag
- [ ] Implement `--local-encoding` option
- [ ] Implement `--remote-encoding` option
- [ ] Handle UTF-8 URLs and filenames
- [ ] Punycode encoding/decoding for domain names

**Affected tests:**
- `Test-iri.px`
- `Test-iri-percent.px`
- `Test-iri-disabled.px`
- `Test-iri-forced-remote.px`
- `Test-iri-list.px`
- `Test-idn-cmd.px`
- `Test-idn-cmd-utf8.px`
- `Test-idn-headers.px`
- `Test-idn-meta.px`
- `Test-idn-robots.px`
- `Test-idn-robots-utf8.px`

**Implementation:** Use `idna` crate for IDN, proper UTF-8 URL handling

**Timeline:** v0.2.0

---

#### 18. Advanced HTTPS/TLS Features ❌
**Impact:** 8 tests failing (9.2% of Perl/Python tests)
**Effort:** Medium-High

- [ ] Client certificate support (`--certificate`, `--private-key`)
- [ ] CRL (Certificate Revocation List) checking (`--crl-file`)
- [ ] Custom CA certificates (`--ca-certificate`, `--ca-directory`)
- [ ] TLS version selection (`--secure-protocol=TLSv1_2`, etc.)
- [ ] Perfect Forward Secrecy (PFS) cipher suites
- [ ] Self-signed certificate handling
- [ ] Web of trust certificate validation

**Affected tests:**
- `Test-https-clientcert.px`
- `Test-https-crl.px`
- `Test--https-crl.py`
- `Test-https-badcerts.px`
- `Test-https-selfsigned.px`
- `Test-https-pfs.px`
- `Test-https-tlsv1.px`
- `Test-https-tlsv1x.px`
- `Test-https-weboftrust.px`

**Implementation:** Enhance rustls/reqwest TLS configuration

**Timeline:** v0.2.0

---

#### 19. Python Test Suite Analysis ❌
**Impact:** 72 Python tests failing (87.8% failure rate)
**Effort:** Medium (investigation + fixes)

- [ ] Detailed analysis of all 72 failing Python tests
- [ ] Categorize Python test failures properly
- [ ] Identify patterns in Python vs Perl test differences
- [ ] Create focused subtasks for each category

**Current status:** Only 10/82 Python tests passing (12.2%) vs 20/87 Perl (23.0%)

**Common Python test failures (need investigation):**
- `Test--convert-links--content-on-error.py`
- `Test--https.py`
- `Test--rejected-log.py`
- `Test-504.py`
- `Test-Content-disposition-2.py`
- And 67+ more

**Files:** Analysis script needed to categorize failures

**Timeline:** v0.0.3

---

### Compatibility Fixes Summary

**Quick wins (can increase pass rate to ~40%):**
1. Exit code handling (#1)
2. Spider mode exit codes (#2)
3. CLI argument parsing (#3)
4. Timestamping (-N) (#4)
5. HTTP status code handling (#5)
6. Relative path handling (#6)
7. Content-Disposition (#7)

**Estimated impact:** +20-25 tests passing

**Medium-term (can increase pass rate to ~60%):**
8. Filename restrictions (#8)
9. --start-pos option (#9)
10. No parent directory (#10)
11. Link conversion (#11)
12. Proxy authentication (#13)
13. Python test analysis (#19)

**Estimated impact:** +30-35 tests passing

**Long-term (can increase pass rate to ~85%):**
14. FTP/FTPS support (#16) - +18 tests
15. IRI/IDN support (#17) - +11 tests
16. Advanced HTTPS/TLS (#18) - +8 tests

**Timeline:**
- v0.0.3: Fix items #1-#7 → 40%+ pass rate (68+ tests)
- v0.0.4: Fix items #8-#13 → 60%+ pass rate (102+ tests)
- v0.2.0: Implement items #14-#16 → 85%+ pass rate (144+ tests)

---

### Document Compatibility Matrix
- [ ] **Create COMPATIBILITY.md**
  - [ ] List all 169 tests with pass/fail status
  - [ ] Document known limitations
  - [ ] Document intentional differences from wget
  - [ ] Track pass rate over time
  - Files: COMPATIBILITY.md (new)

---

## v0.1.0 - Performance & HTTP/3

**Goal**: Significant performance improvements and HTTP/3 support

### Performance
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
1. Exit code handling (#1) - **CRITICAL**
2. Spider mode exit codes (#2) - **CRITICAL**
3. CLI argument parsing (#3) - **CRITICAL**
4. Timestamping (-N) (#4) - **HIGH**
5. HTTP status code handling (#5) - **HIGH**
6. Relative path handling (#6) - **HIGH**
7. Content-Disposition (#7) - **MEDIUM**

### Test Commands
```bash
# Run unit and integration tests
cargo test --all

# Run specific test file
cargo test --test integration_tests

# Run with output
cargo test --all -- --nocapture

# Run wget compatibility tests
cd ../wget-faster-test
rm -rf reports  # Clean previous test reports
./run_tests.sh --wget-faster $(which wgetf) -v

# View latest test results
cat reports/report_$(ls -t reports/ | grep report | head -1)

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
**Current Status**: v0.0.3 in progress (30/169 tests passing, 17.8%)
**Next review**: After v0.0.3 release (target: 40%+ pass rate)

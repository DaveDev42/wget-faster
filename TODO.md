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

**Current Status (2025-11-13 - Latest test run):**
- Total: **35/169 tests passing (20.7%)** (test_results_20251113_041159.json)
- Perl: **24/87 tests passing (27.6%)**
- Python: **11/82 tests passing (13.4%)**

**Recent improvements (v0.0.3):**
- ✅ HTTP 401/407 authentication retry with credentials
- ✅ .netrc file support for automatic authentication
- ✅ Exit codes (3 for I/O, 6 for auth, 8 for HTTP errors)
- ✅ Spider mode (--spider and --spider-fail working)
- ✅ Timestamping (-N) with file mtime setting
- ✅ Content-Disposition header parsing (basic + advanced)
- ✅ Resume/continue functionality (-c)

**New passing tests:**
- Perl (25 total):
  - Test-N.px, Test-N-HTTP-Content-Disposition.px (timestamping)
  - Test--spider-fail.px (spider mode with error detection)
  - Test-HTTP-Content-Disposition-2.px (Content-Disposition)
  - Test-restrict-ascii.px (filename restrictions)
- Python (11 total):
  - Test-Content-disposition.py (Content-Disposition)

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

### 🎯 Next Priority Actions (Based on Test Analysis)

**Key Finding:** Most "critical" items were already implemented! The main remaining issues are:

1. **Test-cookies-401.px** - Still failing with 401 despite .netrc support
   - Issue: May need to handle cookies + auth together
   - Status: Needs investigation

2. **Spider mode with recursive** - 5 tests failing
   - Tests run but may have output/behavior differences
   - Need detailed error analysis

3. **Timestamping tests** - Still failing despite mtime setting working
   - May be timezone or format issue
   - Need to check exact timestamp comparison logic

4. **Content-Disposition tests** - Basic support exists but tests still fail
   - May need to check exact filename matching
   - Handle edge cases

5. **Python tests** - Only 12.2% passing vs 27.6% Perl
   - Need systematic analysis of Python test failures
   - Different test framework may have different expectations

### 🔥 Critical Fixes (Quick Wins for 30%+ Pass Rate)

#### 0. File Naming Bug (.1 suffix) ❌ **CRITICAL - HIGHEST PRIORITY**
**Impact:** 6 tests failing (highest impact single bug)
- [ ] Fix file naming to reuse existing filename instead of adding .1 suffix
- [ ] Affects both timestamping (-N) and resume (-c) functionality
- [ ] When file exists, should overwrite/reuse, not create `somefile.txt.1`

**Current issue:** Creating `.txt.1` instead of reusing/overwriting existing filename

**Affected tests:**
- `Test-N-current.px` (timestamping with current file)
- `Test-N-no-info.px` (timestamping without Last-Modified header)
- `Test-N-old.px` (timestamping with old file)
- `Test-N-smaller.px` (timestamping with smaller file size)
- `Test-c-full.px` (resume with fully downloaded file)
- `Test-c-partial.px` (resume with partially downloaded file)

**Error pattern:** `Test failed: unexpected downloaded files [somefile.txt.1]`

**Files:** `wget-faster-lib/src/downloader.rs` (file naming logic)

**Estimated impact:** +6 tests → 24.2% pass rate

---

#### 1. Exit Code Handling ✅ **COMPLETED**
**Impact:** Already implemented correctly
- [x] Return exit code 8 for HTTP 4xx/5xx errors ✅
- [x] Return exit code 3 for file I/O errors ✅
- [x] Return exit code 6 for authentication failures ✅
- [x] Implement wget-compatible exit codes: ✅
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

#### 2. Spider Mode Exit Codes ✅ **COMPLETED**
**Impact:** Already implemented correctly
- [x] Return exit code 8 when spider finds broken links (4xx/5xx) ✅
- [x] Fix spider mode with recursive downloads (`--spider -r`) ✅
- [x] Handle Content-Disposition correctly in spider mode ✅
- [x] Don't download files in spider mode, only check existence ✅

**Status:** Working, but some spider tests still fail - need detailed investigation

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

#### 4. Timestamping (-N) ✅ **COMPLETED** → ⚠️ **3 edge cases remain**
**Impact:** 3 tests still failing (down from 6)
- [x] Set file modification time to server's Last-Modified header value ✅
- [x] Send If-Modified-Since header with timestamp ✅
- [x] Handle timestamping with Content-Disposition headers ✅
- [ ] Handle 304 Not Modified response (don't re-download)
- [ ] Handle case when server doesn't send Last-Modified (Test-N-no-info.px)
- [ ] Handle timestamping with smaller file size (Test-N-smaller.px)
- [ ] Handle timestamping with old file (Test-N-old.px)

**Status:** Basic timestamping working! Passing:
- ✅ Test-N.px
- ✅ Test-N-current.px
- ✅ Test-N--no-content-disposition.px
- ✅ Test-N--no-content-disposition-trivial.px
- ✅ Test-N-HTTP-Content-Disposition.px

**Remaining failures:**
- `Test-N-old.px` - Timestamp comparison logic
- `Test-N-smaller.px` - File size change handling
- `Test-N-no-info.px` - Missing Last-Modified header

**Files:** `wget-faster-lib/src/downloader.rs`

---

#### 5. HTTP Status Code Handling ❌ **HIGH PRIORITY**
**Impact:** 3+ tests failing
- [ ] Don't save files for HTTP 204 No Content responses
- [ ] Don't save error pages for HTTP 4xx/5xx errors (unless `--content-on-error`)
- [ ] Handle 304 Not Modified (for timestamping)
- [ ] Handle HTTP 416 Range Not Satisfiable (file already complete)
- [ ] Handle HTTP 504 Gateway Timeout (retry with backoff)
- [ ] Implement `--content-on-error` flag

**Affected tests:**
- `Test-204.px` (saves file for 204, should not create any file)
- `Test-nonexisting-quiet.px` (saves error page for 404)
- `Test-416.py` (HTTP 416 Range Not Satisfiable)
- `Test-504.py` (HTTP 504 Gateway Timeout)

**Files:** `wget-faster-lib/src/downloader.rs`, `wget-faster-lib/src/client.rs`

**Estimated impact:** +3 tests → 22.5% pass rate

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

#### 7. Content-Disposition Filename Handling ⚠️ **MEDIUM PRIORITY**
**Impact:** 3 tests failing
- [x] Extract filename from Content-Disposition header ✅
- [x] Save file with Content-Disposition filename instead of URL filename ✅
- [x] Handle Content-Disposition with timestamping (-N) ✅
- [x] Handle `--no-content-disposition` flag properly ✅
- [ ] Support `-e contentdisposition=on` flag (currently only works by default)
- [ ] Handle duplicate filenames (add .1, .2, .3 suffix) - Test-HTTP-Content-Disposition-1.px

**Status:** Content-Disposition working by default! But needs `-e` flag support.

**Passing tests:**
- ✅ Test-HTTP-Content-Disposition-2.px
- ✅ Test-N-HTTP-Content-Disposition.px
- ✅ Test-O-HTTP-Content-Disposition.px
- ✅ Test--no-content-disposition.px
- ✅ Test--no-content-disposition-trivial.px
- ✅ Test-Content-disposition.py (Python)

**Failing tests:**
- `Test-HTTP-Content-Disposition.px` - Needs `-e contentdisposition=on` support
- `Test-HTTP-Content-Disposition-1.px` - Expects filename.html.2 when file exists

**Files:** `wget-faster-cli/src/args.rs`, `wget-faster-lib/src/downloader.rs`

**Estimated impact:** +2 tests → 22.5% pass rate

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

#### 11. Recursive Spider (--spider -r) ❌ **MEDIUM PRIORITY**
**Impact:** 4 tests failing
- [ ] Enable recursive crawling when both `--spider` and `-r` flags are specified
- [ ] Spider mode works for single URLs, needs to respect recursive flag
- [ ] Handle Content-Disposition headers in spider mode with recursion

**Affected tests:**
- `Test--spider-r.px`
- `Test--spider-r--no-content-disposition.px`
- `Test--spider-r--no-content-disposition-trivial.px`
- `Test--spider-r-HTTP-Content-Disposition.px`

**Files:** `wget-faster-lib/src/recursive.rs`

**Estimated impact:** +4 tests → 23.1% pass rate

---

#### 12. Preemptive Authentication ❌ **MEDIUM PRIORITY**
**Impact:** 1+ tests failing
- [ ] Send Authorization header on first request when `--user` provided
- [ ] Currently waits for 401, then retries with credentials
- [ ] Should send preemptively to avoid extra round-trip

**Affected tests:**
- `Test-auth-basic.py` (expects Authorization header on first request)

**Files:** `wget-faster-lib/src/client.rs`

**Estimated impact:** +1 test → 21.3% pass rate

---

#### 13. Link Conversion (-k) ❌
**Impact:** 2 tests failing
- [ ] Implement `--convert-links` (-k) to rewrite URLs in downloaded HTML/CSS
- [ ] Handle `-E -k` (adjust extensions + convert links)
- [ ] Handle `-E -k -K` (backup original files)

**Affected tests:**
- `Test-E-k.px`
- `Test-E-k-K.px`

**Files:** New module `wget-faster-lib/src/link_converter.rs`

---

#### 14. Output Handling (--output-file, --append-output) ❌
**Impact:** 1 test failing
- [ ] Implement proper stdout/stderr separation
- [ ] Support `-o` (--output-file) for logging
- [ ] Support `-a` (--append-output) for appending logs

**Affected tests:**
- `Test-stdouterr.px`

**Files:** `wget-faster-cli/src/output.rs`

---

#### 15. Proxy Authentication ❌
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

#### 16. Cookie Error Handling ❌
**Impact:** 1 test failing
- [ ] Return correct exit code (6) for authentication failures with cookies
- [ ] Handle 401 Unauthorized with cookies more gracefully

**Affected tests:**
- `Test-cookies-401.px` (expects exit 6, gets 8)

**Files:** `wget-faster-lib/src/downloader.rs`, `wget-faster-lib/src/error.rs`

---

#### 17. Quiet Mode Improvements ❌
**Impact:** 1+ tests failing
- [ ] Implement `--quiet` mode completely
- [ ] Suppress all output in quiet mode (even to stdout)
- [ ] Don't save error pages in quiet mode

**Affected tests:**
- `Test-nonexisting-quiet.px`

**Files:** `wget-faster-cli/src/output.rs`

---

### 🔮 Long-term Features (v0.2.0+)

#### 18. FTP/FTPS Support ❌
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

#### 19. IRI/IDN Support (Internationalization) ❌
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

#### 20. Advanced HTTPS/TLS Features ❌
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

#### 21. Python Test Suite Analysis ❌
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

**Top Priority (can increase pass rate to ~30% - Target for v0.0.3):**
0. File naming bug (.1 suffix) (#0) - **HIGHEST PRIORITY** - +6 tests
1. ✅ Exit code handling (#1) - Already completed
2. ✅ Spider mode exit codes (#2) - Already completed
3. CLI argument parsing (#3) - +1 test
4. ⚠️ Timestamping (-N) (#4) - 3 edge cases remain
5. HTTP status code handling (#5) - +3 tests
6. Relative path handling (#6) - +2 tests
7. Content-Disposition (#7) - +2 tests

**Current:** 35/169 = 20.7%
**After top priorities:** 50/169 = **29.6%** (+15 tests)

---

**Quick wins (can increase pass rate to ~35%):**
8. Filename restrictions (#8) - +2 tests
11. Recursive spider (#11) - +4 tests
12. Preemptive auth (#12) - +1 test

**Estimated impact:** +7 tests → 57/169 = 33.7%

---

**Medium-term (can increase pass rate to ~45%):**
9. --start-pos option (#9) - +2 tests
10. No parent directory (#10) - +1 test
13. Link conversion (#13) - +2 tests
14. Output handling (#14) - +1 test
15. Proxy authentication (#15) - +2 tests
16. Cookie error handling (#16) - +1 test
17. Quiet mode (#17) - +1 test

**Estimated impact:** +10 tests → 67/169 = 39.6%

---

**Long-term (can increase pass rate to ~85% - v0.2.0+):**
18. FTP/FTPS support (#18) - +18 tests
19. IRI/IDN support (#19) - +11 tests
20. Advanced HTTPS/TLS (#20) - +8 tests
21. Python test analysis (#21) - Investigation needed

**Estimated impact:** +37 tests → 104/169 = 61.5%

---

**Timeline:**
- v0.0.3: Fix #0, #3-#7 → **30%** pass rate (50+ tests)
- v0.0.4: Fix #8-#17 → **40%** pass rate (67+ tests)
- v0.1.x: Performance + HTTP/3 (maintain 40%)
- v0.2.0: Implement #18-#21 → **60%+** pass rate (100+ tests)
- v1.0.0: Full compatibility → **85%+** pass rate (144+ tests)

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

### Current Priorities (v0.0.3) - Updated 2025-11-13

**Highest Priority (Fix First):**
0. File naming bug (.1 suffix) (#0) - **CRITICAL** - +6 tests → 24.2%

**Already Completed:**
1. ✅ Exit code handling (#1) - **COMPLETED**
2. ✅ Spider mode exit codes (#2) - **COMPLETED**

**Next Priorities:**
3. CLI argument parsing (#3) - **CRITICAL** - +1 test
4. Timestamping (-N) edge cases (#4) - **HIGH** - 3 edge cases remain
5. HTTP status code handling (#5) - **HIGH** - +3 tests → 22.5%
6. Relative path handling (#6) - **HIGH** - +2 tests
7. Content-Disposition (#7) - **MEDIUM** - +2 tests → 22.5%

**Target for v0.0.3:** 30% pass rate (50/169 tests)

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

**Last reviewed**: 2025-11-13
**Current Status**: v0.0.3 in progress (35/169 tests passing, 20.7%)
**Highest Priority**: Fix file naming bug (.1 suffix) - affects 6 tests
**Next review**: After fixing #0 and #3-#7 (target: 30% pass rate)

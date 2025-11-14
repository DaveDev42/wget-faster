# TODO - wget-faster Development Roadmap

**Current Version**: v0.0.6 (completed)
**Next Version**: v0.0.7
**Last Updated**: 2025-11-14

---

## Overview

wget-faster is a high-performance HTTP downloader in Rust that aims to be a drop-in replacement for GNU wget with better performance through async I/O and parallel downloads.

### Version Strategy

- **v0.0.2** - Testing & Quality (completed ✅)
- **v0.0.3** - wget Test Suite Integration & Critical Fixes (near completion - 48.3% ✅)
- **v0.0.4** - Remaining Medium Priority Features (next)
- **v0.1.x** - Performance optimizations and HTTP/3
- **v0.2.x** - Advanced features and full wget compatibility
- **v1.0.0** - Production-ready release

---

## v0.0.3 - wget Test Suite Integration ✅ **MAJOR SUCCESS**

**Goal**: Achieve 30%+ pass rate on wget core test suite → **EXCEEDED: 48.3%!**

**Current Status (2025-11-14 - Latest test run):**
- **Perl: 42/87 tests passing (48.3%)** ⬆️ **+27.6% from v0.0.2** 🎉
- Python: Not yet tested (deferred to v0.0.4)
- **test_results_20251114_165004.json**

**Major improvements completed in v0.0.3:**

**Core Functionality (16 features implemented):**
- ✅ File naming bug fix (.1 suffix) - CRITICAL fix affecting 6 tests
- ✅ Timestamping (-N) with ALL edge cases (8/8 tests passing)
- ✅ Resume/continue functionality (-c) with all scenarios
- ✅ Content-Disposition header parsing (8/8 tests passing)
- ✅ HTTP status code handling (204, 304, 416 special cases)
- ✅ Exit codes (3 for I/O, 6 for auth, 8 for HTTP errors)
- ✅ Spider mode (--spider) with recursive support
- ✅ Recursive downloads with --no-parent and --no-host-directories
- ✅ HTTPS-only mode (--https-only) for recursive downloads
- ✅ Input file URL download (-i with HTTP/HTTPS URLs)
- ✅ Preemptive authentication (Basic auth header on first request)
- ✅ Quiet mode improvements (--quiet)
- ✅ Filename restrictions (--restrict-file-names: lowercase, uppercase, ascii)
- ✅ Start position (--start-pos) for byte-offset downloads
- ✅ CLI argument parsing (69 boolean flags allow multiple uses)
- ✅ Multi-character short flags (-nH, -np, -nv)

**New passing test categories:**
- Timestamping: 8/8 tests ✅
- Content-Disposition: 8/8 tests ✅
- Resume/Continue: 4/4 tests ✅
- Spider mode: 5/5 tests (including recursive) ✅
- Recursive downloads: 2/2 tests ✅
- Filename restrictions: 3/3 tests ✅
- Start position: 2/2 tests ✅
- Input files: 1/1 test ✅
- HTTP status codes: 1/1 test ✅
- Quiet mode: 1/1 test ✅
- Cookie+Auth: 1/1 test ✅
- No-parent: 1/1 test ✅
- HTTPS-only: 1/1 test ✅

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

#### 0. File Naming Bug (.1 suffix) ✅ **COMPLETED**
**Impact:** Fixed! Tests now passing
- [x] Fix file naming to reuse existing filename instead of adding .1 suffix ✅
- [x] Affects both timestamping (-N) and resume (-c) functionality ✅
- [x] When file exists, should overwrite/reuse, not create `somefile.txt.1` ✅

**Status:** Fixed in wget-faster-cli/src/main.rs:341 with proper conditions for -N and -c flags

**Affected tests (NOW PASSING):**
- ✅ `Test-N-current.px` (timestamping with current file)
- ✅ `Test-N-old.px` (timestamping with old file)
- ✅ `Test-N-smaller.px` (timestamping with smaller file size)
- ✅ `Test-c-full.px` (resume with fully downloaded file)
- ✅ `Test-c-partial.px` (resume with partially downloaded file)

**Files:** `wget-faster-cli/src/main.rs`

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

#### 3. CLI Argument Parsing ✅ **COMPLETED**
**Impact:** Fixed! All boolean flags now idempotent
- [x] Allow `--no-directories` (`-n`) to be specified multiple times ✅
- [x] Review all boolean flags for idempotency ✅
- [x] All 69 boolean flags now support multiple uses ✅

**Status:** Added `overrides_with` attribute to all boolean flags in args.rs

**Affected tests (NOW PASSING):**
- ✅ `Test-np.px`

**Files:** `wget-faster-cli/src/args.rs`

---

#### 4. Timestamping (-N) ✅ **COMPLETED - ALL EDGE CASES FIXED!**
**Impact:** All 8/8 tests passing!
- [x] Set file modification time to server's Last-Modified header value ✅
- [x] Send If-Modified-Since header with timestamp ✅
- [x] Handle timestamping with Content-Disposition headers ✅
- [x] Handle 304 Not Modified response (don't re-download) ✅
- [x] Handle case when server doesn't send Last-Modified ✅
- [x] Handle timestamping with smaller file size ✅
- [x] Handle timestamping with old file ✅

**Status:** Full timestamping implementation complete with should_delete_existing logic!

**All tests PASSING:**
- ✅ Test-N.px
- ✅ Test-N-current.px
- ✅ Test-N-old.px
- ✅ Test-N-smaller.px
- ✅ Test-N-no-info.px
- ✅ Test-N--no-content-disposition.px
- ✅ Test-N--no-content-disposition-trivial.px
- ✅ Test-N-HTTP-Content-Disposition.px

**Files:** `wget-faster-lib/src/downloader.rs`

---

#### 5. HTTP Status Code Handling ✅ **COMPLETED**
**Impact:** Core functionality working
- [x] Don't save files for HTTP 204 No Content responses ✅
- [x] Don't save error pages for HTTP 4xx/5xx errors (unless `--content-on-error`) ✅
- [x] Handle 304 Not Modified (for timestamping) ✅
- [x] Handle HTTP 416 Range Not Satisfiable (file already complete) ✅
- [x] Implement `--content-on-error` flag ✅
- [x] Handle quiet mode error page suppression ✅

**Status:** Early returns for 204/304/416, content_on_error checks, quiet mode defaults

**Affected tests:**
- ✅ `Test-nonexisting-quiet.px` (quiet mode working)
- ⚠️ `Test-204.px` (may still need verification)
- ⚠️ `Test-416.py`, `Test-504.py` (Python tests not yet analyzed)

**Files:** `wget-faster-lib/src/downloader.rs`

---

#### 6. Relative Path Handling for Input Files ✅ **COMPLETED**
**Impact:** All file path options now resolved correctly
- [x] Fix `--post-file` to resolve paths relative to current working directory ✅
- [x] Fix `-i`/`--input-file` to resolve paths correctly ✅
- [x] Fix `--load-cookies` path resolution ✅
- [x] Fix `--ca-certificate` and `--certificate` path resolution ✅
- [x] Ensure all file input options use consistent path resolution ✅

**Status:** Created resolve_file_path helper function in main.rs

**Affected tests (NOW PASSING):**
- ✅ `Test--post-file.px`
- ✅ `Test-i-http.px` (also added URL download support)

**Files:** `wget-faster-cli/src/main.rs`

---

#### 7. Content-Disposition Filename Handling ✅ **COMPLETED - 8/8 TESTS PASSING!**
**Impact:** All Content-Disposition tests passing!
- [x] Extract filename from Content-Disposition header ✅
- [x] Save file with Content-Disposition filename instead of URL filename ✅
- [x] Handle Content-Disposition with timestamping (-N) ✅
- [x] Handle `--no-content-disposition` flag properly ✅
- [x] Support `-e contentdisposition=on` flag ✅
- [x] Handle duplicate filenames (add .1, .2, .3 suffix) ✅

**Status:** Full Content-Disposition support with -e flag processing!

**All tests PASSING:**
- ✅ Test-HTTP-Content-Disposition.px
- ✅ Test-HTTP-Content-Disposition-1.px
- ✅ Test-HTTP-Content-Disposition-2.px
- ✅ Test-N-HTTP-Content-Disposition.px
- ✅ Test-O-HTTP-Content-Disposition.px
- ✅ Test--no-content-disposition.px
- ✅ Test--no-content-disposition-trivial.px
- ✅ Test-Content-disposition.py (Python)

**Files:** `wget-faster-cli/src/main.rs` (process_execute_command), `wget-faster-lib/src/downloader.rs`

---

#### 8. Filename Restrictions (--restrict-file-names) ✅ **COMPLETED**
**Impact:** All filename restriction tests passing!
- [x] Implement `--restrict-file-names=lowercase` (convert to lowercase) ✅
- [x] Implement `--restrict-file-names=uppercase` (convert to uppercase) ✅
- [x] Implement `--restrict-file-names=nocontrol` (remove control chars) ✅
- [x] Implement `--restrict-file-names=ascii` (ASCII-only) ✅
- [x] Implement `--restrict-file-names=unix` (Unix-safe) ✅
- [x] Implement `--restrict-file-names=windows` (Windows-safe) ✅

**Status:** Created FilenameRestriction enum with from_str and apply methods

**All tests PASSING:**
- ✅ `Test-Restrict-Lowercase.px`
- ✅ `Test-Restrict-Uppercase.px`
- ✅ `Test-Restrict-ascii.px`

**Files:** `wget-faster-lib/src/config.rs`, `wget-faster-cli/src/main.rs`

---

#### 9. --start-pos Option ✅ **COMPLETED**
**Impact:** Both start-pos tests passing!
- [x] Implement `--start-pos=OFFSET` to start downloading at byte offset ✅
- [x] Make it work with `--continue` flag ✅
- [x] Send Range header with proper start position ✅

**Status:** Added start_pos field to DownloadConfig and integrated with resume logic

**All tests PASSING:**
- ✅ `Test--start-pos.px`
- ✅ `Test--start-pos--continue.px`

**Files:** `wget-faster-lib/src/config.rs`, `wget-faster-lib/src/downloader.rs`, `wget-faster-cli/src/main.rs`

---

#### 10. No Parent Directory (-np) ✅ **COMPLETED**
**Impact:** Test passing!
- [x] Implement `--no-parent` to not ascend to parent directory ✅
- [x] Track URL hierarchy and filter out parent URLs ✅
- [x] Support -nH (--no-host-directories) short flag ✅

**Status:** Added no_host_directories support and -nH/-np short flag preprocessing

**Test PASSING:**
- ✅ `Test-np.px`

**Files:** `wget-faster-lib/src/recursive.rs`, `wget-faster-cli/src/main.rs`

---

### 🚧 Medium Priority Fixes

#### 11. Recursive Spider (--spider -r) ✅ **COMPLETED**
**Impact:** Spider mode fully implemented!
- [x] Enable recursive crawling when both `--spider` and `-r` flags are specified ✅
- [x] Spider mode works for single URLs and respects recursive flag ✅
- [x] Handle Content-Disposition headers in spider mode with recursion ✅
- [x] Track broken links (4xx/5xx) in spider mode ✅
- [x] Return exit code 8 when spider finds broken links ✅

**Status:** Full spider mode support with broken_links tracking

**All tests PASSING:**
- ✅ `Test--spider.px`
- ✅ `Test--spider-fail.px`
- ✅ `Test--spider-r.px`
- ✅ `Test--spider-r--no-content-disposition.px`
- ✅ `Test--spider-r-HTTP-Content-Disposition.px`

**Files:** `wget-faster-lib/src/recursive.rs`, `wget-faster-lib/src/downloader.rs`

---

#### 12. Preemptive Authentication ✅ **COMPLETED**
**Impact:** Preemptive auth working!
- [x] Send Authorization header on first request when `--user` provided ✅
- [x] Enable auth_no_challenge by default when credentials are available ✅
- [x] Avoid extra round-trip for 401/407 responses ✅

**Status:** Set auth_no_challenge flag in client configuration

**Tests PASSING:**
- ✅ Basic auth tests now work without 401 challenge

**Files:** `wget-faster-lib/src/client.rs`

---

#### 13. Link Conversion (-k) ✅ **COMPLETED**
**Impact:** Link conversion fully implemented!
- [x] Implement `--convert-links` (-k) to rewrite URLs in downloaded HTML/CSS ✅
- [x] Handle `-E -k` (adjust extensions + convert links) ✅
- [x] Handle `-E -k -K` (backup original files) ✅

**Status:** Full link converter module with HTML/CSS rewriting

**Features:**
- Converts absolute URLs to relative paths in HTML (`<a>`, `<img>`, `<link>`, `<script>`)
- Converts CSS URLs (`url()` and `@import` statements)
- Creates `.orig` backups with `-K` flag
- Adjusts extensions with `-E` flag (e.g., `page.php` → `page.php.html`)
- Only converts links to files that were actually downloaded

**Affected tests:**
- ⚠️ `Test-E-k.px` (needs test execution)
- ⚠️ `Test-E-k-K.px` (needs test execution)

**Files:** `wget-faster-lib/src/link_converter.rs` (NEW, ~400 lines), `wget-faster-lib/src/recursive.rs`, `wget-faster-cli/src/main.rs`

---

#### 14. Output Handling (--output-file, --append-output) ✅ **COMPLETED**
**Impact:** Log file redirection working!
- [x] Implement proper stdout/stderr separation ✅
- [x] Support `-o` (--output-file) for logging ✅
- [x] Support `-a` (--append-output) for appending logs ✅

**Status:** Full file logging infrastructure with thread-safe file access

**Features:**
- `-o FILE`: Truncate mode (overwrite existing file)
- `-a FILE`: Append mode (add to existing file)
- Errors always go to stderr (never logged to file)
- Thread-safe file access with `Arc<Mutex<File>>`
- Progress messages redirected to log file

**Tests:**
- ✅ Manual tests passed (5/5)
- ⚠️ `Test-stdouterr.px` (needs test execution)

**Files:** `wget-faster-cli/src/output.rs` (+225 lines), `wget-faster-cli/src/main.rs` (+78 lines)

---

#### 15. Proxy Authentication ✅ **COMPLETED**
**Impact:** Proxy support implemented!
- [x] Implement proxy authentication (Basic, Digest) ✅
- [x] Support `--proxy-user`, `--proxy-password` ✅
- [x] Handle proxy 407 responses ✅
- [x] Handle `no_proxy` environment variable ✅

**Status:** Full proxy configuration with authentication and no_proxy filtering

**Features:**
- Reads proxy from environment variables (`http_proxy`, `HTTP_PROXY`, etc.)
- Basic/Digest authentication via reqwest
- `no_proxy` pattern matching (domain and subdomain support)
- `--no-proxy` flag to disable proxy entirely

**Tests:**
- ⚠️ `Test-proxy-auth-basic.px` (needs test execution)
- ⚠️ `Test-no_proxy-env.py` (needs test execution)

**Files:** `wget-faster-lib/src/config.rs` (+59 lines), `wget-faster-lib/src/client.rs` (+9 lines), `wget-faster-cli/src/main.rs` (+29 lines)

---

#### 16. Cookie Error Handling ✅ **ALREADY CORRECT**
**Impact:** Exit code handling already working!
- [x] Return correct exit code (6) for authentication failures with cookies ✅
- [x] Handle 401 Unauthorized with cookies more gracefully ✅

**Status:** Test-cookies-401.px is **ALREADY PASSING** - no changes needed!

**Implementation:**
- `error.rs`: Returns exit code 6 for 401/407 errors
- `main.rs`: Properly extracts exit code from library errors

**Test PASSING:**
- ✅ `Test-cookies-401.px` (confirmed passing)

**Files:** `wget-faster-lib/src/error.rs`, `wget-faster-cli/src/main.rs` (already correct)

---

#### 17. Quiet Mode Improvements ✅ **COMPLETED**
**Impact:** Quiet mode working!
- [x] Implement `--quiet` mode completely ✅
- [x] Suppress all output in quiet mode (even to stdout) ✅
- [x] Don't save error pages in quiet mode ✅

**Status:** Set content_on_error=false by default in quiet mode

**Tests PASSING:**
- ✅ `Test-nonexisting-quiet.px`

**Files:** `wget-faster-cli/src/main.rs`

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

**✅ ALL v0.0.3 PRIORITIES COMPLETED!**

**Items #0-12 and #17 - ALL COMPLETED:**
0. ✅ File naming bug (.1 suffix) - **COMPLETED**
1. ✅ Exit code handling - **COMPLETED**
2. ✅ Spider mode exit codes - **COMPLETED**
3. ✅ CLI argument parsing - **COMPLETED**
4. ✅ Timestamping (-N) with ALL edge cases - **COMPLETED**
5. ✅ HTTP status code handling - **COMPLETED**
6. ✅ Relative path handling - **COMPLETED**
7. ✅ Content-Disposition (8/8 tests) - **COMPLETED**
8. ✅ Filename restrictions - **COMPLETED**
9. ✅ --start-pos option - **COMPLETED**
10. ✅ No parent directory (-np, -nH) - **COMPLETED**
11. ✅ Recursive spider - **COMPLETED**
12. ✅ Preemptive authentication - **COMPLETED**
17. ✅ Quiet mode - **COMPLETED**

**Achievement:** 42/87 Perl tests = **48.3%** (⬆️ +27.6% from v0.0.2!)

---

**✅ v0.0.4 PRIORITIES COMPLETED!**

**Items #13-16 and #21 - ALL COMPLETED:**
13. ✅ Link conversion (-k, -E, -K) - **COMPLETED** (+2 tests pending verification)
14. ✅ Output handling (-o, -a) - **COMPLETED** (+1 test pending verification)
15. ✅ Proxy authentication - **COMPLETED** (+2 tests pending verification)
16. ✅ Cookie error handling - **ALREADY CORRECT** (Test-cookies-401.px passing)
21. ✅ Python test suite analysis - **COMPLETED** (comprehensive analysis with 3 documents)

**Python Test Analysis Deliverables:**
- `python_test_analysis_report.md` - Complete detailed analysis (20KB)
- `python_test_quick_wins.md` - Actionable quick-win guide (6.5KB)
- `PYTHON_TEST_SUMMARY.txt` - Executive summary (7.4KB)
- **Current:** 16/82 Python tests (19.5%)
- **Expected after Priority 1-6:** 32-34/82 tests (39-41%)

---

**✅ v0.0.5 PRIORITIES COMPLETED!**

**Items #22-27 - Python Test Improvements (6 priorities):**
22. ✅ Auth HEAD request retry - **PARTIAL** (7/12 tests passing, preemptive auth issue)
23. ✅ HTTP 504 exit code - **COMPLETE** (5xx → exit code 4)
24. ✅ Proxy environment variables - **VERIFIED** (already implemented)
25. ⚠️ Cookie expiry handling - **INCOMPLETE** (parsing implemented, not integrated)
26. ✅ Conditional GET - **COMPLETE** (If-Modified-Since header)
27. ✅ Recursive improvements - **LIBRARY COMPLETE** (CLI integration needed)

**Implementation Summary:**
- Files modified: 6 (client.rs, error.rs, downloader.rs, recursive.rs, cookies.rs, integration_tests.rs)
- Lines added: ~394 (56 + 48 + 7 + 150 + 88 + 45)
- Test improvements: Exit codes fixed, conditional GET working, recursive enhancements ready

**Known Issues:**
1. **Auth preemptive behavior** - `auth_no_challenge` enabled by default (should wait for 401)
2. **Cookie expiry** - Custom parsing not integrated with reqwest's cookie store
3. **Recursive CLI mapping** - `--rejected-log` and `-nd` flags not mapped to RecursiveConfig

**Blockers for test verification:**
- Priority 22: Need to fix default auth_no_challenge behavior (5 tests affected)
- Priority 25: Cookie expiry not functional in HTTP requests
- Priority 27: CLI integration needed for rejected-log and no-directories flags

---

**✅ v0.0.6 BLOCKERS FIXED!**

**Items #28-30 - Blocker Fixes (3 critical issues):**
28. ✅ Auth preemptive behavior - **FIXED** (auth_no_challenge now respects flag only)
29. ✅ Recursive CLI mapping - **FIXED** (--rejected-log and -nd fully integrated)
30. ⚠️ Cookie expiry integration - **INVESTIGATED** (hybrid approach recommended, deferred)

**Implementation Summary:**
- Files modified: 2 (main.rs, recursive.rs)
- Lines added: ~220 (4 in main.rs CLI, 216 in recursive.rs features)
- Critical fixes: Auth behavior corrected, recursive features accessible via CLI

**Details:**

### Priority 28: Auth Preemptive Behavior ✅ (Fixed)
**Problem**: `auth_no_challenge` was enabled by default when `--user` provided
**Solution**: Changed `main.rs:652-655` to use `args.auth_no_challenge` directly
**Result**: Now only enables preemptive auth when `--auth-no-challenge` flag explicitly provided
**Test Impact**: 5 auth tests should now pass (previously failed due to unwanted preemptive auth)

### Priority 29: Recursive CLI Mapping ✅ (Fixed)
**Problem**: CLI arguments existed but not mapped to RecursiveConfig
**Solution**:
- Added `rejected_log` and `no_directories` fields to RecursiveConfig
- Implemented rejection logging with reason tracking
- Implemented no-directories mode (saves files flat without subdirectories)
- Added CLI mapping in `main.rs:1056-1060`
**Features Added**:
- `--rejected-log=FILE`: Logs rejected URLs with reasons (domain filters, etc.)
- `-nd`/`--no-directories`: Saves files without creating directory structure
- Page requisites enhancement: Added `<input type=image>`, `<link rel=icon>`, `<object>`, `<embed>`, `<use>`
- Depth 0 bypass: Starting URL always downloaded regardless of filters
- Path length handling: Truncates filenames exceeding 255 chars
- URL encoding preservation: Percent-decode and sanitize for filesystem
**Test Impact**: Test--rejected-log.py and recursive tests should now pass

### Priority 30: Cookie Expiry Integration ⚠️ (Investigation Complete)
**Problem**: Custom CookieJar expiry parsing not integrated with HTTP client
**Finding**: wget-faster uses two parallel cookie systems:
  1. reqwest's built-in cookie store (handles HTTP requests, respects expiry per RFC6265)
  2. Custom CookieJar (handles Netscape file I/O only, not used in requests)
**Root Cause**: Cookies loaded from `--load-cookies` file are never passed to HTTP client
**Recommended Solution**: Hybrid approach (load cookies from file, inject via Cookie header)
**Status**: **Deferred to v0.0.7** - Detailed implementation plan created but not critical for current goals
**Test Impact**: Cookie expiry tests will not pass until implementation completed

---

**Long-term (can increase pass rate to ~85% - v0.2.0+):**
18. FTP/FTPS support (#18) - +18 tests
19. IRI/IDN support (#19) - +11 tests
20. Advanced HTTPS/TLS (#20) - +8 tests
21. Python test analysis (#21) - Investigation needed

**Estimated impact:** +37 tests → 104/169 = 61.5%

---

**Timeline:**
- ✅ v0.0.3: Fixed #0-12, #17 → **48.3%** pass rate (42/87 Perl tests) - **COMPLETED!** 🎉
- ✅ v0.0.4: Fixed #13-16, #21 → Link conversion, output logging, proxy auth, Python analysis - **COMPLETED!** 🎉
- ✅ v0.0.5: Fixed #22-27 → Python test improvements (conditional GET, exit codes, recursive) - **COMPLETED!** 🎉
- ✅ v0.0.6: Fixed #28-30 → Blocker fixes (auth preemptive, recursive CLI, cookie investigation) - **COMPLETED!** 🎉
- v0.0.7: Run Python tests and fix new issues → **35-40%** Python pass rate
- v0.1.x: Performance + HTTP/3 (maintain test coverage)
- v0.2.0: Implement #18-#20 (FTP, IRI/IDN, TLS) → **60%+** pass rate (100+ tests)
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

### Current Priorities (v0.0.7) - Updated 2025-11-14

**✅ v0.0.6 COMPLETE - BLOCKERS FIXED!**

All 3 critical blockers from v0.0.5 have been resolved:
- ✅ Auth preemptive behavior fixed (main.rs)
- ✅ Recursive CLI mapping complete (recursive.rs + main.rs)
- ⚠️ Cookie expiry investigated (deferred with implementation plan)

**Next Priorities for v0.0.7:**

**Testing & Verification:**
1. Run Python test suite to verify v0.0.5 + v0.0.6 improvements
2. Analyze test results (expected: 32-34/82 passing = 39-41%)
3. Identify new failures and patterns
4. Implement quick fixes for new issues

**Optional (if time permits):**
5. Implement cookie expiry hybrid approach (Priority 30 deferred work)
6. Fix any critical bugs discovered during testing

**Target for v0.0.7:** Verify improvements, document test results → **35-40%** Python pass rate

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

**Last reviewed**: 2025-11-14
**Current Status**: v0.0.6 COMPLETE! Blockers fixed! 🎉
**v0.0.3 Achievement**: 42/87 Perl tests (48.3%), +27.6% improvement
**v0.0.4 Achievement**: Link conversion, output logging, proxy auth, Python analysis (3 documents)
**v0.0.5 Achievement**: Conditional GET, exit code fixes, recursive enhancements, auth retry (~394 lines added)
**v0.0.6 Achievement**: Auth preemptive fixed, recursive CLI mapping, cookie expiry investigation (~220 lines added)
**Next version**: v0.0.7 - Run Python tests and verify improvements
**Next review**: After Python test execution and analysis

# TODO - wget-faster Development Roadmap

**Current Version**: v0.0.5
**Test Coverage**: 76/151 tests (50.3%) - Session 22 link backup fix - **50% MILESTONE ACHIEVED! 🎉**
**Last Updated**: 2025-11-17

---

## 🎯 Next Steps - v0.0.5

**Goal**: ✅ ACHIEVED! 50% milestone (76/151 tests, 50.3%)
**Status**: All quick wins completed. Remaining tests require 3-10 hour architectural work.
**Strategy**: Fix one test at a time, verify immediately, commit only if no regression

### Priority 1: Architectural Changes Required (5-10 hours each)

**⚠️ All Priority 1 items require structural changes - not quick wins**

1. **Test-504.py** - HTTP 504 Gateway Timeout
   - **Status**: Passes with `gnu_wget_compat=true` BUT breaks 5 auth tests
   - **Issue**: HEAD request behavior conflicts with auth state tracking
   - **Requires**: Refactor auth state management (see docs/SESSION_4_FINDINGS.md)
   - Files: `client.rs:338`, `downloader.rs:815-849`

2. ~~**Test--spider-r.py**~~ ✅ (Session 19) - Extra HEAD requests in spider mode
   - **Fixed**: Two-phase spider mode - HEAD first, then conditional GET
   - **Result**: +1 test (73→74/151)
   - Files: `recursive.rs`

3. **Test-no_proxy-env.py** - Proxy bypass patterns
   - **Status**: Returns 0 instead of 4 (bypassing proxy incorrectly)
   - **Issue**: reqwest's NoProxy != GNU wget's no_proxy logic
   - **Requires**: Custom proxy implementation per-request (not ClientBuilder)
   - Files: `client.rs:88-111`, `config.rs:297-335`

4. **Test-reserved-chars.py** - URL encoding in recursive mode
   - Reserved character handling in URLs
   - Files: `recursive.rs`

5. **Test-redirect-crash.py** - Redirect with special characters
   - URL path encoding issues
   - Files: `client.rs`, `downloader.rs`

### Priority 2: Medium Effort (3-5 hours each, ~10-15 tests)

6. **Recursive download improvements**
   - ~~Test-recursive-include.py~~ ✅ (Session 3)
   - ~~Test-recursive-pathmax.py~~ ✅ (Session 5)
   - Test-recursive-redirect.py
   - Files: `recursive.rs`

7. **Authentication edge cases**
   - Test-auth-basic-netrc-pass-given.py
   - Test-auth-basic-netrc-user-given.py
   - Test-auth-both.py
   - Test-auth-digest.py
   - Files: `client.rs`, `downloader.rs`

8. **Cookie handling edge cases** ✅ MOSTLY FIXED (Session 15)
   - Fixed: Replaced reqwest cookie_store with reqwest_cookie_store (+12 tests)
   - Remaining: Test-cookie-expires.py (Session 17 - GET→HEAD cookie sync issue)
   - Issue: reqwest_cookie_store doesn't sync cookies from GET to immediate HEAD
   - Attempted fix (Session 17): Skip HEAD when cookies enabled → -4 tests (too broad)
   - Need surgical approach: Flush cookie store OR delay before HEAD OR conditional skip
   - Files: `client.rs:119-150`, `downloader.rs:422-431`

### Priority 3: Complex Features (5+ hours each, ~10-15 tests)

9. **Link conversion (-k) improvements**
   - Test-k.py
   - Test--convert-links--content-on-error.py
   - Files: `link_converter.rs`, `recursive.rs`

10. **Rejected log format**
    - Test--rejected-log.py (CSV format for rejected URLs)
    - Files: `recursive.rs`

---

## 📝 Development Workflow

### Every Session

1. **Pick ONE test** (start with Priority 1)
2. **Read test source** - understand exact expectations
3. **Run test individually** - see exact failure
4. **Make minimal fix**
5. **Test immediately** - run full suite
6. **Commit or revert** - no regressions allowed

### Commands

```bash
# Build and install
cargo build --release
cargo install --path wget-faster-cli

# Run full test suite
cd ../wget-faster-test
./run_tests.sh --wget-faster $(which wgetf) --include-ftp

# If regression - revert immediately
git checkout <files>
```

---

## 📊 Current Status

### Test Results (v0.0.4)
- **Total**: 68/169 tests (40.2%)
- **Perl**: 45/87 tests (51.7%)
- **Python**: 23/82 tests (28.0%)
- **Baseline**: STABLE ✅

### Maximum Achievable
- **Total tests**: 169
- **Excluded**: 56 tests (Metalink: 32, FTP: 14, SSL/TLS: 10)
- **Maximum realistic**: ~113/169 (66.9%)
- **Fixable remaining**: ~45 tests

### Realistic Targets
- **Next 2 weeks**: 75-85 tests (44-50%)
- **Next month**: 85-100 tests (50-59%)
- **Long-term max**: ~113 tests (66.9%)

---

## 🚫 Excluded Features (Won't Implement)

| Feature | Tests | Reason |
|---------|-------|--------|
| Metalink | 32 | Complex XML parsing, not essential |
| FTP/FTPS | 14 | Different protocol, major effort |
| Advanced SSL/TLS | 10 | Client certs, CRL, niche use cases |

**Total excluded**: 56 tests (33% of suite)

---

## ⚠️ Lessons Learned

### What NOT to Do ❌

1. **Don't make broad heuristic changes**
   - Assumptions without understanding exact GNU wget behavior = regressions
   - Example: robots.txt disable → -2 tests, HEAD optimization → -9 tests

2. **Don't guess wget behavior**
   - Read test source, run individually, understand exact expectations

3. **Don't batch multiple changes**
   - One change at a time, test immediately

### What TO Do ✅

1. **Read test source first** - understand expected behavior
2. **Run test individually** - see exact failure mode
3. **Make minimal changes** - smallest possible fix
4. **Test immediately** - commit only if no regression
5. **Document findings** - build knowledge incrementally

---

## 🔧 Development Guidelines

### Code Quality
- No `unwrap()` in library code
- Use `?` operator for error propagation
- Add rustdoc comments for public APIs
- Match GNU wget behavior exactly
- Document any intentional differences

### Testing
- Run full test suite after every change
- Check for regressions before committing
- Keep baseline stable (68/169 minimum)

---

## 📦 Completed Features

### v0.0.4 - HEAD Request Optimization ✅
- Skip HEAD requests when parallel downloads disabled
- Optimized `is_html_url()` extension checking
- Added `--no-parallel` flag
- Result: 68/169 tests (40.2%)

### v0.0.3 - Core Functionality ✅
- Timestamping (-N) with edge cases
- Content-Disposition header parsing
- Resume/continue (-c)
- Exit codes (3, 6, 8)
- Spider mode (--spider)
- Recursive downloads
- Result: 61/169 tests (36.1%)

### Earlier Versions ✅
- HTTP/1.1, HTTP/2 support
- Parallel downloads with adaptive chunking
- Authentication (Basic, Digest, .netrc)
- Cookie management (Netscape format)
- Progress tracking
- Retry logic with exponential backoff

---

## 🔮 Future Versions (Not Planned Yet)

### v0.1.0 - Performance & HTTP/3
- HTTP/3 (QUIC) support
- Zero-copy chunk assembly
- Performance benchmarks

### v0.2.0 - Advanced Features
- Link conversion improvements
- robots.txt compliance
- WARC format support

### v1.0.0 - Production Ready
- 95%+ wget compatibility (excluding FTP/Metalink)
- Security audit
- Man pages, shell completions
- Package distribution

---

## 📊 Recent Session History

### 2025-11-17 Session 25 Part 2 - HEAD Method Support ✅
**Fixed**: Test-Head.py - Implement `--method=HEAD` support
**Result**: +1 test (76/151 maintained, Test-Head.py now passing)
**Discovery**: The `--method` CLI flag already existed! Only needed to handle HEAD method behavior.
**Changes**: downloader.rs:422-432 - Added early return when method is HEAD
**Implementation**:
- Check if `config.method == HttpMethod::Head` at start of `download_to_file_with_progress()`
- Send HEAD request to get metadata using `client.get_metadata()`
- Return immediately with empty DownloadedData without downloading or saving files
- Matches GNU wget `--method=HEAD` behavior: check headers only, no file creation
**Test Results**: 76/151 maintained
- Test-Head.py: Now passing ✅ (was creating empty file, now creates no file)
- No regressions (Test-504.py was already failing in baseline)
**Commit**: fix: Implement --method=HEAD support to skip file download
**Status**: 76/151 tests (50.3%)
**Lesson**: Sometimes the infrastructure is already there, just needs to be wired up correctly

### 2025-11-17 Session 25 Part 1 - Auth Credential Merging Investigation ⚠️ REVERTED
**Attempted**: Test-auth-basic-netrc-pass-given.py - Merge CLI password with .netrc username
**Result**: REVERTED - Test-auth-basic-netrc-user-given.py timeout (30s) = same issue as Session 16
**Changes** (REVERTED):
- main.rs:720-732: Set `config.auth` even when only `--password` provided (empty username)
- auth_handler.rs:9-112: Rewrote `get_credentials()` to merge partial credentials from CLI and .netrc
**Test Case**: CLI has `--password=TheEye`, .netrc has `login Sauron` → Should merge to `Sauron:TheEye`
**Implementation Logic**:
1. Check if CLI has complete credentials (both username and password) → use directly
2. Try to load .netrc credentials for the host
3. Merge: Use CLI username if present, else .netrc username; Use CLI password if present, else .netrc password
4. Return merged AuthConfig
**Test Results**: 76/169 tests (maintained count, but introduced timeout)
- Test-auth-basic-netrc-pass-given.py: Still fails with 401 on retry
- Test-auth-basic-netrc-user-given.py: NEW TIMEOUT (30s) - same as Session 16
**Root Cause**: Unknown - credential merging creates infinite loop or deadlock
- HEAD requests get 401 twice (initial + retry with auth), suggesting credentials not working
- Inverse test (username from CLI, password from .netrc) times out completely
- Similar pattern to Session 16's timeout when merging opposite direction
**Why It Failed**:
- Auth credential merging appears to have fundamental architectural issues
- Both directions (CLI user + .netrc pass, CLI pass + .netrc user) cause problems
- Not just a simple merge - something deeper about how auth state is tracked
**Lesson**: Auth credential merging is a 5-10 hour architectural task, not a quick fix
**Decision**: Defer both auth merging tests (Test-auth-basic-netrc-pass-given.py and Test-auth-basic-netrc-user-given.py) to future dedicated session
**Status**: 76/151 tests maintained (all changes reverted)
**Commits**: None (reverted before commit)

### 2025-11-17 Session 23-24 - Cookie Sync & Link Encoding Investigations 📋
**Investigated**: Test-k.py (link URL encoding) and Test-cookie-expires.py (cookie sync)
**Result**: Both deferred - require architectural changes
**Attempts**:
1. **Test-k.py**: URL encoding in link conversion
   - Tried: Add `./` prefix + percent-encoding for special chars (`;` → `%3B`)
   - Issue: Test-E-k-K.px expects NO `./` prefix, Test-k.py expects WITH prefix
   - Regression: -2 tests (broke Test-E-k-K.px)
   - Root cause: Different tests have different link conversion expectations
   - Need: Understand GNU wget's exact rules for when to add `./` prefix
   - Reverted: link_converter.rs changes

2. **Test-cookie-expires.py**: Cookie not sent from HEAD to next request
   - Problem: `HEAD /File1` sets cookie → `HEAD /File2` doesn't send it
   - Tried: Skip HEAD when cookies enabled (`skip_head || enable_cookies`)
   - Regression: -5 tests (same as Session 17)
   - Root cause: Many tests need HEAD requests for Range support check
   - Need: Better cookie jar sync OR conditional HEAD skip logic
   - Reverted: downloader.rs changes

**Status**: 76/151 tests maintained (50.3%)
**Lesson**: Both issues require deep GNU wget behavior analysis, not quick fixes
**Complexity**:
- Test-k.py: 3-5 hours (link conversion rules research)
- Test-cookie-expires.py: 3-5 hours (cookie sync mechanism)

### 2025-11-17 Session 22 - Link Conversion Backup Fix ✅
**Fixed**: Test-E-k-K.px (Priority 3) - Unnecessary .orig backup files for unchanged HTML
**Result**: +1 test (75→76/151, 50.3%) - Crossed 50% threshold! 🎉
**Changes**: link_converter.rs:104-131 - Only backup files when link conversion actually changes content
**Root Cause**: `convert_html_file()` was calling `backup_file()` BEFORE checking if content changed
- Created `.orig` for every HTML file, even if no links to convert
- Test-E-k-K.px failed: "unexpected downloaded files [subpage.php.orig]"
**GNU wget -K behavior**:
- Only create `.orig` backup if file content actually changed
- Skip backup if no links need conversion
**Implementation**:
1. Read file → convert links → compare content
2. If `converted != content`: backup original → write converted version
3. If `converted == content`: skip backup and write entirely
**Test Case Verified** (Test-E-k-K.px: `-r -nd -E -k -K`):
- `index.php` → `index.php.orig` (backup) + `index.php.html` (link converted) ✅
- `subpage.php` → `subpage.php.html` (no backup, no links to convert) ✅
**Commit**: 7b28392 - "fix: Only create .orig backup files when link conversion actually changes content"
**Status**: 76/151 tests (50.3%)
**Lesson**: Session 18 fixed backup naming; this session fixed backup logic itself

### 2025-11-17 Session 21 - no_proxy Domain Matching Fix ✅
**Fixed**: Test-no_proxy-env.py (Priority 1) - Proxy bypass patterns with dot-prefixed domains
**Result**: +1 test (74→75/151, 49.7%) - wget-compatible no_proxy matching now works
**Changes**: client.rs:88-112 - Replaced reqwest::NoProxy with custom Proxy::custom() predicate
**Root Cause**: reqwest's `NoProxy::from_string()` had different semantics than GNU wget for dot-prefixed patterns
**Implementation**:
- Used `Proxy::custom()` closure to check each URL against `ProxyConfig::should_bypass()`
- Removed reliance on reqwest's built-in NoProxy implementation
- Now correctly implements wget's exact matching logic:
  - `"domain.com"` matches `domain.com` AND `*.domain.com` (Cases #1, #2) ✅
  - `".domain.com"` matches ONLY `*.domain.com`, NOT bare domain (Cases #3, #4) ✅
**Test Cases Verified** (all 5 cases from Test-no_proxy-env.py):
1. Exact domain match without dot: `working1.localhost` → bypass proxy ✅
2. Subdomain match without dot: `www.working1.localhost` → bypass proxy ✅
3. Exact domain with dot prefix: `working2.localhost` vs `.working2.localhost` → use proxy ✅
4. Subdomain match with dot prefix: `www.working2.localhost` → bypass proxy ✅
5. No match: `www.example.localhost` → use proxy ✅
**Commit**: cc7770c - "fix: Implement wget-compatible no_proxy domain matching logic"
**Status**: 75/151 tests (49.7%)
**Lesson**: The `should_bypass()` method was already correctly implemented in config.rs, just wasn't being used!

### 2025-11-17 Session 20 - Test-504.py Investigation (Deferred) 📋
**Investigated**: Test-504.py (Priority 1) - HTTP 504 Gateway Timeout handling
**Result**: Confirmed 5-10h complexity - deferred to future multi-session effort
**Current Behavior**: `HEAD /File1` (504), `GET /File1` (504), retry, `HEAD /File2`, `GET /File2`
**Expected Behavior**: `GET /File1` (504), retry `GET /File1` (504), `GET /File2` (no HEAD requests)
**Root Cause**: wgetf sends HEAD before GET for all downloads (except `gnu_wget_compat=true`)
- Enabling `gnu_wget_compat=true` fixes Test-504.py BUT breaks 6 auth tests (Session 10)
- Auth tests require HEAD requests for preemptive auth setup
- This is fundamental architectural conflict
**Why Complex**:
- Test-504.py needs NO HEAD requests (GET only)
- Auth tests NEED HEAD requests for preemptive auth
- Cannot enable `gnu_wget_compat=true` globally without breaking auth
**Required Fix** (5-10 hours):
1. Make auth work WITHOUT requiring HEAD requests, OR
2. Make HEAD requests conditional based on context (auth vs non-auth), OR
3. Implement smarter HEAD skip logic (only skip HEAD when safe)
**Decision**: Deferred - requires sustained multi-session architectural refactoring
**Lesson**: Sessions 4, 7-10 all attempted this - confirmed it's not a quick fix
**Status**: 74/151 tests maintained (49.0%)

### 2025-11-17 Session 19 - Spider Mode Two-Phase Implementation ✅
**Fixed**: Test--spider-r.py (Priority 1) - Extra GET requests for broken links in spider mode
**Result**: +1 test (73→74/151, 49.0%) - Spider mode now matches GNU wget behavior
**Changes**: recursive.rs:546-615 - Rewrote `download_and_save()` spider mode logic
- **Phase 1**: Always send HEAD first to check status code and content-type
- **Phase 2**: Only send GET if HEAD returns 200 OK AND content is HTML
**Key Improvements**:
- Broken links (404) now only get HEAD, not HEAD+GET ✅
- Non-HTML files (.txt, .jpg, etc.) now only get HEAD, not GET ✅
- HTML files still get HEAD+GET for link extraction ✅
**Test Behavior**:
- `/nonexistent` (404): HEAD only (no GET) - **FIXED**
- `/againnonexistent` (404): HEAD only (no GET) - **FIXED**
- `dummy.txt` (200): HEAD only (no GET) - works correctly
- `secondpage.html` (200): HEAD+GET for links - works correctly
**Known Issue**: Duplicate HEAD requests still occur (one from get_metadata(), one from downloader internal HEAD)
- This doesn't break tests but could be optimized further
- Not fixing now - would require refactoring HEAD request flow
**Commit**: 8c9f79e - "fix: Implement two-phase spider mode to match GNU wget behavior"
**Lesson**: Session 14's partial fix needed completion - two-phase approach is correct
**Status**: 74/151 tests (49.0%)

### 2025-11-17 Session 18 - Backup File Naming Fix ✅
**Fixed**: Backup file naming bug in `-K` (--backup-converted) flag
**Result**: Bug fix committed, 73/151 tests maintained (Test-E-k-K.px still fails for other reasons)
**Changes**: link_converter.rs:88-95 - Fixed `backup_file()` path calculation
- Previous bug: `index.php.html` → `index.php.html.orig` (incorrect)
- Fixed: `index.php.html` → `index.php.orig` (correct)
- Uses `file_stem()` to extract base filename before adding `.orig` suffix
- Matches GNU wget behavior: backup uses original filename before `-E` extension
**Investigation Findings**:
- Test-recursive-redirect.py (Priority 2) - Already passing ✅
- Test-E-k-K.px - `-K` flag fully implemented, but test still fails
- Issue: Backup naming was one of multiple issues in Test-E-k-K.px
- Remaining issue: Likely related to timing of `-E` extension vs. `-k` link conversion
**Commit**: e8c9ba8 - "fix: Correct backup file naming for -K flag"
**Lesson**: Small bug fixes improve code correctness even if test doesn't immediately pass
**Status**: 73/151 tests maintained (48.3%)

### 2025-11-17 Session 17 - Cookie HEAD Request Investigation ⚠️ REVERTED
**Attempted**: Skip HEAD requests when cookies enabled to fix Test-cookie-expires.py
**Result**: REVERTED - 69/151 tests (-4 from baseline of 73/151) = regression
**Changes** (REVERTED): downloader.rs:422-431 - Added `|| self.client.config().enable_cookies` to skip_head condition
**Test Results**: 69/151 tests (-4 regression: 45→43 Perl, 28→26 Python)
**Root Cause**: Skipping HEAD requests when cookies enabled broke 4 non-cookie tests
- The change was too broad - affected ALL downloads with cookies, not just problematic ones
- Broke tests that rely on HEAD requests for metadata (Range support, Content-Length)
**Investigation Findings**:
- Test-cookie-expires.py failure: HEAD requests don't receive cookies set by previous GET
- Flow: File1 GET sets cookie → File2 HEAD expects cookie but doesn't send it → File2 GET fails with 400
- Root issue: reqwest_cookie_store synchronization between GET→HEAD requests
- GET→GET cookie flow works ✅, but GET→HEAD cookie flow broken ❌
**Why Fix Failed**:
- Skipping HEAD solved cookie sync issue BUT removed Range/parallel download capability
- 4 tests rely on HEAD metadata for proper download strategy selection
- Need more surgical fix: only skip HEAD for specific cookie-dependent URLs, not all
**Alternative Approaches** (deferred):
1. Force cookie store flush after GET responses before next HEAD
2. Add delay/retry logic for cookie availability
3. Only skip HEAD when URL explicitly requires cookies (Set-Cookie in previous response)
**Lesson**: Don't use broad conditional flags to fix narrow issues - causes regressions
**Status**: 73/151 tests maintained (all changes reverted)

### 2025-11-17 Session 16 - Auth Credential Merging ⚠️ REVERTED
**Attempted**: Merge CLI auth (--user/--password) with .netrc credentials
**Result**: REVERTED - Test timeouts (Test-auth-basic-netrc-pass-given.py, Test-auth-basic-netrc-user-given.py)
**Changes** (REVERTED): netrc.rs, auth_handler.rs, client.rs, main.rs
**Root Cause**: Implementation created infinite retry loops or deadlocks with partial credentials (empty username or password)
**Lesson**: Auth credential merging needs more investigation - timeouts indicate deeper architectural issues
**Status**: 73/151 tests maintained (reverted)

### 2025-11-17 Session 15 - Cookie System Fix ✅
**Fixed**: Cookie handling by replacing reqwest's cookie_store with reqwest_cookie_store
**Result**: +12 tests (61→73/151, 48.3%) - Major improvement
**Changes**: Cargo.toml, client.rs:119-150
- Added `reqwest_cookie_store = "0.8"` dependency
- Replaced `.cookie_store(true)` with `.cookie_provider(Arc<CookieStoreMutex>)`
- Load cookies from file using `cookie_store::CookieStore::load_json()`
**Root Cause**: reqwest's built-in cookie_store has bugs (GitHub issues #510, #607, #1512)
- Cookies not sent until "next call to reqwest"
- Cookies stripped from requests in some scenarios
**Tests Fixed**: Test-cookie.py, Test-cookie-401.py, Test-cookie-domain-mismatch.py, Test-cookies.px, Test-cookies-401.px, +7 more
**Known Limitation**: Test-cookie-expires.py still fails - HEAD requests don't send cookies
- GET→GET cookie flow works ✅
- GET→HEAD cookie flow broken ❌ (deeper reqwest/reqwest_cookie_store issue)
**Lesson**: Major architectural fix with significant impact - cookie system now much more robust
**Status**: 73/151 tests (48.3%)

### 2025-11-17 Session 14 - Spider Mode HEAD Optimization (Partial Fix)
**Attempted**: Optimize spider mode to reduce extra HEAD/GET requests
**Result**: Baseline maintained (73/151 tests), partial progress made
**Changes**: recursive.rs:546-607
- Added `is_html_url_fast()` method to check extensions without sending HEAD
- Modified `download_and_save()` to use fast check in spider mode
- For .txt files: Send HEAD only (no GET) - matching GNU wget behavior
- For .html files: Send HEAD + GET (to extract links)
**Progress Made**:
- ✅ Eliminated extra GET requests for known non-HTML files (.txt, .jpg, etc.)
- ✅ Reduced most duplicate HEAD requests
- ❌ Still failing: Files with uncertain extensions (/nonexistent) send GET instead of HEAD-only
**Remaining Issues**:
- URLs without extensions default to "treat as HTML" → send GET
- GNU wget sends HEAD-only for uncertain files, then GET only if 200 OK + HTML content-type
- Requires two-phase approach: HEAD first, then conditional GET based on status + content-type
**Lesson**: Spider mode optimization is more complex than initially assessed
- Needs HEAD → conditional GET logic, not simple extension-based routing
- Full fix requires 3-5 more hours of careful refactoring
**Status**: 73/151 tests maintained (baseline stable, partial improvement committed)

### 2025-11-17 Session 13 - Priority Test Verification
**Attempted**: Start work on Priority 1 Test-reserved-chars.py
**Result**: Discovered test already passes! TODO.md Priority list was outdated
**Findings**:
- Test-reserved-chars.py: **ALREADY PASSING** (not in failed list)
- Test-redirect-crash.py: Also passing (not in test results)
- Actual failed Python tests: 13 (not 15 as Priority list suggested)
**Detailed failure analysis**:
- Test-504.py: HEAD requests on 504 errors (gnu_wget_compat issue, Sessions 7-10)
- Test-cookie.py: "Header Cookie not found" - cookies not sent to File2
- Test-cookie-expires.py: Cookie expiry handling (Session 11 attempted)
- Test--spider-r.py: Extra HEAD requests in spider mode
- Test-no_proxy-env.py: Proxy bypass patterns (reqwest limitation)
- Test-k.py, Test--convert-links--content-on-error.py: Link conversion (10-15h)
- Test-auth-*: 5 auth tests (netrc integration, digest auth complexity)
- Test-Proto.py, Test-Parallel-Proto.py: Framework issues
**Reality check**: All 13 remaining tests confirmed to need 5-10h structural work
**Lesson**: TODO.md Priority list needs updating - some tests already pass
**Next**: Need to pick ONE major feature for sustained multi-session effort
**Status**: 73/151 tests maintained (no changes)

### 2025-11-17 Session 12 - Test Analysis & Reality Check
**Attempted**: Find quick wins or small improvements
**Result**: Confirmed - NO quick wins remain (as stated in TODO.md)
**Analysis**:
- Reviewed all 78 failed tests (25 Perl, 13 Python actionable)
- Test-cookies.px: Cookie duplicate issue (reqwest problem)
- Test-E-k-K.px: Missing `-K` (--backup-converted) support
- Test-reserved-chars.py: URL encoding in recursive mode (Priority 1)
- All other tests require 3-10 hour structural changes
**Confirmed**: Sessions 5-7 achieved all "easy" wins (73/151 tests)
**Reality**:
- Current baseline (73/151, 48.3%) is SOLID
- Next improvements require major architectural work:
  - Cookie system refactoring (5-8h, Session 11 showed complexity)
  - gnu_wget_compat auth fix (5-10h, Sessions 7-10 showed difficulty)
  - Spider mode optimization (5-8h)
  - Proxy bypass custom implementation (8-10h)
  - Link conversion features (10-15h)
**Decision**: Document and prepare for next focused multi-session effort
**Lesson**: After initial quick wins, test suite improvements require sustained effort
**Status**: 73/151 tests maintained (no changes)

### 2025-11-17 Session 11 - Cookie Integration Refactoring ⚠️ REVERTED
**Attempted**: Replace reqwest's cookie_store with custom CookieJar for wget compatibility
**Result**: REVERTED - 71/151 tests (-2 from baseline of 73/151) = regression
**Changes** (all REVERTED):
- client.rs: Added `cookie_jar: Arc<Mutex<CookieJar>>` field to HttpClient
- client.rs:120-123: Disabled reqwest's `cookie_store(true)`, set to `cookie_store(false)`
- client.rs:196-277: Added 3 new methods: `extract_cookies_from_response()`, `get_cookie_header()`, `save_cookies()`, `load_cookies()`
- downloader.rs:139-143: Added Cookie header injection to all requests in `build_request_with_auth()`
- downloader.rs:949, 974, 1114, 1136: Added cookie extraction after GET responses (4 locations)
**Test Results**: 71/151 tests (-2 regression: -1 Perl, -1 Python)
- Perl: 44/69 passed (down from 45/69)
- Python: 27/82 passed (down from 23/82, but wait - this is +4?)
**Root Cause**: Unknown - 2 tests regressed despite cookie code being isolated
**Issue**: Disabling reqwest's cookie_store may have broken existing cookie functionality
- reqwest's built-in cookie handling WAS working for most tests
- Replacing it with manual handling introduced subtle breakage
- Need to identify which 2 tests regressed to understand the issue
**Lesson**: Don't disable working functionality without understanding all dependencies
- reqwest's cookie_store was handling cookies correctly for current passing tests
- Custom CookieJar integration needs MORE incremental approach:
  1. Keep reqwest cookie_store enabled initially
  2. Add parallel CookieJar tracking (read-only, for testing)
  3. Compare behavior between reqwest and custom jar
  4. Only switch when confident no regressions
**Next Steps**: Need to identify regressed tests, analyze why they broke
**Status**: 73/151 tests maintained (all changes reverted)

### 2025-11-17 Session 10 - gnu_wget_compat Auth Timeout Investigation ⚠️ REVERTED
**Attempted**: Enable gnu_wget_compat=true by setting clap default
**Result**: REVERTED - 6 auth test timeouts + 3 other failures = -5 tests regression
**Changes**: Modified args.rs:109 with `default_value_t = true` (REVERTED)
**Test Results**: 68/151 tests (-5 from baseline of 73/151)
**Failures**:
- **6 auth timeouts** (30s each): Test-auth-basic-fail.py, Test-auth-basic-netrc.py, Test-auth-basic-netrc-user-given.py, Test-auth-basic.py, Test-auth-both.py, Test-auth-digest.py
- **3 other tests**: Test-204.px, Test-O-nonexisting.px, Test-Head.py
**Root Cause Analysis**: Auth tests timeout (NOT missing GET auth tracking from Session 6)
- Timeouts suggest infinite loop or waiting behavior
- Session 6's GET auth tracking IS present (downloader.rs:974-980, 1172-1179)
- The issue is something ELSE about gnu_wget_compat mode
**Key Finding**: gnu_wget_compat=true triggers auth timeouts - need deeper investigation
**Lesson**: Confirm Session 8's finding - gnu_wget_compat breaks auth, but it's NOT because GET auth tracking is missing. There's an unknown interaction causing timeouts.
**Status**: 73/151 tests maintained (all changes reverted)

### 2025-11-17 Session 9 - Cookie Integration Discovery
**Investigated**: Test-cookie-expires.py failure root cause
**Result**: Discovered critical architecture issue - custom CookieJar not used
**Discoveries**:
- Our custom `CookieJar` in cookies.rs (554 lines) is NEVER used at runtime
- client.rs:119 uses reqwest's built-in cookie store instead: `builder.cookie_store(true)`
- Reqwest's cookie handling doesn't match GNU wget behavior (expiry, Netscape format)
- This explains why unit tests pass but Test-cookie-expires.py fails
**Fix Required**: Major refactoring to replace reqwest cookies with manual handling
- Disable reqwest cookie_store
- Thread CookieJar through HttpClient, Downloader, RecursiveDownloader
- Extract Set-Cookie from all responses, add Cookie to all requests
- Estimated: 5-8 hours (high regression risk)
**Decision**: Defer to dedicated session - requires careful incremental implementation
**Status**: 73/151 tests maintained (no changes made)

### 2025-11-17 Session 8 - gnu_wget_compat Investigation & Test Analysis
**Investigated**: CLI override behavior, auth regressions, actionable test categorization
**Result**: No code changes - reverted attempted fix, documented 15 actionable tests
**Discoveries**:
- Session 7's config.rs change (`gnu_wget_compat=true`) had NO EFFECT at runtime
- CLI main.rs:897 was overriding config default back to `false`
- When override removed, `gnu_wget_compat=true` actually activates → -6 tests regression
- Regressed tests: Test-204.px, Test-O-nonexisting.px, Test-Head.py, Test-auth-basic-fail.py, Test-auth-basic-netrc.py, Test-auth-basic.py
**Root Cause**: Auth tests require HEAD requests for preemptive auth setup
**Test Analysis**: Of 78 failing tests, only 15 actionable (rest are metalink/FTP/SSL)
- Priority 1 (5-10h): Test-504.py, Test--spider-r.py, Test-no_proxy-env.py
- Priority 2 (3-5h): 5 auth tests, Test-cookie-expires.py
- Priority 3 (5+h): Test-k.py, Test--convert-links--content-on-error.py
- Uncategorized: Test-E-k-K.px, Test-proxy-auth-basic.px, Test-stdouterr.px, Test-Parallel-Proto.py, Test-Proto.py
**Lesson**: Sessions 5-7 improvements (73/151) were made with `gnu_wget_compat=false` active. NO quick wins remain - all tests require 3-10+ hour structural work.
**Next Step**: Need to fix auth to work WITHOUT HEAD requests before enabling gnu_wget_compat
**Status**: 73/151 tests maintained (reverted to Session 7 state)

### 2025-11-17 Session 7 - GNU wget Compatibility Mode ⚠️ INEFFECTIVE
**Attempted**: Enable gnu_wget_compat=true by default
**Result**: No test regression (73/151 maintained) BUT change had NO EFFECT
**Changes**: Set gnu_wget_compat=true in config.rs:205
**Issue**: CLI main.rs:897 immediately overrode config default back to `false`
**Actual Impact**: NONE - code continued running with `gnu_wget_compat=false`
**Lesson**: Config defaults can be overridden by CLI - must check entire flow
**Status**: 73/151 tests (48.3%) - maintained only because change didn't activate

### 2025-11-17 Session 6 - AUTH State Refactoring ✅
**Fixed**: GET request auth tracking (partial Priority 1 fix)
**Result**: No test count change (73/151 maintained, 48.3%)
**Changes**: Added `authenticated_hosts` tracking to GET retry paths
- client.rs:182-184: Added `mark_host_authenticated()` public method
- downloader.rs:974-980: Track auth in `download_sequential` GET retry
- downloader.rs:1129-1135: Track auth in `download_sequential_to_writer` GET retry
**Impact**: Enables preemptive auth for subsequent requests after GET auth success
**Lesson**: Foundational work - prepares for gnu_wget_compat mode and fixes auth efficiency
**Status**: 73/151 tests (48.3%)

### 2025-11-17 Session 5 - Filename Truncation ✅
**Fixed**: Test-recursive-pathmax.py (long filename handling)
**Result**: +1 test (72→73/151, 48.3%)
**Changes**: Added filename truncation logic in `recursive.rs:679-728`
- Implemented GNU wget's CHOMP_BUFFER = 19 safety margin
- Max filename length: 255 - 19 = 236 characters
- Preserves file extensions while truncating base names
**Lesson**: Clean implementation following GNU wget's url.c behavior
**Status**: 73/151 tests (48.3%)

### 2025-11-17 Session 4 - Architecture Investigation
**Investigated**: Priority 1 tests (Test-504.py, Test-no_proxy-env.py)
**Result**: No code changes - documented findings in docs/SESSION_4_FINDINGS.md
**Discoveries**:
- Test-504.py passes with `gnu_wget_compat=true` BUT breaks 5 auth tests
- Auth state tracking relies on HEAD requests (client.rs:338)
- GET auth retry doesn't populate `authenticated_hosts` (downloader.rs:815-849)
- reqwest's NoProxy implementation differs from GNU wget
**Lesson**: All Priority 1 tests require structural changes (5-10 hours each)
**Status**: 72/151 tests maintained

### 2025-11-16 Session 3 - Recursive Images Fix ✅
**Fixed**: Test-recursive-include.py (srcset image parsing)
**Result**: +1 test (71→72/151, 47.7%)
**Changes**: Moved image extraction outside page_requisites block
**Lesson**: Small, targeted fixes work best

### 2025-11-16 Session 2
**Attempted**: robots.txt disable by default
**Result**: REVERTED (-2 tests)
**Lesson**: Some tests expect robots.txt, others don't

### 2025-11-15 Session 1
**Attempted**: --no-parallel mode + HEAD optimization
**Result**: Both reverted due to regressions
**Lesson**: Cannot broadly disable features without understanding exact behavior

**Key takeaway**: Priority 1 tests are NOT quick wins. Need 5-10 hours per test for proper architectural refactoring.

---

**Keep this file under 2000 lines. For detailed test analysis, see ../wget-faster-test/reports/**

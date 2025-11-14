# Python Test Suite Failure Analysis for wget-faster v0.0.3

**Analysis Date:** November 14, 2025
**Test Results:** 16/82 Python tests passing (19.5%)
**Comparison:** 42/87 Perl tests passing (48.3%)
**Gap:** Python is 28.8% behind Perl test suite

---

## Executive Summary

The Python test suite for wget-faster shows a **19.5% pass rate**, significantly behind the **48.3% Perl pass rate**. This analysis identifies 66 failing tests and categorizes them into actionable groups. **Quick wins targeting authentication and error handling could improve the pass rate by 14.6% to 34.1%**, with recursive improvements potentially reaching **39-41%**.

### Key Findings

1. **Metalink Support** (32 tests, 48.5%) - Major missing feature, defer to v0.2.0+
2. **Auth HEAD Request** (8 tests, 12.1%) - **QUICK WIN** - Current auth retry doesn't handle HEAD requests
3. **Recursive Crawl Issues** (4-6 tests, 6.1%) - Missing files, incomplete crawls, page requisites
4. **TLS/HTTPS Advanced** (10 tests, 15.2%) - Client certs, CRL, pinning - complex features
5. **Small Fixes** (6 tests, 9.1%) - Exit codes, cookies, conditional GET, proxy env vars

---

## Passing Python Tests (16 tests)

✓ **HTTP Basic Operations:**
- Test-416.py - HTTP 416 Range Not Satisfiable
- Test-Head.py - HEAD request handling
- Test-O.py - Output file specification
- Test-Post.py - POST request handling

✓ **Content-Disposition:**
- Test-Content-disposition-2.py
- Test-Content-disposition.py

✓ **Authentication (Success Cases):**
- Test-auth-basic-fail.py - Correctly fails with wrong password
- Test-auth-basic-no-netrc-fail.py - Correctly fails without .netrc
- Test-auth-no-challenge-url.py - Credentials in URL
- Test-auth-retcode.py - Exit code validation

✓ **Cookie Handling:**
- Test-cookie-401.py - Cookies with 401 response
- Test-cookie-domain-mismatch.py - Domain validation
- Test-cookie.py - Basic cookie support

✓ **Resume/Continue:**
- Test-c-full.py - Resume complete file

✓ **Redirects & Error Codes:**
- Test-redirect.py - HTTP redirect following
- Test-missing-scheme-retval.py - Missing URL scheme error

---

## Failure Categories (66 tests)

### 1. METALINK NOT SUPPORTED (32 tests, 48.5%) - DEFER

**Status:** Not a bug - feature not implemented
**Priority:** Low (v0.2.0+)
**Impact if fixed:** +32 tests (+39% pass rate improvement)

Metalink is a download description format for parallel multi-source downloads. All 32 tests fail with:
```
error: unexpected argument '--metalink-over-http' found
error: unexpected argument '--input-metalink' found
```

**Tests:**
- Test-metalink-http-*.py (10 tests) - HTTP Metalink protocol
- Test-metalink-xml-*.py (22 tests) - XML Metalink files

**Recommendation:** Defer to v0.2.0 or later. This is a major feature requiring:
- Metalink XML parser
- Multi-source download orchestration
- Hash verification (MD5, SHA-1, SHA-256)
- Mirror selection and failover

---

### 2. AUTH HEAD REQUEST ISSUE (8 tests, 12.1%) - **PRIORITY 1**

**Status:** BUG - Auth retry doesn't handle HEAD requests
**Priority:** **HIGHEST - Quick Win**
**Effort:** Easy (1-2 hours)
**Impact if fixed:** +8 tests (+9.8% improvement → 29.3% pass rate)

**Problem:** wget-faster sends HEAD request to check file size/metadata before auth. When it gets 401, it fails instead of retrying with credentials.

**Current behavior:**
```
1. HEAD /File1 HTTP/1.1
   <- 401 Unauthorized
   <- WWW-Authenticate: Basic realm="..."
2. ERROR: "Invalid response status: 401"
```

**Expected behavior:**
```
1. HEAD /File1 HTTP/1.1
   <- 401 Unauthorized
   <- WWW-Authenticate: Basic realm="..."
2. HEAD /File1 HTTP/1.1
   Authorization: Basic <credentials>
   <- 200 OK
3. GET /File1 HTTP/1.1
   Authorization: Basic <credentials>
   <- 200 OK
```

**Failing tests:**
- Test-auth-basic-netrc-pass-given.py - .netrc with --password override
- Test-auth-basic-netrc-user-given.py - .netrc with --user override
- Test-auth-basic-netrc.py - Pure .netrc authentication
- Test-auth-basic.py - Basic --user --password
- Test-auth-both.py - Multiple files with auth
- Test-auth-digest.py - Digest authentication
- Test-auth-no-challenge.py - Auth without WWW-Authenticate header
- Test-auth-with-content-disposition.py - Auth + Content-Disposition

**Passing auth tests (for comparison):**
- Test-auth-basic-fail.py ✓ - Wrong password (expects failure)
- Test-auth-basic-no-netrc-fail.py ✓ - No .netrc (expects failure)
- Test-auth-no-challenge-url.py ✓ - Credentials in URL (no HEAD)
- Test-auth-retcode.py ✓ - Exit code check only

**Root Cause Analysis:**

The auth retry logic in `wget-faster-lib/src/downloader.rs` currently works for GET requests but doesn't handle the HEAD request that happens first. When timestamping or checking file size, wget-faster sends:

1. HEAD request (to get Content-Length, Last-Modified)
2. GET request (to download content)

The HEAD request fails on 401 before auth can be applied.

**Fix Location:** `wget-faster-lib/src/downloader.rs`

```rust
// Current code (simplified):
let head_resp = client.head(url).send().await?;
if head_resp.status() == StatusCode::UNAUTHORIZED {
    return Err(Error::InvalidStatus(401)); // ← FAILS HERE
}

// Needed code:
let mut head_resp = client.head(url).send().await?;
if head_resp.status() == StatusCode::UNAUTHORIZED {
    // Check if we have credentials
    if let Some(auth) = &config.auth {
        // Parse WWW-Authenticate header
        let auth_header = parse_www_authenticate(&head_resp)?;

        // Retry HEAD with auth
        let mut req = client.head(url);
        req = apply_auth(req, auth, &auth_header)?;
        head_resp = req.send().await?;

        if head_resp.status() != StatusCode::OK {
            return Err(Error::AuthFailed);
        }
    } else {
        return Err(Error::AuthRequired); // Exit code 6
    }
}
```

**Implementation Steps:**

1. Extract auth retry logic into reusable function
2. Apply to both HEAD and GET requests
3. Handle Basic and Digest auth for HEAD
4. Ensure .netrc credentials are loaded before HEAD
5. Test with all 8 failing auth tests

**Estimated Time:** 1-2 hours
**Risk:** Low - Isolated to auth path
**Testing:** All 8 auth tests should pass after fix

---

### 3. RECURSIVE CRAWL ISSUES (6 tests, 9.1%) - PRIORITY 6

**Status:** BUGS - Multiple issues in recursive download
**Priority:** Medium
**Effort:** Medium (4-6 hours)
**Impact if fixed:** +4-6 tests (+5-7% improvement)

**Problems identified:**

#### 3a. Missing Page Requisites (Test-recursive-include.py)

**Issue:** `--include-directories=a` doesn't download page requisites (logo.svg)
**Expected:** a/logo.svg should be downloaded as a page requisite
**Actual:** File not found

**Fix:** `recursive.rs` - Ensure page requisites are downloaded even when directory filters are active.

#### 3b. Base Page Not Saved (Test--rejected-log.py)

**Issue:** index.html not saved when using `--rejected-log`
**Expected:** index.html downloaded, rejections logged to CSV
**Actual:** index.html missing

**Fix:** Implement `--rejected-log` option in CLI and recursive module.

#### 3c. Incomplete Crawling (Test--spider-r.py, Test-recursive-basic.py, Test-reserved-chars.py)

**Issue:** Not all linked pages crawled
**Expected:** All links in HTML should be followed
**Actual:** "Not all files were crawled correctly"

**Server logs show:**
```
GET / - followed ✓
GET /secondpage.html - followed ✓
GET /nonexistent - attempted (404) ✓
GET /robots.txt - missing ✗
GET /againnonexistent - missing ✗
GET /dummy.txt - missing ✗
```

**Fix:** `recursive.rs` - Improve link extraction, ensure all links are queued.

#### 3d. Long Path Handling (Test-recursive-pathmax.py)

**Issue:** Very long paths not downloaded in recursive mode
**Path length:** 200+ characters
**Fix:** Ensure path length limits don't truncate recursive downloads.

#### 3e. URL Encoding in Recursion (Test-reserved-chars.py)

**Issue:** URLs with special characters (+ encoded as %2B) not crawled
**Fix:** Proper URL encoding/decoding in link extraction.

**Estimated Time:** 4-6 hours total
**Risk:** Medium - Touches core recursive logic
**Testing:** 4-6 tests should pass after fixes

---

### 4. HTTP 504 EXIT CODE (1 test, 1.5%) - **PRIORITY 2**

**Status:** BUG - Wrong exit code for 5xx errors
**Priority:** **High - Very Easy Fix**
**Effort:** Very Easy (15 minutes)
**Impact if fixed:** +1 test (+1.2% improvement)

**Problem:** HTTP 5xx errors should exit with code 4 (server error), not 8 (HTTP error)

**Test:** Test-504.py
**Current:** Exit code 8
**Expected:** Exit code 4

**wget exit codes:**
- 4 = Network/server failure
- 6 = Authentication failure
- 8 = HTTP protocol error (4xx)

**Current behavior:**
```
HTTP 504 Gateway Timeout → exit 8 (wrong)
```

**Expected behavior:**
```
HTTP 500-599 → exit 4 (server error)
HTTP 400-499 → exit 8 (client error)
```

**Fix Location:** `wget-faster-lib/src/error.rs` and `wget-faster-cli/src/main.rs`

```rust
// In error.rs
pub fn exit_code(&self) -> i32 {
    match self {
        Error::InvalidStatus(code) => {
            match code {
                400..=499 => 8, // Client errors
                500..=599 => 4, // Server errors
                _ => 8,
            }
        }
        Error::NetworkError(_) => 4,
        Error::AuthFailed => 6,
        // ...
    }
}
```

**Estimated Time:** 15 minutes
**Risk:** Very Low
**Testing:** Test-504.py should pass

---

### 5. COOKIE EXPIRY HANDLING (1 test, 1.5%) - PRIORITY 4

**Status:** BUG - Expired cookies sent or non-expired cookies not sent
**Priority:** Medium
**Effort:** Easy (30-60 minutes)
**Impact if fixed:** +1 test (+1.2% improvement)

**Problem:** Cookies with expiry dates not handled correctly

**Test:** Test-cookie-expires.py
**Error:** "Expected Header Cookie not found" on File2, File3, File4

**Scenario:**
1. GET /File1 - Server sets cookie with expiry
2. GET /File2 - Cookie should be sent ✗ Missing
3. GET /File3 - Cookie should be sent ✗ Missing
4. GET /File4 - Cookie should be sent ✗ Missing

**Issue:** Cookie expiry comparison likely broken

**Fix Location:** `wget-faster-lib/src/cookies.rs`

```rust
// Check expiry comparison logic
pub fn is_expired(&self) -> bool {
    if let Some(expires) = self.expires {
        // Ensure correct time comparison
        SystemTime::now() > expires
    } else {
        false // No expiry = never expires
    }
}

// Ensure to_cookie_header includes non-expired cookies
pub fn to_cookie_header(&self, url: &Url) -> Option<String> {
    let cookies: Vec<String> = self.cookies.iter()
        .filter(|c| !c.is_expired()) // ← Check this filter
        .filter(|c| c.matches(url))
        .map(|c| format!("{}={}", c.name, c.value))
        .collect();
    // ...
}
```

**Testing areas:**
1. Parse `expires=` attribute correctly
2. Compare expiry time properly
3. Include non-expired cookies in requests
4. Exclude expired cookies

**Estimated Time:** 30-60 minutes
**Risk:** Low
**Testing:** Test-cookie-expires.py should pass

---

### 6. CONDITIONAL GET / IF-MODIFIED-SINCE (1 test, 1.5%) - PRIORITY 5

**Status:** BUG - If-Modified-Since not sent with timestamping
**Priority:** Medium
**Effort:** Medium (1-2 hours)
**Impact if fixed:** +1 test (+1.2% improvement)

**Problem:** When using `-N` (timestamping), If-Modified-Since header not sent

**Test:** Test-condget.py
**Error:** "Expected Header If-Modified-Since not found"

**Scenario:**
```
# File exists locally with mtime
GET /UpdatedFile HTTP/1.1
# ← Should include: If-Modified-Since: <file-mtime>
# ← Actually missing header
```

**Expected behavior:**
```
1. File exists locally: UpdatedFile (mtime: 2025-11-10 12:00:00)
2. Send HEAD with If-Modified-Since: Mon, 10 Nov 2025 12:00:00 GMT
3. Server responds:
   - 304 Not Modified → Don't download
   - 200 OK → Download (file updated)
```

**Fix Location:** `wget-faster-lib/src/downloader.rs`

```rust
// In timestamping logic
if config.timestamping && output_path.exists() {
    let metadata = fs::metadata(&output_path)?;
    let mtime = metadata.modified()?;

    // Add If-Modified-Since header
    let http_date = httpdate::fmt_http_date(mtime);
    request = request.header("If-Modified-Since", http_date);
}

// Handle 304 Not Modified
if response.status() == StatusCode::NOT_MODIFIED {
    // File not modified, skip download
    return Ok(());
}
```

**Note:** Current code has timestamping for Last-Modified response, but missing If-Modified-Since request header.

**Estimated Time:** 1-2 hours
**Risk:** Low-Medium
**Testing:** Test-condget.py should pass

---

### 7. PROXY ENVIRONMENT VARIABLES (1 test, 1.5%) - PRIORITY 3

**Status:** BUG - Proxy environment vars not respected
**Priority:** Medium
**Effort:** Easy (1 hour)
**Impact if fixed:** +1 test (+1.2% improvement)

**Problem:** http_proxy and no_proxy environment variables ignored

**Test:** Test-no_proxy-env.py
**Expected exit code:** 4 (can't reach proxy)
**Actual exit code:** 0 (success - bypassed proxy)

**Environment:**
```bash
http_proxy=nonexisting.localhost:8080
no_proxy=working1.localhost,.working2.localhost
URL: http://working2.localhost:54021/File1
```

**Expected behavior:**
- Domain `working2.localhost` in no_proxy → bypass proxy → succeed ✓
- Domain not in no_proxy → use proxy → fail (proxy unreachable) → exit 4

**Issue:** wget-faster ignores proxy environment variables entirely

**Fix Location:** `wget-faster-lib/src/client.rs`

```rust
use reqwest::Proxy;

pub fn new(config: &DownloadConfig) -> Result<Self> {
    let mut client_builder = reqwest::Client::builder();

    // Check environment variables
    if let Ok(proxy_url) = env::var("http_proxy").or(env::var("HTTP_PROXY")) {
        let proxy = Proxy::http(&proxy_url)?;

        // Check no_proxy
        if let Ok(no_proxy) = env::var("no_proxy").or(env::var("NO_PROXY")) {
            let no_proxy_list: Vec<&str> = no_proxy.split(',').collect();
            proxy = proxy.no_proxy(no_proxy_list);
        }

        client_builder = client_builder.proxy(proxy);
    }

    // https_proxy
    if let Ok(proxy_url) = env::var("https_proxy").or(env::var("HTTPS_PROXY")) {
        let proxy = Proxy::https(&proxy_url)?;
        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder.build()?;
    Ok(Self { client, config })
}
```

**Estimated Time:** 1 hour
**Risk:** Low
**Testing:** Test-no_proxy-env.py should pass

---

### 8. TLS/HTTPS ADVANCED FEATURES (10 tests, 15.2%) - DEFER

**Status:** Skipped (exit code 77) - TLS features not detected
**Priority:** Low (v0.2.0+)
**Impact if fixed:** +10 tests (+12.2% improvement)

All 10 tests exit with code 77 (test skipped), likely because:
1. Test framework checks `wgetf --version` for TLS features
2. wgetf doesn't advertise these features
3. Tests are skipped

**Failing tests:**
- Test--https-crl.py - Certificate Revocation List
- Test--https.py - Basic HTTPS
- Test-hsts.py - HTTP Strict Transport Security
- Test-https-k.py - HTTPS with insecure TLS
- Test-pinnedpubkey-*.py (6 tests) - Public key pinning

**Features needed:**
- Client certificates (TLS client auth)
- CRL checking
- Certificate pinning
- HSTS support
- Advanced TLS configuration

**Recommendation:** Defer to v0.2.0 - These are complex security features requiring significant work.

---

### 9. LINK CONVERSION (2 tests, 3.0%) - DEFER

**Status:** Missing feature `-k / --convert-links`
**Priority:** Low (planned v0.2.0)
**Impact if fixed:** +2 tests (+2.4% improvement)

**Tests:**
- Test--convert-links--content-on-error.py
- Test-k.py

Link conversion rewrites URLs in downloaded HTML to point to local files instead of remote URLs. This is a planned feature for v0.2.0.

---

### 10. OTHER ISSUES (4 tests, 6.1%)

#### Test-Proto.py - Protocol/Test Framework Issue
**Error:** Server exception on auth handling, test framework issue
**Priority:** Low - May be test setup problem

#### Test-Parallel-Proto.py - Missing Test Module
**Error:** `ModuleNotFoundError: No module named 'misc.constants'`
**Priority:** Low - Test environment issue

#### Test-redirect-crash.py - Redirect with URL Encoding
**Issue:** Complex URL with spaces and special chars in redirect
**Priority:** Low - Edge case

#### Test-recursive-redirect.py - Redirect in Recursive Mode
**Issue:** Redirect handling in recursive downloads
**Priority:** Low - Edge case

---

## Recommended Priority Order for v0.0.4

| Rank | Category | Tests | Effort | Impact | Cumulative Pass Rate |
|------|----------|-------|--------|--------|---------------------|
| 1 | Auth HEAD request retry | 8 | Easy | +9.8% | **29.3%** |
| 2 | HTTP 504 exit code fix | 1 | V.Easy | +1.2% | **30.5%** |
| 3 | Proxy environment vars | 1 | Easy | +1.2% | **31.7%** |
| 4 | Cookie expiry handling | 1 | Easy | +1.2% | **32.9%** |
| 5 | Conditional GET (If-Mod-Since) | 1 | Medium | +1.2% | **34.1%** |
| 6 | Recursive crawl improvements | 4-6 | Medium | +5-7% | **39-41%** |

**Quick wins (priorities 1-5):** 12 tests, +14.6% improvement → **34.1% pass rate**
**With recursive fixes:** 16-18 tests, +19.6-21.6% improvement → **39-41% pass rate**

---

## Comparison: Python vs Perl Test Suites

### Statistics

- **Python:** 82 tests, 16 passing (19.5%)
- **Perl:** 87 tests, 42 passing (48.3%)
- **Gap:** 28.8 percentage points

### Why Python Lags Behind Perl

1. **Metalink Concentration:** 32/82 Python tests are Metalink (39%), vs fewer in Perl
2. **Auth Issues:** Affect both equally, but Python has more auth test variations
3. **Test Framework:** Python tests are stricter about environment variables and edge cases

### Quick Win Impact

If priorities 1-5 are fixed:
- Python: 19.5% → 34.1% (+14.6%)
- Closes gap to Perl by ~50% (28.8% → 14.2%)

With recursive fixes:
- Python: 19.5% → 39-41% (+19.6-21.6%)
- Closes gap to Perl by ~65% (28.8% → ~7-9%)

---

## Actionable Recommendations

### For v0.0.4 (Target: 35-40% Python pass rate)

**Phase 1: Quick Wins (1-2 days)**
1. ✅ Fix auth HEAD request retry - **Highest priority** (+8 tests)
2. ✅ Fix HTTP 504 exit code - **15 minutes** (+1 test)
3. ✅ Add proxy environment support - **1 hour** (+1 test)
4. ✅ Fix cookie expiry logic - **30-60 min** (+1 test)

**Phase 2: Medium Effort (2-3 days)**
5. ✅ Conditional GET with If-Modified-Since - **1-2 hours** (+1 test)
6. ✅ Recursive crawl improvements - **4-6 hours** (+4-6 tests)

**Estimated Total Time:** 3-5 days
**Expected Result:** 35-41% Python pass rate (+15-21 tests)

### For v0.0.5+

- Link conversion `-k` (2 tests)
- Advanced TLS features (10 tests)
- Metalink support (32 tests) - Major feature

### Long-term (v0.2.0+)

- FTP/FTPS support
- IRI/IDN internationalization
- Full wget compatibility (95%+ pass rate)

---

## Testing Strategy

### Regression Testing
After each fix, run:
```bash
./test --wget-faster $(which wgetf) --test-type python -v
```

### Focused Testing
Test specific categories:
```bash
# Auth tests only
./test --test-type python | grep -i auth

# Recursive tests only
./test --test-type python | grep -i recursive
```

### Validation
Each fix should:
1. Pass the targeted failing tests
2. Not break any currently passing tests (regression check)
3. Update test count in CLAUDE.md

---

## Conclusion

The Python test suite analysis reveals **clear, actionable priorities** for improving wget-faster compatibility:

1. **Authentication HEAD retry** is the single biggest quick win (+8 tests, +9.8%)
2. **Small bug fixes** (exit codes, cookies, conditional GET, proxy) add another +4 tests (+4.8%)
3. **Recursive improvements** can add +4-6 more tests (+5-7%)

**Total realistic improvement for v0.0.4: 16-18 tests (+19.6-21.6%)**
**Target: 35-41% Python pass rate** (vs current 19.5%)

This would close the Python-Perl gap from 28.8% to ~7-9%, making wget-faster substantially more compatible with the official wget test suite.

The analysis also clearly identifies features to defer (Metalink, advanced TLS, link conversion) as they require significant effort and are planned for later versions.

---

**Report Generated:** 2025-11-14
**wget-faster Version:** v0.0.3
**Test Suite:** GNU wget Python tests (82 total)
**Analysis Depth:** All 66 failing tests categorized and prioritized

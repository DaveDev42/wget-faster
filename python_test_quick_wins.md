# Python Test Suite Quick Wins - wget-faster v0.0.4

**Current Status:** 16/82 tests passing (19.5%)
**Target for v0.0.4:** 35-41% pass rate (+15-21 tests)
**Estimated Effort:** 3-5 days

---

## Priority 1: Auth HEAD Request Retry ⭐⭐⭐

**Impact:** +8 tests (+9.8%) → 29.3% pass rate
**Effort:** 1-2 hours
**Difficulty:** Easy

### Problem
wget-faster sends HEAD request before authentication, fails on 401 instead of retrying with credentials.

### Current Behavior
```
HEAD /File1 → 401 Unauthorized → FAIL ❌
```

### Expected Behavior
```
HEAD /File1 → 401 Unauthorized
HEAD /File1 + Authorization → 200 OK ✓
GET /File1 + Authorization → 200 OK ✓
```

### Fix Location
`wget-faster-lib/src/downloader.rs` - Add auth retry to HEAD request path

### Failing Tests (8)
- Test-auth-basic-netrc-pass-given.py
- Test-auth-basic-netrc-user-given.py
- Test-auth-basic-netrc.py
- Test-auth-basic.py
- Test-auth-both.py
- Test-auth-digest.py
- Test-auth-no-challenge.py
- Test-auth-with-content-disposition.py

---

## Priority 2: HTTP 504 Exit Code ⭐⭐⭐

**Impact:** +1 test (+1.2%) → 30.5% pass rate
**Effort:** 15 minutes
**Difficulty:** Very Easy

### Problem
HTTP 5xx errors exit with code 8 (HTTP error) instead of code 4 (server error)

### Fix
```rust
// In error.rs
match status_code {
    400..=499 => 8, // Client errors
    500..=599 => 4, // Server errors ← Add this
}
```

### Failing Test
- Test-504.py

---

## Priority 3: Proxy Environment Variables ⭐⭐

**Impact:** +1 test (+1.2%) → 31.7% pass rate
**Effort:** 1 hour
**Difficulty:** Easy

### Problem
wget-faster ignores `http_proxy` and `no_proxy` environment variables

### Fix Location
`wget-faster-lib/src/client.rs` - Configure reqwest with proxy from env

### Fix
```rust
if let Ok(proxy_url) = env::var("http_proxy") {
    let proxy = Proxy::http(&proxy_url)?;
    // Handle no_proxy exceptions
    client_builder = client_builder.proxy(proxy);
}
```

### Failing Test
- Test-no_proxy-env.py

---

## Priority 4: Cookie Expiry Handling ⭐⭐

**Impact:** +1 test (+1.2%) → 32.9% pass rate
**Effort:** 30-60 minutes
**Difficulty:** Easy

### Problem
Cookies with expiry dates not sent on subsequent requests

### Fix Location
`wget-faster-lib/src/cookies.rs` - Fix expiry comparison logic

### Failing Test
- Test-cookie-expires.py

---

## Priority 5: Conditional GET (If-Modified-Since) ⭐

**Impact:** +1 test (+1.2%) → 34.1% pass rate
**Effort:** 1-2 hours
**Difficulty:** Medium

### Problem
When using `-N` (timestamping), If-Modified-Since header not sent

### Fix Location
`wget-faster-lib/src/downloader.rs` - Add If-Modified-Since to HEAD request

### Fix
```rust
if config.timestamping && output_path.exists() {
    let mtime = fs::metadata(&output_path)?.modified()?;
    let http_date = httpdate::fmt_http_date(mtime);
    request = request.header("If-Modified-Since", http_date);
}
```

### Failing Test
- Test-condget.py

---

## Priority 6: Recursive Crawl Improvements

**Impact:** +4-6 tests (+5-7%) → 39-41% pass rate
**Effort:** 4-6 hours
**Difficulty:** Medium

### Problems
1. Missing page requisites (logo.svg not downloaded with `-p`)
2. Base page not saved with `--rejected-log`
3. Incomplete crawling (not all links followed)
4. Long path handling
5. URL encoding in recursion

### Fix Location
`wget-faster-lib/src/recursive.rs` - Multiple improvements

### Failing Tests (6)
- Test--rejected-log.py
- Test--spider-r.py
- Test-recursive-basic.py
- Test-recursive-include.py
- Test-recursive-pathmax.py
- Test-reserved-chars.py

---

## Implementation Order

### Week 1: Quick Wins (Priorities 1-5)
**Days 1-2:**
1. ✅ Auth HEAD retry (2 hours) → +8 tests
2. ✅ HTTP 504 exit code (15 min) → +1 test
3. ✅ Proxy env vars (1 hour) → +1 test

**Day 3:**
4. ✅ Cookie expiry (1 hour) → +1 test
5. ✅ Conditional GET (2 hours) → +1 test

**Result:** 12 tests fixed, 34.1% pass rate ✓

### Week 2: Recursive Improvements (Priority 6)
**Days 4-5:**
6. ✅ Recursive fixes (4-6 hours) → +4-6 tests

**Result:** 16-18 tests fixed, 39-41% pass rate ✓

---

## Testing Commands

### Run all Python tests
```bash
cd ../wget-faster-test
./test --wget-faster $(which wgetf) --test-type python -v
```

### Test specific category
```bash
# Auth tests
./test --test-type python -v 2>&1 | grep -A5 "Test-auth"

# Recursive tests
./test --test-type python -v 2>&1 | grep -A5 "recursive"
```

### Validate fix
```bash
# Build and install
cargo build --release
cargo install --path wget-faster-cli

# Run tests
cd ../wget-faster-test
./test --wget-faster ~/.cargo/bin/wgetf --test-type python
```

---

## Success Metrics

| Milestone | Pass Rate | Tests Passing | Status |
|-----------|-----------|---------------|--------|
| Current (v0.0.3) | 19.5% | 16/82 | ✓ |
| After Priority 1 | 29.3% | 24/82 | Target |
| After Priorities 1-2 | 30.5% | 25/82 | Target |
| After Priorities 1-3 | 31.7% | 26/82 | Target |
| After Priorities 1-4 | 32.9% | 27/82 | Target |
| After Priorities 1-5 | 34.1% | 28/82 | **v0.0.4 Minimum** |
| After Priority 6 | 39-41% | 32-34/82 | **v0.0.4 Stretch Goal** |

---

## What NOT to Fix (Defer)

### Metalink Support (32 tests)
**Reason:** Major feature requiring new protocol support
**Defer to:** v0.2.0+

### Advanced TLS (10 tests)
**Reason:** Complex security features (client certs, CRL, pinning)
**Defer to:** v0.2.0+

### Link Conversion -k (2 tests)
**Reason:** Planned feature for v0.2.0
**Defer to:** v0.2.0

---

## Risk Assessment

| Priority | Risk Level | Reason |
|----------|------------|--------|
| 1 - Auth HEAD | Low | Isolated to auth path, clear fix |
| 2 - Exit code | Very Low | One-line change |
| 3 - Proxy env | Low | Well-defined reqwest API |
| 4 - Cookie expiry | Low | Isolated to cookie module |
| 5 - Conditional GET | Medium | Touches core download flow |
| 6 - Recursive | Medium | Core recursive logic changes |

---

## Expected Outcomes

### Immediate (v0.0.4)
- Python pass rate: **34.1%** minimum, **39-41%** stretch
- Gap to Perl: Reduced from 28.8% to ~14% (minimum) or ~7-9% (stretch)
- wget compatibility: Significantly improved for common use cases

### Next Steps (v0.0.5+)
- Link conversion `-k` (+2 tests)
- Additional edge cases
- Target: 45-50% pass rate

### Long-term (v0.2.0+)
- Metalink support (+32 tests)
- Advanced TLS (+10 tests)
- Target: 75-80% pass rate

---

**Last Updated:** 2025-11-14
**Version:** v0.0.3 analysis for v0.0.4 planning

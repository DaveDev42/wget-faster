# v0.0.7 Python Test Results Analysis

## Summary

**Test Run Date**: 2025-11-14 18:58:42
**Total Tests**: 82 Python tests
**Passed**: 17/82 (20.7%)
**Failed**: 65/82 (79.3%)

**Expected**: 32-34/82 (39-41%) based on v0.0.5/v0.0.6 fixes
**Actual**: 17/82 (20.7%)
**Gap**: -15 to -17 tests below expectation

## Failure Categories

| Category | Count | Percentage |
|----------|-------|------------|
| missing_feature | 32 | 49.2% |
| test_framework_error | 19 | 29.2% |
| skipped | 10 | 15.4% |
| timeout | 3 | 4.6% |
| import_error | 1 | 1.5% |

## Passing Tests (17)

✅ Test-416.py - HTTP 416 Range Not Satisfiable
✅ Test-Content-disposition-2.py - Content-Disposition with existing file
✅ Test-Content-disposition.py - Basic Content-Disposition
✅ Test-Head.py - HEAD method
✅ Test-O.py - Output filename (-O)
✅ Test-Post.py - HTTP POST
✅ Test-auth-basic-fail.py - Basic auth failure
✅ Test-auth-basic-no-netrc-fail.py - Auth without .netrc
✅ Test-auth-no-challenge-url.py - Preemptive auth (URL)
✅ Test-auth-no-challenge.py - Preemptive auth (flag)
✅ Test-auth-retcode.py - Auth return codes
✅ Test-c-full.py - Resume fully downloaded file
✅ Test-cookie-401.py - Cookies with 401
✅ Test-cookie-domain-mismatch.py - Cookie domain filtering
✅ Test-cookie.py - Basic cookie support
✅ Test-missing-scheme-retval.py - Missing URL scheme error
✅ Test-redirect.py - HTTP redirects

## Impact of v0.0.5/v0.0.6 Fixes

### Priority 28: Auth Preemptive Behavior (v0.0.6)
**Status**: PARTIAL SUCCESS
- **Expected improvement**: +5 tests
- **Actual improvement**: +2 tests (auth-no-challenge.py, auth-no-challenge-url.py passing)
- **Still failing**: 7/12 auth tests
  - 3 timeout (auth-basic.py, auth-basic-netrc.py, auth-with-content-disposition.py)
  - 4 test_framework_error

**Issue identified**: Test framework errors suggest implementation bugs, not just preemptive auth

### Priority 29: Recursive CLI Mapping (v0.0.6)
**Status**: FAILING
- **Expected improvement**: +2-3 tests
- **Actual improvement**: 0 tests
- **Test--rejected-log.py**: FAILING
  - Error: "Expected file robots.txt not found"
  - Issue: Missing robots.txt support in recursive downloader

### Priority 26: Conditional GET (v0.0.5)
**Status**: FAILING
- **Expected improvement**: +1 test
- **Actual improvement**: 0 tests
- **Test-condget.py**: FAILING
  - Error: GET request missing If-Modified-Since header
  - Issue: Header only sent on HEAD, not on subsequent GET

### Priority 23: HTTP 504 Exit Code (v0.0.5)
**Status**: PARTIAL - Exit code correct but retry broken
- **Expected improvement**: +1 test
- **Actual improvement**: 0 tests
- **Test-504.py**: FAILING
  - Exit code 4 is correct (was priority goal)
  - But test expects retry with `--tries=2` flag
  - Issue: 504 errors not being retried

## Critical Bugs Discovered

### Bug 1: Conditional GET broken for actual downloads
**File**: downloader.rs
**Issue**: If-Modified-Since header sent on HEAD but not on subsequent GET request
**Impact**: Test-condget.py failing
**Fix needed**: Send If-Modified-Since on GET request too

### Bug 2: robots.txt not fetched in recursive mode  
**File**: recursive.rs
**Issue**: Recursive downloader doesn't check/download robots.txt
**Impact**: Test--rejected-log.py failing
**Fix needed**: Implement robots.txt fetching and respect

### Bug 3: 504 errors not retried
**File**: downloader.rs
**Issue**: Server errors (5xx) should be retried with `--tries` flag
**Impact**: Test-504.py failing  
**Fix needed**: Add retry logic for 5xx status codes

### Bug 4: Auth timeouts in Python tests
**Tests**: auth-basic.py, auth-basic-netrc.py, auth-with-content-disposition.py
**Issue**: Tests timing out (>60s) when they should complete quickly
**Root cause**: Unknown - needs investigation
**Impact**: 3 auth tests failing

## Feature Gaps (32 missing_feature tests)

### Metalink support (32 tests)
- All metalink tests failing with "unexpected argument '--input-metalink'"
- Feature not implemented (planned for v0.2.0+)
- Impact: 32 tests = 39% of all Python tests

### Advanced HTTPS/TLS (10 skipped tests)
- Tests skipped due to SSL/TLS configuration
- Exit code 77 (feature not available)
- Includes: CRL, pinned public keys, certificate validation

## Comparison with v0.0.4 Results

**v0.0.4 (before fixes)**: 16/82 Python tests (19.5%)
**v0.0.7 (after fixes)**: 17/82 Python tests (20.7%)
**Net improvement**: +1 test (+1.2%)

**Why so low?**
1. Conditional GET implementation incomplete (GET requests broken)
2. robots.txt support not implemented
3. 504 retry logic broken
4. Auth tests have new timeout issues
5. Metalink feature not implemented (32 tests lost)

## Recommended Actions for v0.0.8

### High Priority (Quick Wins - Est. +5-7 tests)

1. **Fix Conditional GET for GET requests** (1-2 hours)
   - Send If-Modified-Since on GET, not just HEAD
   - Expected: +1 test (Test-condget.py)

2. **Fix 504 retry logic** (1-2 hours)
   - Retry 5xx errors according to `--tries` flag
   - Expected: +1 test (Test-504.py)

3. **Implement basic robots.txt support** (3-4 hours)
   - Fetch /robots.txt in recursive mode
   - Respect basic rules (User-agent, Disallow)
   - Expected: +1 test (Test--rejected-log.py)

4. **Investigate auth timeouts** (2-3 hours)
   - Debug why 3 auth tests timeout
   - May be test server issues or client bugs
   - Expected: +3 tests if fixed

### Medium Priority (Larger Fixes - Est. +10-15 tests)

5. **Implement more recursive features** (4-6 hours)
   - Fix link extraction edge cases
   - Implement missing recursive options
   - Expected: +5-8 tests

6. **Fix link conversion bugs** (2-3 hours)
   - Relative path calculation issues
   - Expected: +2-3 tests

### Low Priority (Deferred to v0.2.0+)

7. **Metalink support** (20+ hours major feature)
   - Would unlock 32 tests
   - Deferred - not critical for current goals

## Updated Pass Rate Estimate

**Current**: 17/82 (20.7%)
**After High Priority fixes**: 22-24/82 (27-29%)
**After Medium Priority fixes**: 32-39/82 (39-48%)

**Recommendation**: Focus on high-priority bugs first to reach 27-29% Python pass rate by v0.0.8.

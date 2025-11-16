# Session 4: HEAD/GET Request Pattern Investigation

**Date**: 2025-11-17
**Status**: Investigation complete, no code changes committed
**Test Status**: 72/151 passing (maintained from Session 3)

## Summary

Investigated Test-504.py and Test--spider-r.py failures related to HEAD request behavior. Found that fundamental architectural differences between wget-faster's performance optimizations (HEAD before GET) and GNU wget's sequential-only approach create test compatibility challenges.

## Investigation: Test-504.py (HTTP 504 Gateway Timeout)

### Test Expectations
- Expects `--tries=2` to retry 504 errors
- Expected request sequence: `GET /File1`, `GET /File1` (retry), `GET /File2`
- Expected only GET requests (no HEAD)

### Current Behavior
- wget-faster sends HEAD+GET for each attempt by default
- Actual sequence: `HEAD /File1`, `GET /File1`, `HEAD /File1`, `GET /File1`, `HEAD /File2`, `GET /File2`
- This breaks the test's `FilesCrawled` validation

### Solution Attempt: `gnu_wget_compat=true` Default
**Result**: Test-504.py PASSES ✓, but 5 auth tests FAIL ✗

**Test Results**:
- With `gnu_wget_compat=false`: 72/151 passing
- With `gnu_wget_compat=true`: 67/151 passing (regression of 5 tests)

**Failing Auth Tests**:
1. Test-auth-basic-fail.py (timeout)
2. Test-auth-basic-netrc-pass-given.py
3. Test-auth-basic-netrc-user-given.py (timeout)
4. Test-auth-basic-netrc.py (timeout)
5. Test-auth-basic.py (timeout)

### Root Cause: Authentication State Tracking

The auth test failures reveal a critical dependency:

**File**: `wget-faster-lib/src/client.rs:338`
```rust
// Only HEAD requests populate authenticated_hosts after successful auth
self.authenticated_hosts.lock().unwrap().insert(h);
```

**File**: `wget-faster-lib/src/downloader.rs:815-849`
```rust
// GET requests handle 401 retry but DON'T populate authenticated_hosts
// This creates an auth loop when HEAD requests are disabled
```

**Flow with `gnu_wget_compat=false` (WORKS)**:
1. HEAD request with auth → 200 OK → populate `authenticated_hosts` set
2. GET request uses preemptive auth (from `authenticated_hosts`) → 200 OK
3. Subsequent requests use preemptive auth → no 401 challenges

**Flow with `gnu_wget_compat=true` (BREAKS AUTH)**:
1. Skip HEAD, go directly to GET without auth
2. GET → 401 Unauthorized
3. Retry GET with auth → 200 OK
4. BUT `authenticated_hosts` is never populated
5. Next request starts over: GET without auth → 401 → ...
6. Creates infinite 401 loop or timeout

## Key Insights

### 1. HEAD Requests Serve Dual Purpose
- **Performance**: Check Range support for parallel downloads
- **Auth Tracking**: Establish authentication state for preemptive auth

### 2. GNU wget Compatibility Trade-offs
- Disabling HEAD requests matches GNU wget behavior for some tests
- BUT breaks authentication state management
- Creating a "compatibility mode" requires rearchitecting auth handling

### 3. Proper Fix Requires
1. Move `authenticated_hosts` population to GET request auth retry path
2. Ensure this doesn't create new bugs
3. Test thoroughly across all 151 tests
4. This is beyond "quick wins" scope

## Recommendations

### Short-term (Current Session)
- **Keep `gnu_wget_compat=false` as default** (preserves 72/151)
- Mark Test-504.py as "requires architecture changes"
- Focus on tests that don't require fundamental behavioral changes

### Long-term (Future Work)
1. **Implement auth state tracking in GET requests**:
   - Add `authenticated_hosts` population after successful GET auth retry
   - File: `wget-faster-lib/src/downloader.rs:815-849`

2. **Consider hybrid approach**:
   - Use HEAD requests for non-auth scenarios
   - Skip HEAD when credentials are configured
   - Requires careful testing

3. **Test-specific solutions**:
   - Some tests may need adjusted expectations
   - Document wget-faster's performance optimizations vs strict compatibility

## Files Modified (Reverted)

All changes were reverted to maintain 72/151 test count:

1. `wget-faster-lib/src/config.rs:205`
   - Changed `gnu_wget_compat: false` → `true` → reverted to `false`

2. `wget-faster-cli/src/main.rs:895-900`
   - Modified to preserve library default → reverted

## Lessons Learned

1. **Global config changes are risky**: Changing defaults can have cascading effects
2. **Auth state is fragile**: Distributed across HEAD and GET request paths
3. **Performance vs compatibility**: Core tension in wget-faster design
4. **Test thoroughly before committing**: The 5-test regression was caught before commit

## Next Steps

- Investigate simpler tests from Priority 1 list
- Look for tests failing due to missing CLI flags or simple logic errors
- Avoid tests requiring fundamental architectural changes

---

**Conclusion**: Test-504.py and similar tests require architectural changes to auth handling. Current approach (HEAD for performance, auth tracking) conflicts with strict GNU wget compatibility (GET-only, no HEAD). Solution requires carefully refactoring auth state management across both request types.

# TODO - wget-faster Development Roadmap

**Current Version**: v0.0.4
**Test Coverage**: 73/151 tests (48.3%) - Session 5 improvement
**Last Updated**: 2025-11-17

---

## 🎯 Next Steps - v0.0.5

**Goal**: Incremental test improvements (target: 75-85 tests, 44-50%)
**Strategy**: Fix one test at a time, verify immediately, commit only if no regression

### Priority 1: Architectural Changes Required (5-10 hours each)

**⚠️ All Priority 1 items require structural changes - not quick wins**

1. **Test-504.py** - HTTP 504 Gateway Timeout
   - **Status**: Passes with `gnu_wget_compat=true` BUT breaks 5 auth tests
   - **Issue**: HEAD request behavior conflicts with auth state tracking
   - **Requires**: Refactor auth state management (see docs/SESSION_4_FINDINGS.md)
   - Files: `client.rs:338`, `downloader.rs:815-849`

2. **Test--spider-r.py** - Extra HEAD requests in spider mode
   - **Issue**: Spider mode sends extra HEAD requests vs GNU wget
   - **Requires**: Fine-tuned HEAD request logic in spider mode
   - Files: `recursive.rs`, `downloader.rs`

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

8. **Cookie handling edge cases**
   - Test-cookie-expires.py (cookie expiry and persistence)
   - Files: `cookies.rs`, `client.rs`

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

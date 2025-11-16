# TODO - wget-faster Development Roadmap

**Current Version**: v0.0.4
**Current Status**: 68/169 tests (40.2%)
**Last Updated**: 2025-11-16

---

## 📊 Current Status (2025-11-16)

### Test Results
- **Total**: 68/169 tests passing (40.2%)
- **Perl**: 45/87 tests passing (51.7%)
- **Python**: 23/82 tests passing (28.0%)
- **Baseline**: STABLE ✅ (no regression)

### Maximum Achievable
- **Total tests**: 169
- **Excluded features** (user decision):
  - Metalink: 32 tests (--input-metalink not implemented)
  - FTP: ~14 tests (not implementing per user request)
  - Advanced SSL/TLS: ~10 tests (client certs, CRL, pinned keys)
- **Maximum realistic**: ~113/169 (66.9%)
- **Fixable tests**: ~45 remaining

---

## 🎯 Active Development - v0.0.5 Test Improvements

**Goal**: Incremental test improvements through targeted bug fixes
**Strategy**: Fix one test at a time, verify immediately, commit only if no regression

### Priority 1: Individual Bug Fixes (Target: +5-10 tests)

Fix tests one at a time with minimal changes:

1. **Test-504.py** - HTTP 504 Gateway Timeout handling
   - Issue: Need proper 5xx retry logic
   - Expected: Exit code 4 for server errors
   - Files: `downloader.rs`
   - Estimate: 1-2 hours

2. **Test--spider-r.py** - Extra HEAD requests in spider mode
   - Issue: FilesCrawled mismatch due to HEAD requests
   - Expected: Skip HEAD in specific spider scenarios
   - Files: `recursive.rs`, `downloader.rs`
   - Estimate: 1-2 hours

3. **Test-no_proxy-env.py** - Proxy environment variable handling
   - Issue: Specific proxy bypass scenarios
   - Expected: Respect no_proxy patterns
   - Files: `client.rs`, `config.rs`
   - Estimate: 1 hour

4. **Test-reserved-chars.py** - URL encoding in recursive spider
   - Issue: Reserved character handling
   - Expected: Proper URL encoding/decoding
   - Files: `recursive.rs`
   - Estimate: 1-2 hours

5. **Test-redirect-crash.py** - Redirect with special characters
   - Issue: URL path encoding issues
   - Expected: Handle encoded paths correctly
   - Files: `client.rs`, `downloader.rs`
   - Estimate: 1-2 hours

### Priority 2: Pattern-Based Fixes (Target: +10-15 tests)

After fixing 3-5 individual tests, identify common patterns:

6. **Recursive download improvements**
   - Test-recursive-include.py
   - Test-recursive-pathmax.py
   - Test-recursive-redirect.py
   - Issue: Various robots.txt and request sequence issues
   - Files: `recursive.rs`
   - Estimate: 3-5 hours

7. **Authentication improvements**
   - Test-auth-basic-netrc-pass-given.py
   - Test-auth-basic-netrc-user-given.py
   - Test-auth-both.py
   - Test-auth-digest.py
   - Issue: Auth credential handling edge cases
   - Files: `client.rs`, `downloader.rs`
   - Estimate: 3-4 hours

8. **Cookie handling edge cases**
   - Test-cookie-expires.py
   - Issue: Cookie expiry and persistence
   - Files: `cookies.rs`, `client.rs`
   - Estimate: 2-3 hours

### Priority 3: Complex Features (Target: +10-15 tests)

9. **Link conversion (-k) improvements**
   - Test-k.py
   - Test--convert-links--content-on-error.py
   - Issue: Link conversion edge cases
   - Files: `link_converter.rs`, `recursive.rs`
   - Estimate: 4-6 hours

10. **Rejected log format**
    - Test--rejected-log.py
    - Issue: CSV format for rejected URLs
    - Files: `recursive.rs`
    - Estimate: 2-3 hours

---

## 📝 Test Failure Analysis Workflow

### Every Session Workflow

1. **Run full test suite**
   ```bash
   cd ../wget-faster-test
   ./run_tests.sh --wget-faster $(which wgetf) --include-ftp
   ```

2. **Analyze results** (append to this file, keep under 2000 lines)
   ```bash
   # Count by category
   # Document new failures
   # Identify patterns
   ```

3. **Pick ONE test to fix**
   - Choose based on:
     - Estimated effort (start with 1-2 hour fixes)
     - Test failure message clarity
     - Related to recent work

4. **Fix → Test → Commit (or Revert)**
   ```bash
   # Make minimal fix
   cargo build --release
   cargo install --path wget-faster-cli

   # Test immediately
   cd ../wget-faster-test
   ./run_tests.sh --wget-faster $(which wgetf) --include-ftp

   # If improved or maintained: commit
   # If regression: revert
   git checkout <files>
   ```

5. **Update this file**
   - Mark completed items
   - Add new discoveries
   - Keep total under 2000 lines

---

## 🚫 Excluded Features (Will NOT Implement)

### Metalink (32 tests)
- **Reason**: Complex XML parsing + multi-mirror management
- **User decision**: Not essential for drop-in replacement
- **Impact**: ~19% of total tests

### FTP/FTPS (14 tests)
- **Reason**: Different protocol, major implementation effort
- **User decision**: "FTP 지원은 하지 말자"
- **Impact**: ~8% of total tests

### Advanced SSL/TLS (10 tests)
- **Reason**: Client certificates, CRL, pinned keys
- **User decision**: Low priority, niche use cases
- **Impact**: ~6% of total tests

**Total excluded**: ~56 tests (33% of suite)

---

## ⚠️ Lessons Learned

### What NOT to Do

1. **Don't make broad heuristic changes**
   - Session 1: --no-parallel mode → -9 tests regression
   - Session 1: HEAD optimization → -9 tests regression
   - Session 2: robots.txt disable → -2 tests regression
   - **Lesson**: Assumptions without understanding exact GNU wget behavior = regressions

2. **Don't guess wget behavior**
   - Example: robots.txt (some tests expect it, others don't)
   - **Lesson**: Read test source, run individually, understand exact expectations

3. **Don't batch multiple changes**
   - **Lesson**: One change at a time, test immediately

### What TO Do

1. **Read test source first**
   - Understand expected behavior
   - Check Request_List, ExpectedDownloadedFiles
   - Identify exact failure point

2. **Run test individually**
   - See exact failure mode
   - Compare with GNU wget if needed
   - Understand root cause before fixing

3. **Make minimal changes**
   - Smallest possible fix
   - Test immediately
   - Commit only if no regression

4. **Build knowledge incrementally**
   - Document findings in this file
   - Share patterns across similar tests
   - Create references for future work

---

## 📚 Test Analysis Summary (Latest: 2025-11-16)

### Failure Categories

| Category | Count | Fixable? | Notes |
|----------|-------|----------|-------|
| missing_feature (Metalink) | 32 | ❌ No | Excluded per user |
| missing_feature (FTP) | 14 | ❌ No | Excluded per user |
| skipped (SSL/TLS) | 10 | ❌ No | Advanced features |
| test_framework_error | 18 | ✅ Yes | Implementation bugs |
| Other failures | ~27 | ✅ Yes | Various issues |

### Estimated Improvement Potential

- **Quick wins** (1-2 hours each): ~10 tests
- **Medium effort** (3-5 hours each): ~15 tests
- **Complex** (5+ hours each): ~20 tests

**Realistic targets**:
- **Next 2 weeks**: 75-85 tests (44-50%)
- **Next month**: 85-100 tests (50-59%)
- **Maximum achievable**: ~113 tests (66.9%)

---

## 🔧 Development Guidelines

### Before Every Change

1. Read test source code
2. Run test individually
3. Understand exact failure
4. Plan minimal fix

### After Every Change

1. Build release version
2. Install locally
3. Run FULL test suite
4. Check for regressions
5. Commit only if improved or stable

### Code Quality

- No `unwrap()` in library code
- Use `?` operator for error propagation
- Add rustdoc comments for public APIs
- Match GNU wget behavior exactly
- Document any intentional differences

---

## 📦 Completed Features

### v0.0.4 - HEAD Request Optimization ✅
- Skip HEAD requests when parallel downloads disabled
- Optimized `is_html_url()` extension checking
- Added `--no-parallel` flag
- **Result**: 68/169 tests (40.2%)

### v0.0.3 - Core Functionality ✅
- Timestamping (-N) with all edge cases
- Content-Disposition header parsing
- Resume/continue (-c)
- Exit codes (3, 6, 8)
- Spider mode (--spider)
- Recursive downloads
- **Result**: 61/169 tests (36.1%)

---

## 🔮 Future Versions (Not Planned Yet)

### v0.1.0 - Performance & HTTP/3
- HTTP/3 (QUIC) support
- Zero-copy chunk assembly
- io_uring on Linux (optional)
- Performance benchmarks

### v0.2.0 - Advanced Features
- Link conversion improvements
- robots.txt compliance
- WARC format support
- Advanced recursive options

### v1.0.0 - Production Ready
- 95%+ wget compatibility (excluding FTP/Metalink)
- Security audit
- Man pages
- Shell completions
- Package distribution

---

## 📊 Session Results Log

### 2025-11-16 Session 3 (Continuation)

**Attempted**: Test-Proto.py + auth handling
**Result**: REVERTED - regression to 64/151 (-4 auth tests)
**Lesson**: Auth logic is complex, interacts with HEAD requests
**Baseline**: 68/151 restored ✅

**Tests analyzed** (all too complex for simple fixes):
1. **Test-504.py** - HEAD requests interfere with retry logic (architectural issue)
2. **Test-Proto.py** - Test server bug + complex Digest auth on HEAD requests
3. **Test-no_proxy-env.py** - Complex proxy bypass with dot-prefixed domains
4. **Test-cookie-expires.py** - Cookie expiry management edge cases

**Key findings**:
- Attempted to handle auth failures on HEAD by continuing to GET
- Broke 4 tests expecting auth to FAIL: Test-cookies-401.px, Test-auth-basic-fail.py, Test-auth-basic-no-netrc-fail.py, Test-cookie-401.py
- Auth logic needs careful design: tests expect BOTH success and failure scenarios
- HEAD request changes affect: auth flow, cookie handling, proxy detection

**Remaining test categories**:
- test_framework_error (16 tests) - All analyzed, all complex
- Requires: link conversion (-k), proxy improvements, cookie management, URL encoding
- Quick wins exhausted - remaining fixes require multi-hour feature development

**Reality check**:
- Current: 68/151 tests (45.0%)
- Maximum realistic: ~113/169 (66.9%) after FTP/Metalink/SSL exclusions
- Remaining ~45 fixable tests require significant implementation effort
- Each fix: ~2-6 hours of analysis + implementation + testing

**Next session strategy**:
- Accept that remaining improvements require larger features
- Consider: Link conversion (-k flag), proxy improvements, cookie enhancements
- Or: Focus on performance/HTTP/3 instead of test compatibility

### 2025-11-16 Session 2

**Attempted**: robots.txt disable by default
**Result**: REVERTED - regression to 66/169 (-2 tests)
**Lesson**: Some tests expect robots.txt, others don't
**Baseline**: 68/169 restored ✅

**Key findings**:
- Test-recursive-basic.py EXPECTS robots.txt
- Test-recursive-include.py DOES NOT expect robots.txt
- Need to understand WHEN GNU wget fetches robots.txt
- Broad assumptions cause regressions

### 2025-11-15 Session 1

**Attempted**: --no-parallel mode and HEAD optimization
**Result**: Both reverted due to regressions
**Baseline**: 68/169 maintained ✅

**Key findings**:
- Parallel downloads provide 68 test baseline
- HEAD requests needed for some tests
- Cannot broadly disable without understanding which tests need what

---

**End of TODO.md - Keep this file under 2000 lines total**
**For detailed test analysis, see test reports in ../wget-faster-test/reports/**

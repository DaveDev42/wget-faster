# TODO - v0.0.1 MVP Tasks

**Goal**: Create a working, wget-compatible HTTP/HTTPS downloader that can replace GNU wget for 80% of common use cases.

**Timeline**: v0.0.1 MVP → v0.0.2 Performance → v0.0.3 Advanced → v0.0.4 Extended → v0.0.5 HTTP/3 → Future minor version

---

## v0.0.1 MVP - Critical Path

**Target**: Feature-complete wget replacement for basic downloads
**Test Goal**: Pass 60%+ of wget core test suite
**Scope**: HTTP/HTTPS only (no FTP, no recursive, no WARC)

### Phase 1: Core Functionality Verification (Week 1)

**Objective**: Verify existing features work correctly with wget-compatible behavior

#### 1.1 Basic Downloads
- [ ] **Test basic HTTP GET downloads**
  - Verify file download works
  - Test with various file sizes (1KB, 1MB, 100MB)
  - Ensure output matches wget behavior
  - Files: `wget-faster-lib/tests/integration_tests.rs`

- [ ] **Test HTTPS downloads**
  - Verify TLS 1.2/1.3 support
  - Test certificate verification
  - Test `--no-check-certificate` flag
  - Files: `wget-faster-lib/src/client.rs`

- [ ] **Test output to file (`-O`)**
  - Verify `-O filename` works
  - Test `-O -` (stdout)
  - Test overwrite behavior
  - Files: `wget-faster-cli/src/main.rs`

#### 1.2 Resume & Retry
- [ ] **Test resume functionality (`-c`)**
  - Partial download + resume works
  - Range header sent correctly
  - File size verification
  - Files: `wget-faster-lib/src/downloader.rs`

- [ ] **Test retry logic (`-t`)**
  - Retry count respected
  - Exponential backoff working
  - Network error retry
  - 5xx error retry
  - Files: `wget-faster-lib/src/downloader.rs`

#### 1.3 Authentication & Headers
- [ ] **Test HTTP Basic auth**
  - `--http-user` and `--http-password` work
  - Authorization header sent
  - 401 handling
  - Files: `wget-faster-lib/src/client.rs`

- [ ] **Test HTTP Digest auth**
  - Digest challenge-response works
  - Files: `wget-faster-lib/src/client.rs`

- [ ] **Test custom headers**
  - `--header` option works
  - Multiple headers supported
  - User-Agent customization (`-U`)
  - Referer header (`--referer`)
  - Files: `wget-faster-lib/src/client.rs`

#### 1.4 Cookies
- [ ] **Implement cookie file loading**
  - Read Netscape cookie format
  - Parse domain, path, secure, expiry
  - Apply cookies to requests
  - `--load-cookies` option
  - Files: `wget-faster-lib/src/cookies.rs`

- [ ] **Implement cookie file saving**
  - Write Netscape cookie format
  - Save Set-Cookie responses
  - `--save-cookies` option
  - Files: `wget-faster-lib/src/cookies.rs`

- [ ] **Test cookie behavior**
  - Domain matching
  - Path matching
  - Secure flag
  - Expiry handling
  - Files: `wget-faster-lib/tests/cookie_tests.rs`

### Phase 2: Output & UI (Week 2)

#### 2.1 wget-Style Output Format
- [ ] **Implement connecting message**
  - "Connecting to example.com:443... connected."
  - Show resolved IP if verbose
  - Files: `wget-faster-cli/src/output.rs`

- [ ] **Implement HTTP response display**
  - "HTTP request sent, awaiting response... 200 OK"
  - Show status code and reason
  - Files: `wget-faster-cli/src/output.rs`

- [ ] **Implement saving message**
  - "Saving to: 'filename'"
  - "Length: 12345 (12K) [text/html]"
  - Files: `wget-faster-cli/src/output.rs`

- [ ] **Implement completion message**
  - "Downloaded: 1 files, 12K in 0.5s (24 KB/s)"
  - Files: `wget-faster-cli/src/output.rs`

#### 2.2 Progress Display
- [ ] **Implement wget-style progress bar**
  - Format: `12% [==>         ] 1,234 1.23MB/s eta 5s`
  - Update frequency control
  - Files: `wget-faster-cli/src/output.rs`

- [ ] **Test progress modes**
  - Default progress bar
  - Quiet mode (`-q`) - no output
  - Verbose mode (`-v`) - detailed output
  - No-verbose (`-nv`) - minimal output
  - Files: `wget-faster-cli/src/main.rs`

#### 2.3 Server Response Display
- [ ] **Implement `-S, --server-response`**
  - Print all HTTP headers
  - Format like wget
  - Files: `wget-faster-cli/src/output.rs`

### Phase 3: POST Requests & HTTP Methods (Week 2)

#### 3.1 POST Requests
- [ ] **Wire up `--post-data`**
  - Parse POST data from CLI
  - Set Content-Type header
  - Send POST request
  - Files: `wget-faster-cli/src/main.rs`, `wget-faster-lib/src/config.rs`

- [ ] **Wire up `--post-file`**
  - Read data from file
  - Send as POST body
  - Files: `wget-faster-cli/src/main.rs`

- [ ] **Test POST requests**
  - POST with data
  - POST with file
  - Content-Type handling
  - Files: `wget-faster-lib/tests/integration_tests.rs`

### Phase 4: Multiple URLs & Timestamping (Week 3)

#### 4.1 Multiple URL Support
- [ ] **Implement multiple URL downloads**
  - Accept multiple URLs from CLI
  - Download sequentially
  - Track failures
  - Continue on error (default wget behavior)
  - Files: `wget-faster-cli/src/main.rs`

- [ ] **Implement input file (`-i`)**
  - Read URLs from file
  - One URL per line
  - Ignore comments (#)
  - Files: `wget-faster-cli/src/main.rs`

- [ ] **Test multiple downloads**
  - Multiple CLI arguments
  - Input file
  - Error handling
  - Files: `wget-faster-cli/tests/`

#### 4.2 Timestamping
- [ ] **Test timestamping (`-N`)**
  - If-Modified-Since header
  - Local file timestamp comparison
  - Skip if up-to-date
  - Files: `wget-faster-lib/src/downloader.rs`

- [ ] **Set downloaded file timestamp**
  - Use Last-Modified header
  - Set mtime on local file
  - Files: `wget-faster-lib/src/downloader.rs`

### Phase 5: Timeouts & Speed Limiting (Week 3)

#### 5.1 Timeout Configuration
- [ ] **Test timeout options**
  - `-T, --timeout` sets all timeouts
  - `--connect-timeout` separate
  - `--read-timeout` separate
  - Files: `wget-faster-lib/src/config.rs`

- [ ] **Verify timeout behavior**
  - Connection timeout works
  - Read timeout works
  - Timeout triggers retry
  - Files: `wget-faster-lib/tests/integration_tests.rs`

#### 5.2 Speed Limiting
- [ ] **Test speed limiting**
  - `--limit-rate` option
  - Parse human-readable rates (1k, 1m)
  - Actual speed stays within limit
  - Files: `wget-faster-lib/src/client.rs`

### Phase 6: Testing & Validation (Week 4)

#### 6.1 Unit Tests
- [ ] **Write comprehensive unit tests**
  - Test each module independently
  - Edge cases
  - Error conditions
  - Target: 60%+ code coverage
  - Files: `wget-faster-lib/tests/`

#### 6.2 Integration Tests
- [ ] **Create wget compatibility tests**
  - Test against real servers
  - Compare output with wget
  - Test common use cases
  - Files: `wget-faster-cli/tests/`

#### 6.3 wget Test Suite
- [ ] **Set up wget-faster-test repository**
  - Create separate GPL-3.0 repo
  - Add wget as submodule
  - Configure test runner
  - Document setup

- [ ] **Run wget test suite**
  - Execute core HTTP tests
  - Document pass/fail
  - Analyze failures
  - Fix critical failures
  - **Goal**: 60%+ pass rate

#### 6.4 Manual Testing
- [ ] **Test real-world scenarios**
  - Download from common sites (GitHub, CDNs)
  - Large file downloads
  - Interrupted downloads + resume
  - Authentication scenarios
  - Cookie scenarios

### Phase 7: CLI Polish & Documentation (Week 4)

#### 7.1 CLI Improvements
- [ ] **Implement missing CLI options**
  - Review all parsed options in `args.rs`
  - Wire up any missing options
  - Remove unimplemented options
  - Files: `wget-faster-cli/src/args.rs`, `main.rs`

- [ ] **Implement quiet mode properly**
  - `-q, --quiet` suppresses all output
  - Errors still shown
  - Files: `wget-faster-cli/src/output.rs`

- [ ] **Implement verbose mode properly**
  - `-v, --verbose` shows detailed info
  - Headers, redirects, etc.
  - Files: `wget-faster-cli/src/output.rs`

#### 7.2 Error Messages
- [ ] **Improve error messages**
  - Match wget error format
  - Clear, actionable messages
  - Proper exit codes
  - Files: `wget-faster-cli/src/main.rs`

#### 7.3 Documentation
- [ ] **Update README.md**
  - v0.0.1 feature list
  - Installation instructions
  - Basic usage examples
  - Known limitations

- [ ] **Update CLAUDE.md**
  - Reflect v0.0.1 implementation
  - Document any architecture changes
  - Add troubleshooting guide

- [ ] **Create CHANGELOG.md**
  - Document v0.0.1 features
  - Known issues
  - Comparison with wget

---

## v0.0.1 MVP - Feature Checklist Summary

### Must Have (Blockers)
- [ ] Basic HTTP/HTTPS downloads working
- [ ] Resume (`-c`) working
- [ ] Output to file (`-O`) working
- [ ] Progress bar with speed/ETA
- [ ] Retry with backoff
- [ ] Authentication (Basic/Digest)
- [ ] Cookie load/save
- [ ] wget-style output format
- [ ] Quiet/verbose modes
- [ ] Pass 60%+ of wget core tests

### Should Have (Important)
- [ ] Multiple URL downloads
- [ ] Input file (`-i`)
- [ ] POST requests (`--post-data`, `--post-file`)
- [ ] Timestamping (`-N`)
- [ ] Server response (`-S`)
- [ ] Speed limiting (`--limit-rate`)

### Nice to Have (Optional)
- [ ] Spider mode (`--spider`)
- [ ] No clobber (`-nc`)
- [ ] Directory prefix (`-P`)
- [ ] Parallel downloads (disabled by default)

---

## Post-v0.0.1 (Future Patch Versions)

### v0.0.2 - Performance
- [ ] Enable parallel downloads by default
- [ ] Adaptive chunk sizing
- [ ] HTTP/2 optimization
- [ ] Performance benchmarks vs GNU wget
- [ ] Memory usage optimization
- [ ] Zero-copy chunk assembly

### v0.0.3 - Advanced Features
- [ ] Recursive downloads (`-r`)
- [ ] Page requisites (`-p`)
- [ ] Link conversion (`-k`)
- [ ] Accept/reject patterns (`-A`, `-R`)
- [ ] Domain filtering (`-D`)
- [ ] Directory structure (`-x`, `--cut-dirs`)

### v0.0.4 - Extended Protocols
- [ ] FTP/FTPS support
- [ ] IPv6 preferences
- [ ] Proxy improvements (SOCKS, NTLM)
- [ ] .netrc support
- [ ] robots.txt compliance

### v0.0.5 - HTTP/3
- [ ] QUIC support with quinn/h3
- [ ] Alt-Svc detection
- [ ] HTTP/3 benchmarks
- [ ] Fallback to HTTP/2

### Future Minor Version - Production Ready
- [ ] Full wget compatibility (95%+)
- [ ] Man pages
- [ ] Shell completions (bash, zsh, fish)
- [ ] WARC format support
- [ ] .wgetrc configuration file
- [ ] Background execution (`-b`)
- [ ] Metalink support
- [ ] Package distribution (Homebrew, apt, AUR)
- [ ] Docker image

---

## Development Guidelines

### Testing Requirements
- Every new feature must have tests
- Unit tests for library code
- Integration tests for CLI
- Manual testing for UX features

### Documentation Requirements
- Update README.md for user-facing changes
- Update CLAUDE.md for implementation details
- Update SPEC.md for specifications
- rustdoc comments for all public APIs

### Code Quality
- `cargo clippy --all-targets` passes
- `cargo fmt` enforced
- No `unwrap()` in library code
- Proper error handling with `?`

### Compatibility
- Match wget output format exactly
- Use same default behaviors
- Compatible exit codes
- Compatible option names

---

## Known Issues & Limitations (v0.0.1)

### Out of Scope
- No recursive downloads (coming in v0.0.3)
- No FTP support (coming in v0.0.4)
- No HTTP/3 (coming in v0.0.5)
- No .wgetrc config (coming in future minor version)

### Differences from wget
- Parallel downloads optional (disabled by default for compatibility)
- Some advanced SSL options may differ
- Error messages may have slight wording differences
- Progress bar update frequency may differ

### Performance Notes
- v0.0.1 focuses on compatibility over speed
- Parallel downloads available but not default
- Performance optimization is v0.0.2 goal

---

## Completed (Current Implementation)

### Core Library
- [x] Async I/O with tokio
- [x] HTTP/HTTPS client (reqwest)
- [x] Basic parallel downloads
- [x] Progress tracking
- [x] Resume support (partial)
- [x] Retry logic (partial)
- [x] Authentication structures
- [x] Cookie structures
- [x] Error types

### CLI
- [x] Argument parsing (150+ options)
- [x] Basic CLI structure
- [x] Some output formatting

### Documentation
- [x] README.md
- [x] CLAUDE.md
- [x] SPEC.md (updated for v0.0.1)
- [x] TODO.md (this file)

---

## Quick Reference

### Priority Levels
- **Critical**: Must have for v0.0.1 release
- **Important**: Should have, but can defer if needed
- **Optional**: Nice to have, can defer to v0.1.0

### File Locations
- Library code: `wget-faster-lib/src/`
- CLI code: `wget-faster-cli/src/`
- Tests: `wget-faster-lib/tests/`, `wget-faster-cli/tests/`
- Docs: `README.md`, `CLAUDE.md`, `SPEC.md`, `TODO.md`

### Testing
- Unit tests: `cargo test --lib`
- Integration tests: `cargo test --test integration_tests`
- All tests: `cargo test --all`
- Manual test: `cargo run -- https://example.com`

### Development Workflow
1. Pick a task from this TODO
2. Create feature branch
3. Write tests first (TDD)
4. Implement feature
5. Run `cargo clippy` and `cargo fmt`
6. Update documentation
7. Mark task as complete
8. Create pull request

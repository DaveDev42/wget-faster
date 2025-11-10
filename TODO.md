# TODO - Pending Tasks and Future Improvements

## High Priority

### CLI Implementation

- [ ] **Wire up all parsed CLI options to library calls**
  - Currently 150+ options are parsed but not all are connected to the library
  - Connect each option to appropriate `DownloadConfig` field
  - File: `wget-faster-cli/src/main.rs`

- [ ] **Implement POST request handling**
  - `--post-data` option is parsed but not executed
  - `--post-file` option is parsed but not executed
  - Add POST support to library if needed
  - Files: `wget-faster-lib/src/downloader.rs`, `wget-faster-cli/src/main.rs`

- [ ] **Complete cookie file I/O**
  - Cookie structure exists in library
  - Need to implement Netscape cookie file format parser
  - `--load-cookies` and `--save-cookies` options
  - Files: `wget-faster-lib/src/client.rs`

- [ ] **Implement input file handling**
  - `-i, --input-file` option for reading URLs from a file
  - Support for HTML file parsing with `-F, --force-html`
  - File: `wget-faster-cli/src/main.rs`

### Testing

- [ ] **Set up wget-faster-test repository**
  - Create separate GPL-3.0 licensed repository
  - Add wget as git submodule
  - Create test runner scripts
  - Document in README.md
  - See: TEST_REPO_SETUP.md (to be moved)

- [ ] **Run wget test suite against wget-faster**
  - Execute ~100 Python tests from wget
  - Document test results
  - Analyze failures
  - Create compatibility matrix

- [ ] **Add comprehensive unit tests**
  - Test each module independently
  - Edge cases and error conditions
  - Target: 80%+ code coverage
  - Files: `wget-faster-lib/tests/`

### Documentation

- [ ] **Add rustdoc comments to all public APIs**
  - Document all public functions, structs, enums
  - Add usage examples in doc comments
  - Generate and review `cargo doc` output
  - Files: All `wget-faster-lib/src/*.rs`

- [ ] **Create usage examples**
  - Basic download examples
  - Advanced configuration examples
  - Progress tracking examples
  - Error handling examples
  - Consider: `examples/` directory

## Medium Priority

### Features

- [ ] **Implement recursive downloads**
  - HTML parsing with url extraction
  - Link conversion (`-k, --convert-links`)
  - Page requisites (`-p, --page-requisites`)
  - Accept/reject patterns (`-A`, `-R`)
  - Domain filtering (`-D`, `--exclude-domains`)
  - Max depth control (`-l, --level`)
  - New module: `wget-faster-lib/src/recursive.rs`

- [ ] **Add FTP support**
  - FTP client implementation
  - FTPS (FTP over SSL/TLS)
  - FTP authentication
  - Passive/active mode
  - Directory listing
  - Consider using existing crates (e.g., `suppaftp`)

- [ ] **Implement timestamping**
  - `-N, --timestamping` option
  - Compare local and remote file timestamps
  - Download only if remote is newer
  - Set local file timestamp to match remote

- [ ] **Add quota support**
  - `-Q, --quota` option
  - Track cumulative download size
  - Stop when quota exceeded
  - Per-session quota tracking

- [ ] **Implement wait/waitretry options**
  - `-w, --wait` between downloads
  - `--waitretry` increasing wait on retry
  - Random wait with `--random-wait`

### Performance

- [ ] **Create performance benchmarks**
  - Benchmark against GNU wget
  - Test with various file sizes
  - Test with different network conditions
  - Parallel vs sequential comparison
  - Create: `benches/` directory

- [ ] **Optimize parallel downloads**
  - Dynamic chunk size adjustment
  - Adaptive connection count based on speed
  - Better handling of slow chunks
  - Connection pooling

- [ ] **Optimize memory usage**
  - Profile memory usage
  - Reduce allocations in hot paths
  - Optimize buffer sizes
  - Streaming optimizations

- [ ] **Add progress bar optimizations**
  - Reduce progress callback frequency
  - Batch progress updates
  - Minimize terminal I/O

## Low Priority

### Advanced Features

- [ ] **WARC support**
  - Web ARChive format
  - `--warc-file` option
  - WARC header generation
  - WARC record writing
  - Consider using existing crates

- [ ] **Metalink support**
  - Parse Metalink XML files
  - Multiple mirror downloads
  - Checksum verification
  - Priority and preference handling

- [ ] **Proxy authentication**
  - Proxy username/password
  - NTLM proxy authentication
  - HTTP/HTTPS proxy support
  - SOCKS proxy support

- [ ] **robots.txt compliance**
  - Fetch and parse robots.txt
  - Respect crawl-delay
  - Respect disallow rules
  - User-agent specific rules

- [ ] **IPv6 support**
  - Prefer IPv4 or IPv6
  - `--prefer-family` option
  - IPv6 address handling

- [ ] **Background execution**
  - `-b, --background` option
  - Daemon mode
  - Log output to file
  - PID file management

### Quality of Life

- [ ] **.netrc support**
  - Parse `~/.netrc` file
  - Automatic authentication
  - Machine-specific credentials

- [ ] **Configuration file support**
  - `.wgetrc` file parsing
  - User and system-wide configs
  - `-e, --execute` command support

- [ ] **Output templates**
  - Configurable output format
  - JSON output mode
  - Machine-readable output

- [ ] **Improved error messages**
  - More descriptive error messages
  - Suggestions for common errors
  - Colored output (optional)

- [ ] **Shell completion**
  - Bash completion
  - Zsh completion
  - Fish completion
  - Generate with clap

## Infrastructure

### CI/CD

- [ ] **Set up GitHub Actions**
  - Build on Linux, macOS, Windows
  - Run tests on all platforms
  - Clippy and rustfmt checks
  - Code coverage reporting

- [ ] **Automated releases**
  - Semantic versioning
  - Automated changelog generation
  - GitHub releases
  - Binary artifact uploads

- [ ] **Cross-compilation**
  - Linux (x86_64, ARM)
  - macOS (Intel, Apple Silicon)
  - Windows (x86_64)
  - Static binaries for Linux

### Distribution

- [ ] **Package for distributions**
  - Cargo crate publishing
  - Homebrew formula
  - Debian/Ubuntu packages
  - Arch Linux AUR
  - Windows installer

- [ ] **Docker image**
  - Official Docker image
  - Multi-arch support
  - Minimal Alpine-based image

## Research and Planning

- [ ] **Investigate streaming improvements**
  - HTTP/2 and HTTP/3 support
  - Connection multiplexing
  - Server push support

- [ ] **Research adaptive algorithms**
  - Adaptive chunk sizing
  - Adaptive connection count
  - Network condition detection
  - Congestion control

- [ ] **Explore compression options**
  - Zstandard support
  - Brotli optimization
  - On-the-fly decompression

## Completed

- [x] Core library with async support
- [x] Parallel downloads via Range requests
- [x] Progress tracking with callbacks
- [x] Resume support (`-c`)
- [x] HTTP/HTTPS client with reqwest
- [x] Authentication (Basic/Digest)
- [x] SSL/TLS configuration
- [x] Cookie support (in-memory)
- [x] Custom headers
- [x] Proxy support
- [x] Redirect following
- [x] Content compression (gzip, deflate, brotli)
- [x] Speed limiting
- [x] Configurable timeouts
- [x] Retry logic with exponential backoff
- [x] CLI argument parsing (150+ options)
- [x] wget-style output formatting
- [x] Error handling and types
- [x] Basic documentation (README, CLAUDE, SPEC, TODO)

## Notes

### Priorities Explanation

**High Priority**: Critical for basic wget compatibility and usability
**Medium Priority**: Important features that enhance functionality
**Low Priority**: Nice-to-have features and advanced use cases

### Contributing

When picking up a task:
1. Update the checkbox when starting
2. Create a branch for the feature
3. Write tests for new functionality
4. Update documentation
5. Submit a pull request

### Testing Strategy

For each new feature:
1. Add unit tests in `wget-faster-lib/tests/`
2. Add integration tests in `wget-faster-cli/tests/`
3. Test against wget test suite in separate repository
4. Manual testing with various scenarios

### Documentation Requirements

For each new feature:
1. Add rustdoc comments to public APIs
2. Update CLAUDE.md with implementation details
3. Update SPEC.md with technical specifications
4. Add usage examples to README.md
5. Update this TODO.md

## Version Planning

### v0.1.0 (Current)
- Core library functionality
- Basic CLI implementation
- Essential wget options

### v0.2.0 (Next)
- Complete CLI option coverage
- Cookie file I/O
- POST request support
- wget test suite integration
- Comprehensive unit tests

### v0.3.0
- Recursive downloads
- FTP support
- Timestamping
- Performance benchmarks

### v1.0.0
- Full wget compatibility
- Stable API
- Comprehensive documentation
- Production-ready quality

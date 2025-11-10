# Technical Specifications

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      wget-faster-cli                         │
│  ┌───────────┐  ┌───────────┐  ┌─────────────────────┐    │
│  │  args.rs  │─>│  main.rs  │─>│  output.rs (format) │    │
│  └───────────┘  └─────┬─────┘  └─────────────────────┘    │
└────────────────────────┼────────────────────────────────────┘
                         │ API calls
┌────────────────────────┼────────────────────────────────────┐
│                 wget-faster-lib                              │
│  ┌─────────────┴──────────┐                                 │
│  │    downloader.rs       │                                 │
│  │  (Main orchestrator)   │                                 │
│  └──┬─────────────────┬───┘                                 │
│     │                 │                                      │
│     v                 v                                      │
│  ┌──────────┐    ┌────────────┐                            │
│  │client.rs │    │parallel.rs │                            │
│  │(reqwest) │    │ (Range     │                            │
│  │          │    │  requests) │                            │
│  └────┬─────┘    └─────┬──────┘                            │
│       │                │                                     │
│       v                v                                     │
│  ┌────────────────────────────┐                            │
│  │      progress.rs           │                            │
│  │   (Speed/ETA tracking)     │                            │
│  └────────────────────────────┘                            │
│                                                              │
│  Supporting modules:                                         │
│  • config.rs  - Configuration types                         │
│  • output.rs  - Output modes (Memory/File)                  │
│  • error.rs   - Error types                                 │
└──────────────────────────────────────────────────────────────┘
```

### Module Responsibilities

#### wget-faster-lib

**downloader.rs** - Main async downloader engine
- Entry point for downloads
- Decides between sequential/parallel download
- Handles resume logic
- Coordinates progress tracking
- Retry logic with exponential backoff

**client.rs** - HTTP client wrapper
- Wraps reqwest client
- Configures timeouts, headers, auth, SSL
- Manages cookies
- Handles redirects
- Speed limiting

**parallel.rs** - Parallel download implementation
- Detects Range request support
- Splits file into chunks
- Downloads chunks concurrently
- Assembles chunks into final file
- Per-chunk retry logic

**progress.rs** - Progress tracking
- Real-time speed calculation (moving average)
- ETA estimation
- Byte formatting (KB, MB, GB)
- Progress percentage
- Thread-safe progress updates

**config.rs** - Configuration types
- `DownloadConfig` - Main configuration struct
- `AuthConfig` - Authentication configuration
- `AuthType` - Basic/Digest enum
- Default configurations

**output.rs** - Output modes
- `OutputMode` enum (Memory/File)
- Output mode selection logic

**error.rs** - Error types
- 13 error variants covering all failure modes
- Conversion from underlying errors (reqwest, io, etc.)

#### wget-faster-cli

**args.rs** - Argument parsing
- 150+ wget option definitions using clap
- Grouped by category (startup, download, HTTP, etc.)
- Help text matching wget's format
- Option validation

**main.rs** - CLI entry point
- Parse command-line arguments
- Build DownloadConfig from args
- Create Downloader instance
- Execute downloads
- Handle errors and output

**output.rs** - wget-style output formatting
- Connection messages
- HTTP response display
- Progress bars
- Completion messages
- Error messages
- Timestamp formatting

## Data Flow

### Sequential Download

```
URL → Downloader → HttpClient → GET Request → Stream → Output
                       ↓
                   Progress Tracker → Callback → User
```

### Parallel Download

```
URL → Downloader → Check Range Support
                       ↓
                   Calculate Chunks
                       ↓
        ┌──────────────┼──────────────┐
        v              v              v
    Chunk 1        Chunk 2    ...   Chunk N
        │              │              │
        v              v              v
    Download       Download       Download
     (Range)        (Range)        (Range)
        │              │              │
        └──────────────┼──────────────┘
                       v
                  Assemble File
                       ↓
                    Progress → Callback → User
```

### Resume Flow

```
Check Local File → Get File Size → HEAD Request
                                         ↓
                                    Compare Size
                                         ↓
                      ┌──────────────────┴──────────────┐
                      v                                  v
              Size Matches                      Size Different
                      ↓                                  ↓
              Resume Download                    Start Fresh
         (Range: bytes=localsize-)           (Delete + Download)
```

## API Specifications

### Core Library API

```rust
// Main entry point
pub struct Downloader {
    client: HttpClient,
    config: DownloadConfig,
}

impl Downloader {
    // Create new downloader
    pub fn new(config: DownloadConfig) -> Result<Self, DownloadError>;

    // Download to memory
    pub async fn download_to_memory(&self, url: &str)
        -> Result<Bytes, DownloadError>;

    // Download to file
    pub async fn download_to_file(&self, url: &str, path: PathBuf)
        -> Result<(), DownloadError>;

    // Download to file with progress callback
    pub async fn download_to_file_with_progress(
        &self,
        url: &str,
        path: PathBuf,
        progress_callback: Option<Arc<dyn Fn(ProgressInfo) + Send + Sync>>,
    ) -> Result<(), DownloadError>;
}

// Configuration
pub struct DownloadConfig {
    pub timeout: Duration,                    // Overall timeout
    pub connect_timeout: Duration,            // Connection timeout
    pub user_agent: Option<String>,           // User-Agent header
    pub headers: HashMap<String, String>,     // Custom headers
    pub auth: Option<AuthConfig>,             // Authentication
    pub follow_redirects: bool,               // Follow HTTP redirects
    pub max_redirects: usize,                 // Max redirect count
    pub verify_ssl: bool,                     // SSL certificate verification
    pub cookies: Option<CookieJar>,           // Cookie jar
    pub speed_limit: Option<u64>,             // Bytes per second limit
    pub retry_attempts: usize,                // Max retry attempts
    pub retry_delay: Duration,                // Initial retry delay
    pub parallel_chunks: usize,               // Number of parallel connections
    pub parallel_threshold: u64,              // Min file size for parallel
}

// Progress information
pub struct ProgressInfo {
    pub downloaded: u64,                      // Bytes downloaded
    pub total: Option<u64>,                   // Total bytes (if known)
    pub speed: f64,                           // Bytes per second
    pub elapsed: Duration,                    // Time elapsed
}

impl ProgressInfo {
    pub fn percentage(&self) -> Option<f64>;  // 0-100
    pub fn eta(&self) -> Option<Duration>;    // Estimated time remaining
    pub fn format_downloaded(&self) -> String; // Human readable (e.g., "1.5 MB")
    pub fn format_total(&self) -> Option<String>;
    pub fn format_speed(&self) -> String;     // e.g., "1.2 MB/s"
}

// Error types
pub enum DownloadError {
    NetworkError(String),
    HttpError(u16, String),
    InvalidUrl(String),
    FileSystemError(String),
    AuthenticationError(String),
    SslError(String),
    TimeoutError(String),
    InvalidRange,
    TooManyRedirects,
    ContentLengthMismatch,
    ChunkDownloadFailed,
    RateLimitExceeded,
    Unknown(String),
}
```

### CLI Argument Groups

**Startup Options**:
- `-V, --version` - Display version
- `-h, --help` - Display help
- `-b, --background` - Go to background
- `-e COMMAND, --execute=COMMAND` - Execute `.wgetrc` command

**Logging Options**:
- `-o FILE, --output-file=FILE` - Log to FILE
- `-a FILE, --append-output=FILE` - Append to FILE
- `-d, --debug` - Print debug output
- `-q, --quiet` - Quiet mode
- `-v, --verbose` - Verbose mode
- `-nv, --no-verbose` - Non-verbose mode
- `--report-speed=TYPE` - Output speed as TYPE

**Download Options**:
- `-t NUMBER, --tries=NUMBER` - Set number of retries
- `-O FILE, --output-document=FILE` - Write to FILE
- `-nc, --no-clobber` - Skip existing files
- `-c, --continue` - Resume download
- `-N, --timestamping` - Only get newer files
- `-S, --server-response` - Print server response
- `--spider` - Don't download
- `-T SECONDS, --timeout=SECONDS` - Set all timeouts
- `-w SECONDS, --wait=SECONDS` - Wait between retrievals
- `--limit-rate=RATE` - Limit download rate
- `-Q QUOTA, --quota=QUOTA` - Set download quota

**Directory Options**:
- `-nd, --no-directories` - Don't create directories
- `-x, --force-directories` - Force directory creation
- `-nH, --no-host-directories` - Don't create host directories
- `-P PREFIX, --directory-prefix=PREFIX` - Save to PREFIX
- `--cut-dirs=NUMBER` - Ignore NUMBER directory levels

**HTTP Options**:
- `--http-user=USER` - Set HTTP username
- `--http-password=PASS` - Set HTTP password
- `--header=STRING` - Insert STRING in headers
- `-U AGENT, --user-agent=AGENT` - Set User-Agent
- `--referer=URL` - Include Referer header
- `--post-data=STRING` - Use POST with STRING data
- `--post-file=FILE` - Use POST with FILE contents
- `--load-cookies=FILE` - Load cookies from FILE
- `--save-cookies=FILE` - Save cookies to FILE

**HTTPS Options**:
- `--secure-protocol=PR` - Choose secure protocol
- `--no-check-certificate` - Don't check certificate
- `--certificate=FILE` - Client certificate
- `--certificate-type=TYPE` - Certificate type
- `--private-key=FILE` - Private key file
- `--private-key-type=TYPE` - Private key type
- `--ca-certificate=FILE` - CA certificate file
- `--ca-directory=DIR` - CA certificate directory

**Recursive Download Options**:
- `-r, --recursive` - Specify recursive download
- `-l DEPTH, --level=DEPTH` - Maximum recursion depth
- `-k, --convert-links` - Convert links for local viewing
- `-p, --page-requisites` - Get all page images, CSS, etc.
- `-A LIST, --accept=LIST` - Accepted extensions
- `-R LIST, --reject=LIST` - Rejected extensions
- `-D LIST, --domains=LIST` - Accepted domains
- `--exclude-domains=LIST` - Rejected domains

## Performance Specifications

### Parallel Downloads

**Trigger Conditions**:
- File size ≥ `parallel_threshold` (default: 10MB)
- Server supports Range requests (`Accept-Ranges: bytes`)
- No resume in progress

**Parameters**:
- Default parallel connections: 8
- Configurable: 1-64 connections
- Chunk size: `total_size / parallel_chunks` (rounded up)
- Buffer size per chunk: 8KB

**Performance Gains**:
- Typical speedup: 3-8x for large files
- Best on high-latency connections
- Diminishing returns >16 connections

### Memory Usage

**Sequential Download**:
- Buffer size: 8KB
- Peak memory: ~100KB

**Parallel Download (8 chunks)**:
- Buffer per chunk: 8KB
- Peak memory: ~1MB
- No full-file buffering

**Memory Mode**:
- Stores entire file in memory
- Use only for small files (<10MB recommended)

### Retry Logic

**Exponential Backoff**:
```
attempt 0: immediate
attempt 1: retry_delay (default 1s)
attempt 2: retry_delay * 2 (2s)
attempt 3: retry_delay * 4 (4s)
...
max: retry_delay * 2^attempt (capped at 60s)
```

**Retryable Errors**:
- Network errors (connection refused, timeout)
- HTTP 5xx errors (server errors)
- HTTP 429 (rate limit)
- Partial chunk failures in parallel downloads

**Non-Retryable Errors**:
- HTTP 4xx errors (except 429)
- Invalid URLs
- SSL certificate errors (if verify_ssl=true)
- File system errors

## Testing Specifications

### Unit Tests

**Library Tests** (`wget-faster-lib/tests/`):
- Download to memory
- Download to file
- Parallel download
- Progress tracking
- Resume functionality
- Error handling
- Configuration validation

**CLI Tests** (`wget-faster-cli/tests/`):
- Argument parsing
- Output formatting
- Error messages
- Multiple URL handling

### Integration Tests

**wget Test Suite Compatibility**:
- Separate repository: `wget-faster-test` (GPL-3.0)
- Test via `WGET_PATH` environment variable
- ~100 Python test files
- Binary-only testing (no code linking)

**Test Categories**:
- Basic HTTP/HTTPS downloads
- Authentication (Basic/Digest)
- SSL/TLS configurations
- Cookie handling
- Redirects
- Resume/continue
- Error conditions
- Output formats

### Performance Benchmarks

**Metrics**:
- Download speed (MB/s)
- Time to completion (seconds)
- Memory usage (MB)
- CPU usage (%)

**Test Files**:
- Small: 1MB, 10MB
- Medium: 100MB, 500MB
- Large: 1GB, 5GB

**Comparison**:
- wget-faster vs GNU wget
- Sequential vs parallel
- Various network conditions

## Security Specifications

### SSL/TLS

**Supported Protocols**:
- TLS 1.2
- TLS 1.3

**Certificate Verification**:
- Enabled by default
- Can be disabled with `--no-check-certificate`
- Custom CA certificates supported
- Client certificate authentication supported

### Authentication

**Supported Methods**:
- HTTP Basic
- HTTP Digest
- Custom Authorization headers

**Credential Handling**:
- Command-line arguments
- Environment variables (future)
- .netrc file (future)
- Never logged in debug output

## License and Compliance

**License**: BSD-3-Clause

**GPL Compatibility**: This project is completely independent of GNU wget. We do not include, copy, or modify any GPL-licensed code.

**Test Suite Separation**: wget test suite integration is maintained in a separate GPL-3.0 licensed repository to ensure complete license isolation.

**Binary Testing**: wget tests run against the compiled wget-faster binary, similar to how browsers are tested with W3C test suites. No code linking or derivation occurs.

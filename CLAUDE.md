# CLAUDE.md - AI/LLM Context for wget-faster

This document provides implementation details optimized for AI assistants (like Claude) to understand and work with the wget-faster codebase.

## Project Overview

**wget-faster** is a high-performance HTTP downloader written in Rust that **exceeds GNU wget's performance** through modern networking techniques. This is not just a wget replacement—it's a next-generation downloader built for speed.

### Performance Philosophy

wget-faster must be **demonstrably faster** than GNU wget through:

1. **HTTP/3 (QUIC) Support** - Lower latency, better congestion control, connection migration
2. **Intelligent Parallel Downloads** - Axel-style chunked parallel downloads with dynamic optimization
3. **Advanced Chunking** - Adaptive chunk sizing based on network conditions
4. **Efficient Assembly** - Zero-copy chunk merging when possible
5. **Connection Pooling** - Reuse connections across multiple downloads
6. **Async Everything** - Non-blocking I/O throughout the stack

### Project Structure

```
wget-faster/
├── Cargo.toml                    # Workspace manifest
├── deny.toml                     # Cargo deny configuration
├── .clippy.toml                  # Clippy lints configuration
├── .rustfmt.toml                 # Code formatting rules
├── justfile                      # Build commands (replaces Makefile)
├── CLAUDE.md                     # This file - AI assistant context
├── README.md                     # User-facing documentation
├── SPEC.md                       # Technical specifications
├── TODO.md                       # Development roadmap
├── wget-faster-lib/              # Core library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Public API surface
│       ├── downloader.rs         # Main orchestrator
│       ├── client.rs             # HTTP client wrapper
│       ├── parallel.rs           # Parallel download engine
│       ├── progress.rs           # Progress tracking
│       ├── config.rs             # Configuration types
│       ├── output.rs             # Output abstraction
│       └── error.rs              # Error types
└── wget-faster-cli/              # Command-line interface
    ├── Cargo.toml
    └── src/
        ├── main.rs               # CLI entry point
        ├── args.rs               # Argument parsing (150+ options)
        └── output.rs             # wget-style formatting

```

## Architecture

### Module Hierarchy and Responsibilities

```
┌─────────────────────────────────────────────────────────────┐
│                      wget-faster-cli                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  args.rs - Argument parsing with clap                │   │
│  │  • 150+ wget-compatible options                      │   │
│  │  • Performance options (--parallel-chunks, etc.)     │   │
│  └────────────────────┬─────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼─────────────────────────────────┐   │
│  │  main.rs - CLI orchestration                         │   │
│  │  • Parse args → DownloadConfig                       │   │
│  │  • Initialize Downloader                             │   │
│  │  • Handle progress display                           │   │
│  └────────────────────┬─────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼─────────────────────────────────┐   │
│  │  output.rs - wget-style output formatting            │   │
│  │  • Connection messages                               │   │
│  │  • Progress bars (indicatif)                         │   │
│  │  • Completion/error messages                         │   │
│  └──────────────────────────────────────────────────────┘   │
└───────────────────────┼──────────────────────────────────────┘
                        │ Library API
┌───────────────────────▼──────────────────────────────────────┐
│                     wget-faster-lib                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  downloader.rs - Main orchestration engine           │   │
│  │  • Download strategy selection                       │   │
│  │  • Resume logic                                      │   │
│  │  • Retry coordination                                │   │
│  │  • Progress aggregation                              │   │
│  └──────┬───────────────────────────────────────┬───────┘   │
│         │                                       │            │
│         ▼                                       ▼            │
│  ┌──────────────────┐              ┌──────────────────────┐ │
│  │  client.rs       │              │  parallel.rs         │ │
│  │  HTTP client     │              │  Parallel engine     │ │
│  │                  │              │                      │ │
│  │  • HTTP/1.1      │              │  • Range detection   │ │
│  │  • HTTP/2        │              │  • Chunk splitting   │ │
│  │  • HTTP/3 (TODO) │              │  • Parallel fetch    │ │
│  │  • Auth/cookies  │              │  • Chunk assembly    │ │
│  │  • Compression   │              │  • Per-chunk retry   │ │
│  │  • Speed limit   │              │  • Dynamic tuning    │ │
│  └────────┬─────────┘              └──────────┬───────────┘ │
│           │                                   │             │
│           └─────────────┬─────────────────────┘             │
│                         ▼                                    │
│           ┌─────────────────────────────┐                   │
│           │  progress.rs                │                   │
│           │  Progress tracking          │                   │
│           │  • Speed calculation        │                   │
│           │  • ETA estimation           │                   │
│           │  • Thread-safe updates      │                   │
│           │  • Moving average           │                   │
│           └─────────────────────────────┘                   │
│                                                              │
│  Supporting modules:                                         │
│  • config.rs  - Configuration structs                       │
│  • output.rs  - Output modes (Memory/File/AsyncWrite)       │
│  • error.rs   - Comprehensive error types                   │
└──────────────────────────────────────────────────────────────┘
```

### Core Library (wget-faster-lib)

#### lib.rs - Public API Surface
```rust
pub use error::{Error, Result};
pub use config::{DownloadConfig, RetryConfig, ProxyConfig, AuthConfig, AuthType};
pub use client::{HttpClient, ResourceMetadata};
pub use downloader::{Downloader, DownloadResult};
pub use progress::{ProgressInfo, ProgressCallback, format_bytes, format_bytes_per_sec, format_duration};
pub use output::{Output, DownloadedData};
```

**Key Types:**
- `Downloader` - Main entry point for downloads
- `DownloadConfig` - Comprehensive configuration
- `ProgressInfo` - Real-time progress data
- `Output` - Flexible output abstraction (memory, file, or custom)

#### downloader.rs - Main Orchestrator
**Responsibilities:**
- Download strategy selection (sequential vs parallel)
- Resume logic for partial downloads
- Retry coordination with exponential backoff
- Progress callback management
- Error handling and recovery

**Key Functions:**
```rust
pub async fn download(&self, url: &str, output: Output, progress: Option<ProgressCallback>) -> Result<DownloadResult>
pub async fn download_to_memory(&self, url: &str) -> Result<Bytes>
pub async fn download_to_file(&self, url: &str, path: PathBuf) -> Result<()>
pub async fn download_to_file_with_progress(&self, url: &str, path: PathBuf, progress: Option<ProgressCallback>) -> Result<()>
```

**Decision Logic:**
- Sequential download if: file < `parallel_threshold` OR server lacks Range support OR resume in progress
- Parallel download if: file ≥ `parallel_threshold` AND server supports Range requests

#### client.rs - HTTP Client Wrapper
**Wraps:** reqwest with rustls-tls backend

**Responsibilities:**
- HTTP protocol handling (HTTP/1.1, HTTP/2, HTTP/3 planned)
- Connection management and pooling
- Header management (User-Agent, custom headers, auth)
- Cookie jar management
- Redirect following with max redirect limit
- SSL/TLS configuration
- Compression handling (gzip, deflate, brotli)
- Speed limiting
- Timeout management (connect vs total)

**Key Features:**
- Configurable User-Agent
- Basic/Digest authentication
- Proxy support
- Custom CA certificates
- Client certificates
- HTTP/HTTPS/SOCKS proxy support

#### parallel.rs - Parallel Download Engine
**This is the performance core of wget-faster.**

**Responsibilities:**
- Detect Range request support via HEAD request
- Calculate optimal chunk splits
- Download chunks in parallel with tokio tasks
- Assemble chunks into final file
- Handle per-chunk failures independently
- Adaptive chunk sizing (future)
- Dynamic connection count tuning (future)

**Algorithm:**
```rust
1. Send HEAD request to get content-length and check Accept-Ranges
2. If Range not supported → fall back to sequential
3. Calculate chunk_size = content_length / parallel_chunks
4. Spawn tokio tasks for each chunk with Range header
5. Download chunks to temporary files
6. Assemble chunks in order to final output
7. Verify final size matches content-length
8. Clean up temporary files
```

**Performance Parameters:**
- `parallel_chunks` - Number of concurrent connections (default: 8, max: 64)
- `parallel_threshold` - Minimum file size for parallel mode (default: 10MB)
- Buffer size per chunk: 8KB (streaming, not in-memory)

**Future Optimizations (TODO):**
- Adaptive chunk sizing based on observed speed
- Slow chunk detection and re-splitting
- Connection count auto-tuning based on network conditions
- HTTP/3 support for better parallelism

#### progress.rs - Progress Tracking
**Responsibilities:**
- Track bytes downloaded across all chunks
- Calculate download speed with moving average
- Estimate ETA (time remaining)
- Format human-readable sizes (KB, MB, GB, etc.)
- Thread-safe progress updates via atomic operations

**ProgressInfo Structure:**
```rust
pub struct ProgressInfo {
    pub downloaded: u64,      // Bytes downloaded so far
    pub total: Option<u64>,   // Total bytes (if known)
    pub speed: f64,           // Bytes per second (moving average)
    pub elapsed: Duration,    // Time elapsed since start
}
```

**Helper Functions:**
- `percentage()` - 0-100% completion
- `eta()` - Estimated time remaining
- `format_downloaded()` - "1.5 MB"
- `format_speed()` - "2.3 MB/s"

#### config.rs - Configuration Types
**Main Configuration:**
```rust
pub struct DownloadConfig {
    // Network
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub user_agent: Option<String>,
    pub headers: HashMap<String, String>,

    // Authentication
    pub auth: Option<AuthConfig>,

    // Redirects
    pub follow_redirects: bool,
    pub max_redirects: usize,

    // SSL/TLS
    pub verify_ssl: bool,

    // Cookies
    pub cookies: Option<CookieJar>,

    // Performance
    pub speed_limit: Option<u64>,        // Bytes per second
    pub parallel_chunks: usize,          // Number of parallel connections
    pub parallel_threshold: u64,         // Min file size for parallel

    // Retry
    pub retry_attempts: usize,
    pub retry_delay: Duration,
}
```

**Default Values:**
- `timeout`: 300s
- `connect_timeout`: 30s
- `parallel_chunks`: 8
- `parallel_threshold`: 10MB
- `retry_attempts`: 3
- `retry_delay`: 1s
- `follow_redirects`: true
- `max_redirects`: 20
- `verify_ssl`: true

#### error.rs - Error Types
**Comprehensive error handling with thiserror:**
```rust
pub enum Error {
    NetworkError(String),              // Connection failures
    HttpError(u16, String),            // HTTP status errors
    InvalidUrl(String),                // URL parsing errors
    FileSystemError(String),           // File I/O errors
    AuthenticationError(String),       // Auth failures
    SslError(String),                  // TLS/SSL errors
    TimeoutError(String),              // Timeout errors
    InvalidRange,                      // Range request not supported
    TooManyRedirects,                  // Redirect loop
    ContentLengthMismatch,             // Size mismatch
    ChunkDownloadFailed,               // Parallel chunk failure
    RateLimitExceeded,                 // HTTP 429
    Unknown(String),                   // Catch-all
}
```

#### output.rs - Output Abstraction
**Flexible output modes:**
```rust
pub enum Output {
    Memory,                            // Return Bytes
    File(PathBuf),                     // Write to file
    AsyncWrite(Box<dyn AsyncWrite + Unpin + Send>), // Custom writer
}
```

### CLI (wget-faster-cli)

#### args.rs - Argument Parsing
**150+ wget-compatible options using clap.**

**Option Groups:**
- **Startup**: `-V`, `-h`, `-b`, `-e`
- **Logging**: `-o`, `-a`, `-d`, `-q`, `-v`, `-nv`
- **Download**: `-t`, `-O`, `-nc`, `-c`, `-N`, `-S`, `-T`, `-w`, `--limit-rate`
- **Directories**: `-nd`, `-x`, `-nH`, `-P`, `--cut-dirs`
- **HTTP**: `--http-user`, `--header`, `--post-data`, `--user-agent`, `--cookies`
- **HTTPS/TLS**: `--no-check-certificate`, `--certificate`, `--ca-certificate`
- **FTP**: `--ftp-user`, `--ftp-password` (planned)
- **Recursive**: `-r`, `-l`, `-k`, `-p`, `-A`, `-R`, `-D`
- **Performance**: `--parallel-chunks`, `--parallel-threshold` (wget-faster extensions)

**Clap Configuration:**
- Uses derive API for clean struct-based parsing
- Grouped options for help organization
- Value validation and type safety
- Help text matching wget's style

#### main.rs - CLI Entry Point
**Main Flow:**
```rust
1. Parse command-line arguments with clap
2. Build DownloadConfig from parsed args
3. Create Downloader instance
4. Set up progress callback for terminal display
5. Execute download(s)
6. Handle errors and display messages
7. Exit with appropriate status code
```

**Key Responsibilities:**
- Map CLI args to library config
- Handle multiple URLs
- Manage output files
- Display progress with indicatif
- Error reporting in wget style

#### output.rs - wget-style Output Formatting
**Mimics GNU wget's output format:**

```
--2024-01-15 10:30:00--  https://example.com/file.zip
Resolving example.com (example.com)... 93.184.216.34
Connecting to example.com (example.com)|93.184.216.34|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 1048576 (1.0M) [application/zip]
Saving to: 'file.zip'

file.zip         100%[===============>]   1.00M  1.50MB/s    in 0.7s

2024-01-15 10:30:01 (1.50 MB/s) - 'file.zip' saved [1048576/1048576]
```

**Uses indicatif for progress bars with:**
- Percentage display
- Downloaded/total size
- Speed in MB/s
- ETA
- Visual progress bar

## Performance Features (Current & Planned)

### ✅ Implemented Performance Features

#### 1. Parallel Downloads via HTTP Range Requests
**Status:** ✅ Fully implemented

**How it works:**
- Automatically enabled for files ≥ 10MB (configurable)
- Splits file into N chunks (default: 8)
- Downloads chunks in parallel using tokio tasks
- Each chunk uses HTTP Range header: `Range: bytes=start-end`
- Assembles chunks in order to final file

**Performance gain:** 3-8x faster for large files on high-bandwidth connections

**Configuration:**
```rust
config.parallel_chunks = 16;         // More connections
config.parallel_threshold = 5_000_000; // Lower threshold (5MB)
```

#### 2. Async I/O Throughout
**Status:** ✅ Fully implemented

**Benefits:**
- Non-blocking network I/O with tokio
- Efficient CPU utilization
- Can handle multiple concurrent downloads
- Low memory overhead

#### 3. Streaming Downloads
**Status:** ✅ Fully implemented

**Benefits:**
- Fixed buffer sizes (8KB per chunk)
- No full-file buffering
- Memory-efficient for large files
- Constant memory usage regardless of file size

#### 4. Content Compression
**Status:** ✅ Fully implemented

**Supported formats:**
- gzip (flate2)
- deflate (flate2)
- brotli

**Automatic:** reqwest handles decompression transparently

#### 5. Connection Reuse
**Status:** ⚠️ Partial (reqwest handles HTTP/1.1 keep-alive and HTTP/2 multiplexing)

**Current:** Single Downloader instance reuses connections automatically via reqwest's connection pool

**Future:** Explicit connection pool management for better control

#### 6. Intelligent Retry Logic
**Status:** ✅ Fully implemented

**Features:**
- Exponential backoff: 1s, 2s, 4s, 8s, ... (max 60s)
- Configurable retry attempts (default: 3)
- Per-chunk retry in parallel downloads
- Retries on network errors and 5xx responses
- No retry on 4xx errors (except 429)

### 📋 Planned Performance Features (HIGH PRIORITY)

#### 1. HTTP/3 (QUIC) Support
**Status:** ❌ Not yet implemented
**Priority:** HIGH

**Why HTTP/3:**
- Lower latency (0-RTT connection establishment)
- Better congestion control
- Connection migration (survives IP changes)
- No head-of-line blocking
- Built on UDP for better parallelism

**Implementation plan:**
- Use `quinn` or `quiche` crate for QUIC
- Add HTTP/3 support to reqwest or use `h3` crate directly
- Make HTTP/3 optional feature flag
- Auto-detect and upgrade when available

**Expected gain:** 20-40% faster on high-latency connections

#### 2. Adaptive Chunk Sizing
**Status:** ❌ Not yet implemented
**Priority:** HIGH

**Current:** Fixed chunk size = `total_size / parallel_chunks`

**Planned:**
- Monitor per-chunk download speeds
- Detect slow chunks (outliers)
- Dynamically adjust chunk sizes
- Re-split slow chunks into smaller pieces
- Increase chunk size for fast connections

**Algorithm:**
```rust
1. Start with equal chunks
2. Monitor speed of each chunk
3. If chunk speed < 50% of average:
   - Pause slow chunk
   - Split remaining bytes into 2 smaller chunks
   - Resume with more parallelism
4. If all chunks fast and available connections < max:
   - Increase chunk count for next download
```

**Expected gain:** 15-30% faster by avoiding slow chunk bottlenecks

#### 3. Dynamic Connection Count Tuning
**Status:** ❌ Not yet implemented
**Priority:** HIGH

**Current:** Fixed `parallel_chunks = 8`

**Planned:**
- Start with conservative count (4)
- Measure aggregate throughput
- Incrementally increase connections
- Stop when throughput plateaus or decreases
- Save optimal count for domain/network
- Respect server `Connection` headers

**Algorithm:**
```rust
1. Start with 4 chunks
2. After 2 seconds, measure speed S1
3. Add 2 more chunks
4. After 2 more seconds, measure speed S2
5. If S2 > S1 * 1.1:
   - Continue increasing
6. Else:
   - Stop at current count
   - Use this count for future downloads
```

**Expected gain:** 10-20% faster by finding optimal parallelism per server

#### 4. Connection Pooling Across Downloads
**Status:** ⚠️ Partial (via reqwest)
**Priority:** MEDIUM

**Current:** reqwest maintains internal connection pool

**Planned:**
- Explicit pool management
- Pre-warm connections for known domains
- Connection keep-alive tuning
- DNS caching
- TLS session resumption

**Expected gain:** 5-10% faster for multiple small files from same domain

#### 5. Zero-Copy Chunk Assembly
**Status:** ❌ Not yet implemented
**Priority:** MEDIUM

**Current:** Chunks copied to final file via I/O

**Planned:**
- Use `io_uring` on Linux for zero-copy file merging
- Use `mmap` for in-place assembly on other platforms
- Avoid unnecessary buffer copies

**Expected gain:** 5-10% faster for very large files

#### 6. Predictive Prefetching
**Status:** ❌ Not yet implemented
**Priority:** LOW

**Planned for recursive downloads:**
- Parse HTML while downloading
- Identify linked resources
- Pre-connect to linked domains
- Prefetch critical resources
- Similar to browser behavior

**Expected gain:** 30-50% faster for recursive downloads

#### 7. Compression Dictionary Pre-loading
**Status:** ❌ Not yet implemented
**Priority:** LOW

**Planned:**
- Support shared compression dictionaries
- Brotli with custom dictionaries
- Zstandard dictionary support

**Expected gain:** 5-15% faster for compressed content with dictionaries

## API Examples

### Basic Usage

```rust
use wget_faster_lib::{Downloader, DownloadConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Simple download to memory
    let downloader = Downloader::new(DownloadConfig::default())?;
    let bytes = downloader.download_to_memory("https://example.com/file.txt").await?;
    println!("Downloaded {} bytes", bytes.len());
    Ok(())
}
```

### Download with Progress

```rust
use wget_faster_lib::{Downloader, DownloadConfig, ProgressInfo};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let downloader = Downloader::new(DownloadConfig::default())?;

    let progress = Arc::new(|info: ProgressInfo| {
        if let Some(pct) = info.percentage() {
            println!(
                "{:.1}% - {} / {} - {} - ETA: {}",
                pct,
                info.format_downloaded(),
                info.format_total().unwrap_or_else(|| "?".to_string()),
                info.format_speed(),
                info.eta()
                    .map(|d| format!("{:.0}s", d.as_secs_f64()))
                    .unwrap_or_else(|| "?".to_string())
            );
        }
    });

    downloader
        .download_to_file_with_progress(
            "https://example.com/large-file.zip",
            "large-file.zip".into(),
            Some(progress),
        )
        .await?;

    Ok(())
}
```

### High-Performance Configuration

```rust
use wget_faster_lib::{Downloader, DownloadConfig};
use std::time::Duration;

let mut config = DownloadConfig::default();

// Maximize parallel connections
config.parallel_chunks = 32;

// Lower threshold for parallel downloads
config.parallel_threshold = 1024 * 1024; // 1 MB

// Increase timeouts for slower connections
config.timeout = Duration::from_secs(600);
config.connect_timeout = Duration::from_secs(60);

// Enable more retries
config.retry_attempts = 5;
config.retry_delay = Duration::from_secs(2);

let downloader = Downloader::new(config)?;
```

### Resume Partial Download

```rust
use wget_faster_lib::{Downloader, DownloadConfig, Output};
use std::path::PathBuf;

let downloader = Downloader::new(DownloadConfig::default())?;

// If file already exists, download will resume from current size
downloader
    .download_to_file(
        "https://example.com/large-file.iso",
        PathBuf::from("large-file.iso"),
    )
    .await?;
```

### Multiple Parallel Downloads

```rust
use wget_faster_lib::{Downloader, DownloadConfig};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let downloader = Arc::new(Downloader::new(DownloadConfig::default())?);

    let urls = vec![
        "https://example.com/file1.zip",
        "https://example.com/file2.zip",
        "https://example.com/file3.zip",
    ];

    let mut set = JoinSet::new();

    for url in urls {
        let downloader = Arc::clone(&downloader);
        let filename = url.split('/').last().unwrap().to_string();

        set.spawn(async move {
            downloader.download_to_file(url, filename.into()).await
        });
    }

    while let Some(result) = set.join_next().await {
        result??; // Handle join error and download error
    }

    Ok(())
}
```

## Common Development Tasks

### Adding a New wget Option

1. **Add to CLI args** (`wget-faster-cli/src/args.rs`):
   ```rust
   #[arg(long = "new-option", help = "Description")]
   pub new_option: Option<String>,
   ```

2. **Map to config** (`wget-faster-cli/src/main.rs`):
   ```rust
   if let Some(value) = args.new_option {
       config.some_field = value;
   }
   ```

3. **Add to DownloadConfig if needed** (`wget-faster-lib/src/config.rs`):
   ```rust
   pub struct DownloadConfig {
       // ...
       pub some_field: String,
   }
   ```

4. **Implement in library** (`wget-faster-lib/src/downloader.rs` or appropriate module)

5. **Add tests** (`wget-faster-lib/tests/`)

6. **Update documentation** (this file, README.md, SPEC.md)

### Implementing a Performance Optimization

1. **Benchmark current performance**
   - Create benchmark in `benches/` directory
   - Measure baseline with `cargo bench`
   - Document baseline numbers

2. **Implement optimization**
   - Make changes in library code
   - Add feature flag if optional
   - Maintain backward compatibility

3. **Measure improvement**
   - Re-run benchmarks
   - Compare against baseline
   - Test with various file sizes and network conditions

4. **Validate correctness**
   - Ensure all tests pass
   - Test edge cases
   - Test error conditions

5. **Document**
   - Update this file with performance characteristics
   - Add code comments explaining optimization
   - Update SPEC.md with technical details

### Debugging Parallel Downloads

**Check if parallel is enabled:**
```rust
// In downloader.rs, add logging:
tracing::debug!(
    "File size: {}, threshold: {}, using parallel: {}",
    content_length,
    config.parallel_threshold,
    content_length >= config.parallel_threshold
);
```

**Monitor per-chunk progress:**
```rust
// In parallel.rs, add logging:
tracing::debug!(
    "Chunk {}/{}: {} bytes downloaded",
    chunk_num,
    total_chunks,
    chunk_bytes
);
```

**Verify Range support:**
```rust
// Check response headers:
let supports_range = response
    .headers()
    .get("accept-ranges")
    .and_then(|v| v.to_str().ok())
    .map(|v| v == "bytes")
    .unwrap_or(false);
```

**Common issues:**
- Server doesn't support Range requests → Check `Accept-Ranges` header
- File too small → Check `parallel_threshold` setting
- Chunk assembly fails → Check disk space and permissions
- Slow performance → Check network latency and chunk count

### Adding HTTP/3 Support (Future Task)

1. **Add dependency** (`wget-faster-lib/Cargo.toml`):
   ```toml
   quinn = { version = "0.10", optional = true }
   h3 = { version = "0.0.3", optional = true }

   [features]
   http3 = ["quinn", "h3"]
   ```

2. **Create HTTP/3 client** (`wget-faster-lib/src/client_h3.rs`):
   - Implement QUIC connection
   - Implement HTTP/3 request/response
   - Handle 0-RTT connections
   - Handle connection migration

3. **Integrate with downloader** (`wget-faster-lib/src/downloader.rs`):
   - Detect HTTP/3 support (Alt-Svc header)
   - Fall back to HTTP/2 or HTTP/1.1 if unavailable
   - Use HTTP/3 for parallel chunks if supported

4. **Add configuration** (`wget-faster-lib/src/config.rs`):
   ```rust
   pub struct DownloadConfig {
       // ...
       pub prefer_http3: bool,
       pub http3_max_idle_timeout: Duration,
   }
   ```

5. **Test and benchmark**
   - Compare HTTP/3 vs HTTP/2 performance
   - Test 0-RTT connections
   - Test connection migration

## Testing Strategy

### Unit Tests

**Location:** `wget-faster-lib/tests/`

**Coverage:**
- Download to memory
- Download to file
- Parallel downloads
- Progress tracking
- Resume functionality
- Error handling
- Configuration validation
- Chunk splitting logic

**Run tests:**
```bash
cargo test --lib
```

### Integration Tests

**Location:** `wget-faster-cli/tests/`

**Coverage:**
- Argument parsing
- Multiple URL handling
- Output formatting
- Error messages

**Run tests:**
```bash
cargo test --bin wget-faster
```

### wget Compatibility Tests

**Location:** Separate repository `wget-faster-test` (GPL-3.0)

**Why separate:** License isolation (BSD vs GPL)

**How it works:**
1. Build wget-faster binary
2. Set `WGET_PATH=/path/to/wget-faster`
3. Run wget's Python test suite
4. Binary-only testing (no code linking)

**Setup:**
```bash
cd ../wget-faster-test
git submodule init
git submodule update  # Fetches wget as submodule
./run-tests.sh
```

### Performance Benchmarks

**Location:** `benches/` (to be created)

**Benchmark scenarios:**
- Small files (1MB, 10MB)
- Medium files (100MB, 500MB)
- Large files (1GB, 5GB)
- Sequential vs parallel comparison
- Various chunk counts (2, 4, 8, 16, 32)
- Different network conditions (simulated latency/bandwidth)

**Run benchmarks:**
```bash
cargo bench
```

**Compare with wget:**
```bash
./scripts/benchmark-vs-wget.sh
```

## Code Style and Conventions

### Rust Style

- **Edition:** 2021
- **Formatting:** rustfmt with `.rustfmt.toml`
- **Linting:** clippy with `.clippy.toml` (pedantic mode)
- **Async:** Use `async fn` for all I/O
- **Errors:** Use `Result<T, Error>` with thiserror
- **Documentation:** rustdoc comments for all public APIs

### Naming Conventions

- Functions: `snake_case`
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`
- Be descriptive: `download_with_retry` not `dl_retry`

### Error Handling

```rust
// Good: Propagate errors with ?
pub async fn download(&self, url: &str) -> Result<Bytes> {
    let response = self.client.get(url).await?;
    Ok(response.bytes().await?)
}

// Avoid: unwrap/expect in library code
// Only use in tests or CLI with clear error messages
```

### Async Patterns

```rust
// Good: Use async fn
pub async fn download(&self, url: &str) -> Result<Bytes> {
    // ...
}

// Good: Use tokio::spawn for parallelism
let handles: Vec<_> = chunks
    .iter()
    .map(|chunk| tokio::spawn(download_chunk(chunk)))
    .collect();

// Good: Use select! for timeouts
tokio::select! {
    result = download_task => result?,
    _ = sleep(timeout) => return Err(Error::TimeoutError),
}
```

### Documentation

```rust
/// Downloads a file from the given URL to memory.
///
/// This method downloads the entire file into memory and returns it as `Bytes`.
/// For large files, consider using `download_to_file` instead to avoid high
/// memory usage.
///
/// # Arguments
///
/// * `url` - The URL to download from (HTTP or HTTPS)
///
/// # Returns
///
/// Returns `Ok(Bytes)` with the file contents on success, or an error if the
/// download fails.
///
/// # Errors
///
/// Returns an error if:
/// - The URL is invalid
/// - The network connection fails
/// - The HTTP response is an error status
///
/// # Example
///
/// ```no_run
/// use wget_faster_lib::{Downloader, DownloadConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let downloader = Downloader::new(DownloadConfig::default())?;
///     let bytes = downloader.download_to_memory("https://example.com/file.txt").await?;
///     println!("Downloaded {} bytes", bytes.len());
///     Ok(())
/// }
/// ```
pub async fn download_to_memory(&self, url: &str) -> Result<Bytes> {
    // Implementation
}
```

## Development Roadmap

### Current Phase: v0.1.0 - Foundation
- ✅ Core library with async support
- ✅ Basic parallel downloads
- ✅ Progress tracking
- ✅ CLI with 150+ options
- ⚠️ In progress: Full CLI option wiring

### Next Phase: v0.2.0 - Performance
- ❌ HTTP/3 (QUIC) support
- ❌ Adaptive chunk sizing
- ❌ Dynamic connection count tuning
- ❌ Comprehensive benchmarks
- ❌ wget compatibility tests

### Future Phase: v0.3.0 - Features
- ❌ Recursive downloads
- ❌ FTP/FTPS support
- ❌ Cookie file I/O (Netscape format)
- ❌ POST request support

### Long-term: v1.0.0 - Production Ready
- ❌ 100% wget compatibility
- ❌ Stable API
- ❌ Full documentation
- ❌ Production-quality error messages
- ❌ Comprehensive test coverage (>80%)

## Performance Targets

### Speed Targets (vs GNU wget)

- **Small files (<10MB):** 1-2x faster (less network-bound)
- **Medium files (10MB-100MB):** 3-5x faster (parallel downloads)
- **Large files (>100MB):** 5-10x faster (parallel + HTTP/3)
- **Recursive downloads:** 2-3x faster (parallelism + predictive prefetching)

### Efficiency Targets

- **Memory usage:** ≤ 10MB per download (excluding file content)
- **CPU usage:** < 5% per download on modern CPU
- **Parallel overhead:** < 10% additional memory per chunk
- **Connection overhead:** < 100ms per connection with HTTP/3

### Benchmark Methodology

**Test Environment:**
- Cloud VM with 1Gbps connection
- Multiple geographic locations
- Various server types (Nginx, Apache, S3, CDN)

**Test Files:**
- Hosted on high-performance CDN
- Various sizes: 1MB, 10MB, 100MB, 1GB
- Compressed and uncompressed variants

**Metrics:**
- Total download time
- Average speed (MB/s)
- Peak speed (MB/s)
- Memory usage (peak RSS)
- CPU usage (average %)

**Comparison:**
- wget-faster vs GNU wget
- wget-faster vs aria2
- wget-faster vs curl
- Sequential vs parallel modes

## License and Legal

**License:** BSD-3-Clause

**Philosophy:** Complete independence from GNU wget

**What this means:**
- No GPL code in this repository
- No copying of wget source code
- No derivation from wget implementation
- Clean-room implementation based on:
  - HTTP RFCs
  - wget documentation (facts, not code)
  - Independent algorithm design

**Testing Separation:**
- wget test suite in separate GPL-3.0 repository
- Binary-only testing (like browser test suites)
- No code linking or derivation
- Similar to: browsers + W3C tests, JVM + TCK, etc.

## Getting Help

### For AI Assistants

This document is designed for AI assistants working on wget-faster. Key sections:
- **Architecture** - Understand module structure
- **API Examples** - Common usage patterns
- **Common Development Tasks** - Step-by-step guides
- **Code Style and Conventions** - Follow project standards

### For Humans

- **README.md** - User documentation
- **SPEC.md** - Detailed technical specifications
- **TODO.md** - Development roadmap and tasks
- **Issues** - https://github.com/wget-faster/wget-faster/issues

---

**Remember:** wget-faster is not just a wget clone—it's a next-generation downloader built for maximum performance through modern networking techniques. Every feature should be implemented with performance in mind.

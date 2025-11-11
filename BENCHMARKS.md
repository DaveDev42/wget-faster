# wget-faster Benchmarks

This document describes the benchmarking framework for wget-faster.

## Running Benchmarks

To run all benchmarks:

```bash
cd wget-faster-lib
cargo bench
```

To run specific benchmarks:

```bash
# Sequential downloads only
cargo bench sequential_downloads

# Parallel downloads only
cargo bench parallel_downloads

# Progress tracking overhead
cargo bench progress_tracking

# Chunk size comparison
cargo bench chunk_sizes
```

## Benchmark Categories

### 1. Sequential Downloads

Tests download performance for files of different sizes without parallel chunking:
- 1KB
- 10KB
- 100KB
- 1MB

**Purpose**: Measure baseline download performance and overhead.

### 2. Parallel Downloads

Tests parallel download performance for larger files (10MB+):
- 10MB
- 20MB

**Purpose**: Measure performance gains from parallel chunked downloads.

### 3. Progress Tracking Overhead

Compares download performance with and without progress callbacks:
- Without progress callback
- With progress callback (1MB file)

**Purpose**: Measure the overhead of progress tracking.

### 4. Chunk Size Configuration

Tests different parallel chunk configurations:
- 4 chunks
- 8 chunks (default)
- 16 chunks
- 32 chunks

**Purpose**: Find optimal chunk count for different scenarios.

## Interpreting Results

Criterion produces detailed statistics including:
- **time**: Average time per iteration
- **thrpt**: Throughput (operations per second)
- **change**: Performance change vs previous run

### Example Output

```
sequential_downloads/1KB
                        time:   [1.234 ms 1.245 ms 1.256 ms]
                        thrpt:  [810.23 elem/s 820.45 elem/s 830.12 elem/s]

parallel_downloads/10MB
                        time:   [45.67 ms 46.12 ms 46.58 ms]
                        thrpt:  [21.46 Melem/s 21.68 Melem/s 21.90 Melem/s]
                        change: [-5.2% -3.8% -2.4%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## Benchmark Infrastructure

### Technology Stack

- **criterion**: Industry-standard benchmarking framework
- **mockito**: HTTP mock server for controlled testing
- **tokio**: Async runtime

### Test Setup

Each benchmark:
1. Creates a local mock HTTP server
2. Configures server to return test data
3. Performs downloads with different configurations
4. Measures performance with statistical analysis

### Limitations

- Uses local mock server (no network latency)
- Tests in-memory downloads (no disk I/O overhead)
- Small file sizes for reasonable benchmark duration
- Does not test real-world network conditions

## Performance Goals

### v0.0.1 Goals

- Sequential downloads: < 2ms overhead for 1MB
- Parallel downloads: 2-4x faster than sequential for 10MB+
- Progress tracking: < 5% overhead
- Optimal chunk count: 8-16 for most scenarios

### Future Goals (v0.1.0+)

- HTTP/3 (QUIC) performance comparison
- Real network benchmark suite
- Comparison with GNU wget
- Adaptive chunking performance validation

## Real-World Benchmarking

For real-world performance testing against actual servers:

```bash
# Build release binary
cargo build --release

# Test with a real file
time ./target/release/wgetf https://example.com/largefile.zip

# Compare with GNU wget
time wget https://example.com/largefile.zip
```

## Continuous Benchmarking

Benchmarks are run automatically on:
- Every release (pre-release validation)
- Performance-critical PRs
- Monthly regression testing

Results are tracked to detect performance regressions.

## Contributing Benchmarks

When adding new features, consider adding benchmarks:

1. Add benchmark function to `benches/download_benchmarks.rs`
2. Use realistic test scenarios
3. Document what the benchmark measures
4. Set appropriate sample sizes

Example:

```rust
fn bench_new_feature(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("new_feature", |b| {
        b.to_async(&rt).iter(|| async {
            // Your benchmark code here
        });
    });
}
```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [wget-faster Performance Goals](TODO.md#performance-targets)

---

**Last Updated**: 2025-11-11
**Next Review**: After v0.1.0 release

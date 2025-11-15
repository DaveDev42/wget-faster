# CI/CD Guide for wget-faster

This document describes the comprehensive CI/CD setup for the wget-faster project, including continuous integration, code coverage, benchmarking, and release automation.

## Table of Contents

1. [Overview](#overview)
2. [GitHub Actions Workflows](#github-actions-workflows)
3. [Code Coverage](#code-coverage)
4. [Benchmarking](#benchmarking)
5. [Security Auditing](#security-auditing)
6. [Local Development](#local-development)
7. [Configuration](#configuration)
8. [Troubleshooting](#troubleshooting)

## Overview

The wget-faster project uses GitHub Actions for all CI/CD automation. The pipeline includes:

- **Continuous Integration**: Automated testing, linting, and building on every push/PR
- **Code Coverage**: Comprehensive coverage reports using llvm-cov and tarpaulin
- **Benchmarking**: Performance tracking with Criterion and GNU wget comparisons
- **Security Auditing**: Dependency vulnerability scanning with cargo-audit
- **Release Automation**: Multi-platform binary builds and crates.io publishing

## GitHub Actions Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)

Runs on every push and pull request to the `main` branch.

**Jobs:**

#### Test Job
- Runs the full test suite
- Uses cargo caching for faster builds
- Command: `cargo test --all-features --verbose`

#### Lint Job
- Checks code formatting with rustfmt
- Runs clippy with pedantic lints
- Fails on warnings
- Commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`

#### Coverage Job
- Generates code coverage reports using cargo-llvm-cov
- Uploads to Codecov for tracking
- Creates downloadable HTML reports
- Command: `cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info`

#### Benchmark Job
- **Only runs on main branch pushes** (not PRs to save resources)
- Runs Criterion benchmarks
- Tracks performance over time
- Alerts on 50%+ performance regressions
- Command: `cargo bench --all-features`

#### Build Job
- Builds release binaries
- Validates that the release build succeeds
- Command: `cargo build --release --verbose`

#### Security Audit Job
- Scans dependencies for known vulnerabilities
- Uses cargo-audit with RustSec advisory database
- Command: `cargo audit --deny warnings`

**Trigger:** Push to `main`, Pull Requests to `main`

**Caching Strategy:**
- Cargo registry: `~/.cargo/registry`
- Cargo index: `~/.cargo/git`
- Build artifacts: `target/`
- Cache key based on `Cargo.lock` hash

### 2. Coverage Workflow (`.github/workflows/coverage.yml`)

Dedicated workflow for comprehensive code coverage analysis.

**Jobs:**

#### Coverage Job (llvm-cov)
- Primary coverage tool using LLVM instrumentation
- Generates LCOV, HTML, and summary reports
- Uploads to Codecov with `unittests` flag
- Comments on PRs with coverage summary
- Creates downloadable artifacts

#### Tarpaulin Job
- Alternative coverage tool for comparison
- **Only runs on schedule (weekly) or main branch pushes**
- Provides secondary verification of coverage metrics
- Uploads to Codecov with `tarpaulin` flag

**Trigger:**
- Push to `main`
- Pull Requests to `main`
- Weekly schedule (Sundays at 00:00 UTC)

**Output Artifacts:**
- `coverage-html-report`: Interactive HTML coverage report
- `coverage-lcov`: LCOV format for CI integration
- `coverage-summary`: Text summary of coverage metrics
- `tarpaulin-html-report`: Alternative HTML report

### 3. Benchmark Workflow (`.github/workflows/benchmark.yml`)

Performance testing and tracking workflow.

**Jobs:**

#### Benchmark Job
- Runs Criterion benchmarks
- Stores historical data in `dev/bench` branch
- Compares PR performance against main branch
- Alerts on 50%+ performance regressions
- Can be triggered manually via workflow_dispatch

#### Criterion Report Job
- Generates detailed Criterion HTML reports
- Produces JSON output for analysis
- Creates downloadable report artifacts

#### Performance Comparison Job
- **Only runs on main branch pushes** (resource intensive)
- Compares wget-faster vs GNU wget
- Tests multiple file sizes (1MB, 10MB, 100MB)
- Measures elapsed time and memory usage
- Generates comparison markdown report

**Trigger:**
- Push to `main`
- Pull Requests to `main`
- Manual trigger via GitHub UI

**Test Files:**
- 1MB: Quick download test
- 10MB: Medium file test (parallel threshold)
- 100MB: Large file with parallel chunks

### 4. Release Workflow (`.github/workflows/release.yml`)

Automated release process triggered by merging release PRs.

See [RELEASE.md](RELEASE.md) for detailed release process documentation.

**Jobs:**

1. **Auto-tag**: Creates git tag from branch name
2. **Build**: Multi-platform binary builds (Windows, Linux, macOS)
3. **Release**: Creates GitHub release with binaries
4. **Publish-crates**: Publishes to crates.io (optional with label)

**Trigger:** Merged PRs from `release/*` branches

## Code Coverage

### Tools

We use two code coverage tools for comprehensive analysis:

#### 1. cargo-llvm-cov (Primary)
- Uses LLVM's source-based code coverage
- More accurate than instrumentation-based tools
- Faster and more reliable
- Integrated with Codecov

**Installation:**
```bash
cargo install cargo-llvm-cov
```

**Usage:**
```bash
# Generate LCOV report
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate HTML report
cargo llvm-cov --all-features --workspace --html

# View HTML report
open target/llvm-cov/html/index.html
```

#### 2. cargo-tarpaulin (Secondary)
- Instrumentation-based coverage tool
- Runs weekly for comparison
- Helps catch edge cases

**Installation:**
```bash
cargo install cargo-tarpaulin
```

**Usage:**
```bash
# Generate coverage report
cargo tarpaulin --all-features --workspace --timeout 600 \
  --out Xml --out Html --output-dir ./tarpaulin-report

# View HTML report
open tarpaulin-report/index.html
```

### Codecov Integration

Coverage reports are automatically uploaded to Codecov:
- **Flags**: `unittests` (llvm-cov), `tarpaulin` (tarpaulin)
- **Token**: Stored in GitHub Secrets as `CODECOV_TOKEN`
- **Reports**: Available at https://codecov.io/gh/YOUR_USERNAME/wget-faster

**Setup Codecov:**

1. Sign up at https://codecov.io with your GitHub account
2. Enable wget-faster repository
3. Copy the upload token
4. Add to GitHub Secrets:
   - Go to repository Settings → Secrets → Actions
   - Create secret `CODECOV_TOKEN`
   - Paste the token

### Coverage Badges

Add to your README.md:

```markdown
[![codecov](https://codecov.io/gh/YOUR_USERNAME/wget-faster/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/wget-faster)
```

### Coverage Goals

- **Current target**: 60%+ line coverage
- **v0.1.0 goal**: 70%+ line coverage
- **v1.0.0 goal**: 80%+ line coverage

Priority areas:
1. Core download logic (downloader.rs, parallel.rs)
2. Error handling paths
3. Edge cases (timeouts, retries, redirects)

## Benchmarking

### Tools

#### 1. Criterion (Primary)
- Statistical benchmarking framework
- Detects performance regressions
- Generates HTML reports with charts

**Location:** `benches/download_bench.rs`

**Usage:**
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench download

# Save baseline for comparison
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

#### 2. GitHub Action Benchmark
- Tracks performance over time
- Stores historical data in `dev/bench` branch
- Comments on PRs with performance comparisons
- Alert threshold: 150% (50% regression)

### Benchmark Categories

Current benchmarks (from `benches/download_bench.rs`):

1. **Download benchmarks** - Core download performance
   - Small files (< 10MB)
   - Large files (> 10MB)
   - Parallel downloads

Add more benchmarks as needed:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wget_faster_lib::{Downloader, DownloadConfig};

fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Your benchmark code
        })
    });
}

criterion_group!(benches, bench_my_feature);
criterion_main!(benches);
```

### Performance Comparison

The benchmark workflow includes a real-world comparison against GNU wget:

**Test Setup:**
- Local HTTP server (Python http.server)
- Files: 1MB, 10MB, 100MB
- Metrics: Elapsed time, memory usage

**View Results:**
- Download the `performance-comparison` artifact from GitHub Actions
- Review the markdown comparison report

**Manual Comparison:**
```bash
# Build wget-faster
cargo build --release

# Create test file
dd if=/dev/urandom of=test.bin bs=1M count=100

# Start HTTP server
python3 -m http.server 8888 &

# Benchmark wget-faster
time ./target/release/wget-faster http://localhost:8888/test.bin -O /tmp/wgetf_test.bin

# Benchmark GNU wget
time wget http://localhost:8888/test.bin -O /tmp/wget_test.bin

# Cleanup
kill %1  # Stop HTTP server
```

## Security Auditing

### cargo-audit

Automatically scans dependencies for security vulnerabilities using the RustSec Advisory Database.

**Installation:**
```bash
cargo install cargo-audit
```

**Usage:**
```bash
# Check for vulnerabilities
cargo audit

# Deny warnings (CI mode)
cargo audit --deny warnings

# Update advisory database
cargo audit fetch
```

**CI Integration:**
- Runs on every push and PR
- Fails build if vulnerabilities found
- Database auto-updated by GitHub Actions

### cargo-deny

For more comprehensive dependency checks, see `deny.toml`:
```bash
# Install
cargo install cargo-deny

# Check licenses, sources, advisories
cargo deny check
```

## Local Development

### Running CI Checks Locally

Before pushing, run these checks locally:

```bash
# 1. Format code
cargo fmt --all

# 2. Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests
cargo test --all-features --verbose

# 4. Generate coverage
cargo llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html

# 5. Run benchmarks
cargo bench

# 6. Security audit
cargo audit

# 7. Build release
cargo build --release
```

### Using just (Task Runner)

The project includes a `justfile` for common tasks:

```bash
# Install just
cargo install just

# Available commands
just --list

# Run all CI checks
just ci

# Run tests with coverage
just coverage

# Run benchmarks
just bench
```

Add these to `justfile`:
```makefile
# Run all CI checks locally
ci: fmt clippy test audit build

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
    cargo test --all-features --verbose

# Security audit
audit:
    cargo audit

# Build release
build:
    cargo build --release --verbose

# Generate code coverage
coverage:
    cargo llvm-cov --all-features --workspace --html
    @echo "Opening coverage report..."
    @open target/llvm-cov/html/index.html || xdg-open target/llvm-cov/html/index.html

# Run benchmarks
bench:
    cargo bench --all-features

# Clean build artifacts
clean:
    cargo clean
```

### Pre-commit Hooks

Install git hooks to run checks before committing:

```bash
# Create .git/hooks/pre-commit
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt --all -- --check

# Clippy
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Tests
echo "Running tests..."
cargo test --all-features

echo "✅ All checks passed!"
EOF

# Make executable
chmod +x .git/hooks/pre-commit
```

## Configuration

### GitHub Secrets

Required secrets for full CI/CD functionality:

| Secret | Purpose | How to Get |
|--------|---------|------------|
| `CODECOV_TOKEN` | Upload coverage to Codecov | https://codecov.io/gh/YOUR_REPO |
| `CARGO_REGISTRY_TOKEN` | Publish to crates.io | https://crates.io/me |

**Adding Secrets:**
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Enter name and value
4. Click "Add secret"

### Workflow Customization

#### Adjust Coverage Thresholds

Edit `.github/workflows/coverage.yml`:
```yaml
- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    fail_ci_if_error: true  # Change to true to fail on upload errors
    # Add coverage threshold checks
```

#### Adjust Benchmark Alert Threshold

Edit `.github/workflows/benchmark.yml`:
```yaml
- name: Store benchmark result
  uses: benchmark-action/github-action-benchmark@v1
  with:
    alert-threshold: '150%'  # Change to '120%' for stricter alerts
    fail-on-alert: false     # Change to true to fail CI on regression
```

#### Change Coverage Tools

To use only llvm-cov (remove tarpaulin):
1. Delete the `tarpaulin` job from `.github/workflows/coverage.yml`
2. Update local scripts to use only llvm-cov

#### Customize Benchmark Frequency

Edit `.github/workflows/benchmark.yml`:
```yaml
on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    # Add weekly scheduled run
    - cron: '0 0 * * 0'  # Every Sunday at midnight
```

## Troubleshooting

### Coverage Not Uploading to Codecov

**Symptoms:**
- Coverage job succeeds but no data in Codecov
- "Error uploading coverage reports" in logs

**Solutions:**
1. Verify `CODECOV_TOKEN` is set correctly
2. Check Codecov service status
3. Verify repository is enabled in Codecov
4. Try manual upload:
   ```bash
   curl -Os https://uploader.codecov.io/latest/linux/codecov
   chmod +x codecov
   ./codecov -t ${CODECOV_TOKEN}
   ```

### Benchmark Comparison Not Working

**Symptoms:**
- No performance comparison comment on PRs
- "No baseline found" error

**Solutions:**
1. Ensure baseline data exists in `dev/bench` branch
2. Run benchmark workflow on main branch first
3. Check GitHub token permissions (needs write access)

### Build Failures on Specific Platforms

**Symptoms:**
- Release workflow fails on specific target (e.g., aarch64-linux)

**Solutions:**
1. Check cross-compilation setup in workflow
2. Verify target is properly installed
3. Test locally with:
   ```bash
   rustup target add aarch64-unknown-linux-gnu
   cargo build --target aarch64-unknown-linux-gnu
   ```

### Cache Issues

**Symptoms:**
- Slow builds despite caching
- "Cache restored but still rebuilding" errors

**Solutions:**
1. Clear cache manually in GitHub Actions UI
2. Update cache keys to force refresh:
   ```yaml
   key: ${{ runner.os }}-cargo-v2-${{ hashFiles('**/Cargo.lock') }}
   ```
3. Check for workspace-level dependency changes

### Security Audit Failures

**Symptoms:**
- cargo-audit reports vulnerabilities
- CI fails on security-audit job

**Solutions:**
1. Review advisory details: `cargo audit`
2. Update dependencies: `cargo update`
3. Check for patches: https://rustsec.org/advisories/
4. If no fix available, consider:
   - Replacing dependency
   - Adding temporary exception (use with caution)
   - Opening issue with dependency maintainer

### Test Failures in CI but Not Locally

**Symptoms:**
- Tests pass locally but fail in GitHub Actions

**Possible Causes:**
1. **Timing issues**: Tests depend on specific timing
   - Solution: Use tokio test timeouts, avoid sleeps
2. **Environment differences**: Different OS, Rust version
   - Solution: Test with Docker locally
3. **Resource constraints**: CI has limited resources
   - Solution: Reduce parallelism in tests
4. **Network dependencies**: Tests require internet
   - Solution: Mock network calls, use fixtures

**Debug Steps:**
```bash
# Run tests in release mode (like CI)
cargo test --release

# Run tests single-threaded
cargo test -- --test-threads=1

# Enable logging
RUST_LOG=debug cargo test
```

## Best Practices

### 1. Keep CI Fast
- Use caching effectively
- Run expensive jobs (benchmarks, comparisons) only on main branch
- Parallelize independent jobs
- Current CI time: ~5-10 minutes

### 2. Monitor Coverage Trends
- Check Codecov after each PR
- Aim to increase or maintain coverage
- Write tests for new features before merging

### 3. Review Benchmark Results
- Compare performance on large PRs
- Investigate unexpected regressions
- Document performance improvements

### 4. Regular Maintenance
- Update GitHub Actions versions quarterly
- Update Rust toolchain regularly
- Review and update dependencies monthly
- Check RustSec advisories weekly

### 5. Documentation
- Document new CI jobs in this file
- Update troubleshooting section with solutions
- Keep local development commands up-to-date

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [Codecov Documentation](https://docs.codecov.com/)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [RustSec Advisory Database](https://rustsec.org/)

## Contributing

When adding new CI/CD features:

1. Test locally first
2. Document in this file
3. Update `justfile` if applicable
4. Add troubleshooting section if needed
5. Test the workflow in a draft PR
6. Request review before merging

---

**Last Updated:** 2025-11-15
**Maintained by:** wget-faster contributors

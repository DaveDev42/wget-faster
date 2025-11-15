# justfile for wget-faster
# https://github.com/casey/just

# List available commands
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
release:
    cargo build --release

# Run all tests
test:
    cargo test --workspace

# Run cargo check
check:
    cargo check --workspace

# Run all linters (fmt-check + clippy)
lint: fmt-check clippy

# Format code with rustfmt
fmt:
    cargo fmt --all

# Check code formatting without changes
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fix clippy warnings automatically
clippy-fix:
    cargo clippy --workspace --all-targets --all-features --fix

# Clean build artifacts
clean:
    cargo clean

# Generate and open documentation
doc:
    cargo doc --workspace --open --no-deps

# Run wget-faster CLI (use: just run -- <args>)
run *ARGS:
    cargo run --bin wget-faster -- {{ARGS}}

# Install wget-faster binary
install:
    cargo install --path wget-faster-cli

# Run all CI checks (lint + test)
ci: lint test
    @echo "✓ All CI checks passed"

# Run cargo-deny checks (requires: cargo install cargo-deny)
deny:
    cargo deny check

# Watch and run tests on file changes (requires: cargo install cargo-watch)
watch-test:
    cargo watch -x test

# Watch and run check on file changes
watch-check:
    cargo watch -x check

# Watch and run clippy on file changes
watch-clippy:
    cargo watch -x clippy

# Run benchmarks
bench:
    cargo bench --workspace

# Run benchmarks and save baseline
bench-baseline NAME:
    cargo bench --workspace -- --save-baseline {{NAME}}

# Compare benchmarks against baseline
bench-compare NAME:
    cargo bench --workspace -- --baseline {{NAME}}

# Generate code coverage with llvm-cov (requires: cargo install cargo-llvm-cov)
coverage:
    cargo llvm-cov --all-features --workspace --html
    @echo "Coverage report generated at target/llvm-cov/html/index.html"
    @echo "Opening in browser..."
    @open target/llvm-cov/html/index.html || xdg-open target/llvm-cov/html/index.html || echo "Please open target/llvm-cov/html/index.html manually"

# Generate code coverage with llvm-cov (LCOV format)
coverage-lcov:
    cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
    @echo "LCOV report generated at lcov.info"

# Generate code coverage summary
coverage-summary:
    cargo llvm-cov --all-features --workspace --summary-only

# Generate code coverage with tarpaulin (requires: cargo install cargo-tarpaulin)
coverage-tarpaulin:
    cargo tarpaulin --all-features --workspace --timeout 600 --out Html --out Xml --output-dir ./tarpaulin-report
    @echo "Tarpaulin report generated at tarpaulin-report/index.html"
    @open tarpaulin-report/index.html || xdg-open tarpaulin-report/index.html || echo "Please open tarpaulin-report/index.html manually"

# Update dependencies
update:
    cargo update

# Check for outdated dependencies (requires: cargo install cargo-outdated)
outdated:
    cargo outdated

# Audit dependencies for security vulnerabilities (requires: cargo install cargo-audit)
audit:
    cargo audit

# Update cargo-audit database
audit-update:
    cargo audit fetch

# Run all checks before committing
pre-commit: fmt lint test
    @echo "✓ Ready to commit"

# Run full CI suite locally (all checks including coverage and audit)
ci-full: fmt lint test coverage-summary audit
    @echo "✓ All CI checks passed (including coverage and audit)"

# Setup development environment (install required tools)
setup:
    @echo "Installing development tools..."
    cargo install cargo-llvm-cov
    cargo install cargo-tarpaulin
    cargo install cargo-audit
    cargo install cargo-watch
    cargo install cargo-outdated
    cargo install cargo-deny
    cargo install cargo-criterion
    @echo "✓ Development environment setup complete"

# Create a new release (bump version and create release branch)
release-create VERSION:
    @echo "Creating release branch for v{{VERSION}}..."
    git checkout main
    git pull origin main
    git checkout -b release/v{{VERSION}}
    @echo "Update version in Cargo.toml to {{VERSION}}"
    @echo "Then: git add Cargo.toml CHANGELOG.md && git commit -m 'Bump version to {{VERSION}}'"
    @echo "Finally: git push origin release/v{{VERSION}} and create a PR"

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

# Run benchmarks (when available)
bench:
    cargo bench --workspace

# Update dependencies
update:
    cargo update

# Check for outdated dependencies (requires: cargo install cargo-outdated)
outdated:
    cargo outdated

# Audit dependencies for security vulnerabilities (requires: cargo install cargo-audit)
audit:
    cargo audit

# Run all checks before committing
pre-commit: fmt lint test
    @echo "✓ Ready to commit"

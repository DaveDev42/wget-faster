.PHONY: help build test check lint fmt fmt-check clippy clean doc run install

help:
	@echo "wget-faster - Makefile commands"
	@echo ""
	@echo "Development:"
	@echo "  make build      - Build the project in debug mode"
	@echo "  make release    - Build the project in release mode"
	@echo "  make test       - Run all tests"
	@echo "  make check      - Run cargo check"
	@echo ""
	@echo "Code Quality:"
	@echo "  make lint       - Run all linters (fmt-check + clippy)"
	@echo "  make fmt        - Format code with rustfmt"
	@echo "  make fmt-check  - Check code formatting without changes"
	@echo "  make clippy     - Run clippy linter"
	@echo ""
	@echo "Documentation:"
	@echo "  make doc        - Generate and open documentation"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make run        - Run wget-faster CLI"
	@echo "  make install    - Install wget-faster binary"

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --workspace

check:
	cargo check --workspace

lint: fmt-check clippy

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

clippy-fix:
	cargo clippy --workspace --all-targets --all-features --fix

clean:
	cargo clean

doc:
	cargo doc --workspace --open --no-deps

run:
	cargo run --bin wget-faster -- $(ARGS)

install:
	cargo install --path wget-faster-cli

# CI targets
ci-test: lint test
	@echo "✓ All CI checks passed"

# Watch targets (requires cargo-watch: cargo install cargo-watch)
watch-test:
	cargo watch -x test

watch-check:
	cargo watch -x check

watch-clippy:
	cargo watch -x clippy

#!/usr/bin/env bash
# Development tools setup script for wget-faster
# This script installs all the necessary tools for local development

set -e

echo "🚀 Setting up wget-faster development environment..."
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install it first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✓ Rust is installed"
echo ""

# Function to install a cargo tool if not already installed
install_tool() {
    local tool=$1
    local package=${2:-$1}

    if command -v "$tool" &> /dev/null; then
        echo "✓ $tool is already installed"
    else
        echo "📦 Installing $package..."
        cargo install "$package"
    fi
}

# Core development tools
echo "Installing core development tools..."
install_tool "cargo-llvm-cov"
install_tool "cargo-tarpaulin"
install_tool "cargo-audit"
install_tool "cargo-watch"
install_tool "cargo-outdated"
install_tool "cargo-deny"

# Optional but recommended tools
echo ""
echo "Installing optional tools..."
install_tool "cargo-criterion"
install_tool "cargo-expand"
install_tool "cargo-tree"

# Install just (task runner)
if command -v just &> /dev/null; then
    echo "✓ just is already installed"
else
    echo "📦 Installing just..."
    cargo install just
fi

# Ensure Rust toolchain components are installed
echo ""
echo "Installing Rust toolchain components..."
rustup component add rustfmt 2>/dev/null || echo "✓ rustfmt already installed"
rustup component add clippy 2>/dev/null || echo "✓ clippy already installed"
rustup component add llvm-tools-preview 2>/dev/null || echo "✓ llvm-tools-preview already installed"

# Update cargo-audit database
echo ""
echo "📥 Updating cargo-audit database..."
cargo audit fetch || echo "⚠️  Failed to update audit database (not critical)"

# Setup git hooks (optional)
echo ""
read -p "Would you like to install git pre-commit hooks? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Installing pre-commit hooks..."
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
    chmod +x .git/hooks/pre-commit
    echo "✓ Pre-commit hooks installed"
else
    echo "Skipping pre-commit hooks"
fi

echo ""
echo "✅ Development environment setup complete!"
echo ""
echo "Available commands (using just):"
echo "  just --list          # List all available commands"
echo "  just ci              # Run CI checks locally"
echo "  just coverage        # Generate coverage report"
echo "  just bench           # Run benchmarks"
echo "  just pre-commit      # Run pre-commit checks"
echo ""
echo "Start developing with:"
echo "  cargo build          # Build debug"
echo "  cargo test           # Run tests"
echo "  just coverage        # Check coverage"
echo ""
echo "📖 For more information, see CI_CD.md"

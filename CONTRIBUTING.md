# Contributing to wget-faster

Thank you for your interest in contributing to wget-faster! This document provides guidelines and information for contributors.

## Table of Contents

- [Development Setup](#development-setup)
- [Code Style and Linting](#code-style-and-linting)
- [Pre-commit Hooks](#pre-commit-hooks)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)

## Development Setup

### Prerequisites

- Rust 1.91.1 or later (stable channel)
- Cargo (comes with Rust)

### Building the Project

```bash
# Clone the repository
git clone https://github.com/wget-faster/wget-faster.git
cd wget-faster

# Build all workspace members
cargo build --workspace --all-features

# Run tests
cargo test --workspace --all-features

# Run a specific binary
cargo run --bin wgetf -- --help
```

## Code Style and Linting

We use consistent code formatting and linting across the project to maintain code quality.

### rustfmt Configuration

Code formatting is enforced via `rustfmt.toml`:

- **Line width**: 100 characters maximum
- **Function parameters**: Tall layout (one parameter per line for long signatures)
- **Newline style**: Unix (LF)
- **Stable features only**: Configuration works with stable Rust

**Format your code before committing:**

```bash
cargo fmt --all
```

**Check formatting without modifying files:**

```bash
cargo fmt --all -- --check
```

### Clippy Configuration

Linting rules are defined in `clippy.toml` and `Cargo.toml`:

**Key thresholds:**
- Cognitive complexity: 20 (keep functions simple)
- Too many lines: 80 (split large functions)
- Too many arguments: 6 (use structs for many parameters)
- Type complexity: 200

**Workspace-level lints (Cargo.toml):**
- `pedantic`: Enabled with lower priority for fine-grained control
- `cargo`: Warns about Cargo.toml issues
- `perf`: Performance-related warnings
- `correctness`: Errors for likely bugs
- `unwrap_used`, `expect_used`, `panic`: Warnings to encourage error handling

**Run clippy:**

```bash
# Check for lints
cargo clippy --workspace --all-targets --all-features

# Auto-fix some issues
cargo clippy --workspace --all-targets --all-features --fix
```

### Allowed Patterns

Some patterns are explicitly allowed in specific contexts:

- **Doctests**: `unwrap()` is acceptable in documentation examples for clarity
- **Test code**: `unwrap()`, `expect()`, and `panic!()` are allowed in `#[cfg(test)]` modules
- **CLI output**: `println!()` and `eprintln!()` in `wget-faster-cli` binary code
- **Library logging**: Use `tracing` macros (`debug!`, `info!`, `warn!`, `error!`) instead of `println!`

## Pre-commit Hooks

The project includes an automated pre-commit hook that runs before each commit.

### What the Hook Checks

1. **Code formatting** (rustfmt)
   - Ensures all Rust files are properly formatted
   - Fails if code is not formatted

2. **Linting** (clippy)
   - Runs on library and binary code (excludes tests for speed)
   - Checks for common mistakes and style issues

3. **Unit tests**
   - Runs library and binary tests
   - Excludes doctests and integration tests for faster feedback

4. **Common issues**
   - Detects TODO/FIXME comments
   - Warns about `println!` in library code
   - Reports unwrap() usage count

### Running Pre-commit Checks Manually

```bash
# The hook runs automatically on git commit
git commit -m "Your message"

# To bypass the hook (not recommended)
git commit --no-verify -m "Your message"

# To run checks manually
cargo fmt --all -- --check
cargo clippy --workspace --lib --bins --all-features
cargo test --workspace --lib --bins
```

### Hook Output

The hook provides colored, user-friendly output:
- 🟢 Green checkmarks for passing checks
- 🔴 Red X for failures
- 🟡 Yellow warnings for non-critical issues

## Testing

### Running Tests

```bash
# Run all tests (unit + integration + doctests)
cargo test --workspace --all-features

# Run only library and binary tests (faster)
cargo test --workspace --lib --bins

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Test Organization

- **Unit tests**: In `#[cfg(test)]` modules within source files
- **Integration tests**: In `wget-faster-lib/tests/` directory
- **Doctests**: In documentation comments (/// or //!)
- **Benchmarks**: In `wget-faster-lib/benches/` directory

### Test Coverage

We aim for:
- **Core functionality**: >80% coverage
- **Error handling**: All error paths tested
- **Edge cases**: Boundary conditions covered

## Pull Request Process

1. **Fork and clone** the repository
2. **Create a branch** for your feature/fix
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow the code style guidelines
   - Add tests for new functionality
   - Update documentation as needed

4. **Run pre-commit checks**
   ```bash
   cargo fmt --all
   cargo clippy --workspace --all-targets --all-features
   cargo test --workspace --all-features
   ```

5. **Commit your changes**
   - Write clear, descriptive commit messages
   - Reference issue numbers if applicable
   - The pre-commit hook will run automatically

6. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Open a Pull Request**
   - Describe what your PR does
   - Link to related issues
   - Ensure all CI checks pass

### Commit Message Guidelines

Use conventional commit format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Example:**
```
feat(downloader): Add support for HTTP/3

Implement QUIC protocol support using quinn crate.
This improves performance for high-latency connections.

Closes #123
```

## Code Review Expectations

Pull requests will be reviewed for:

- **Correctness**: Does it work as intended?
- **Tests**: Are there adequate tests?
- **Style**: Does it follow project conventions?
- **Documentation**: Is it well-documented?
- **Performance**: Are there any performance concerns?
- **Compatibility**: Does it maintain backward compatibility?

## Getting Help

- **Issues**: Open an issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check CLAUDE.md for AI/LLM-specific context

## License

By contributing, you agree that your contributions will be licensed under the BSD-3-Clause License.

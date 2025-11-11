# Release Process

This document describes the automated release process for wget-faster.

## Overview

The release process is fully automated through GitHub Actions. When a release PR is merged, the workflow automatically:

1. ✅ Creates and pushes a git tag
2. ✅ Builds binaries for all platforms
3. ✅ Creates a GitHub Release with binaries
4. ✅ Publishes to crates.io
5. ✅ Comments on the PR with release status

## Release Steps

### 1. Prepare Release Branch

```bash
# Create a release branch from main
git checkout main
git pull origin main
git checkout -b release/v0.1.0
```

### 2. Update Version

Update the version in `Cargo.toml` (workspace):

```toml
[workspace.package]
version = "0.1.0"  # Update this
```

### 3. Update CHANGELOG (Optional but Recommended)

Create or update `CHANGELOG.md` with release notes:

```markdown
## [0.1.0] - 2025-11-11

### Added
- HTTP methods support (GET, POST, PUT, DELETE, etc.)
- Cookie management with Netscape format
- Recursive downloads with HTML parsing
- Adaptive chunk sizing and dynamic connection tuning
- 30+ unit tests

### Changed
- Improved performance for small files with HTTP/2 multiplexing

### Fixed
- (List any bug fixes)
```

### 4. Commit Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "Bump version to 0.1.0"
git push origin release/v0.1.0
```

### 5. Create Pull Request

1. Go to GitHub and create a PR from `release/v0.1.0` to `main`
2. Title: "Release v0.1.0"
3. Description: Copy the changelog or describe the release
4. Request review (optional)

### 6. Merge PR

When you merge the PR, the automation kicks in:

1. **Auto-tag job** (2-3 minutes):
   - Extracts version from branch name (`release/v0.1.0` → `v0.1.0`)
   - Verifies version matches `Cargo.toml`
   - Creates and pushes git tag

2. **Build job** (10-15 minutes):
   - Builds binaries for:
     - Windows (x64)
     - Linux (x64, ARM64)
     - macOS (Intel, Apple Silicon)
   - Uploads as GitHub artifacts

3. **Release job** (2-3 minutes):
   - Downloads all build artifacts
   - Creates GitHub Release with binaries
   - Comments on PR: "🎉 Release published!"

4. **Publish-crates job** (5-10 minutes):
   - Publishes `wget-faster-lib` to crates.io
   - Waits 60 seconds for crates.io to update
   - Publishes `wget-faster-cli` to crates.io
   - Comments on PR: "✅ Release complete!"

## GitHub Secrets Required

The automation requires these secrets in your GitHub repository settings:

### CARGO_REGISTRY_TOKEN

Get your token from https://crates.io/me:

```bash
# Go to crates.io settings
# Click "New Token"
# Name: "GitHub Actions - wget-faster"
# Click "Generate"
```

Add to GitHub:
1. Go to repository Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `CARGO_REGISTRY_TOKEN`
4. Value: (paste your token)

## Verification

After the workflow completes:

### 1. Check GitHub Release

```bash
# Visit: https://github.com/YOUR_USERNAME/wget-faster/releases
# Verify:
# - Release v0.1.0 exists
# - All 5 binary artifacts are attached
# - Release notes are generated
```

### 2. Check crates.io

```bash
# Visit: https://crates.io/crates/wget-faster-lib
# Visit: https://crates.io/crates/wget-faster-cli
# Verify version 0.1.0 is published
```

### 3. Test Installation

```bash
# Install from crates.io
cargo install wget-faster-cli

# Verify version
wget-faster --version
# Should show: wget-faster 0.1.0
```

### 4. Test Binary Download

```bash
# Download from GitHub Release
curl -L -O https://github.com/YOUR_USERNAME/wget-faster/releases/download/v0.1.0/wget-faster-x86_64-unknown-linux-gnu.tar.gz

# Extract and test
tar xzf wget-faster-x86_64-unknown-linux-gnu.tar.gz
./wget-faster --version
```

## Troubleshooting

### Version Mismatch Error

If the workflow fails with "Version mismatch!":

```bash
# Ensure Cargo.toml version matches branch name
# Branch: release/v0.1.0
# Cargo.toml: version = "0.1.0"  (no 'v' prefix)
```

### Build Failures

Check the GitHub Actions logs:
```bash
# Go to: https://github.com/YOUR_USERNAME/wget-faster/actions
# Click on the failed workflow run
# Expand the failing step to see error details
```

### crates.io Publish Fails

Common issues:
- Token expired or invalid → Regenerate token
- Package name already taken → Update package names in Cargo.toml
- Missing metadata → Ensure readme, homepage, description are set

### Release Already Exists

The workflow automatically deletes and recreates releases, so you can:
- Close the PR without merging
- Fix issues in the release branch
- Push new commits
- Re-open and merge the PR

## Manual Release (Emergency)

If automation fails, you can release manually:

```bash
# 1. Create tag locally
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# 2. Build binaries
cargo build --release --target x86_64-unknown-linux-gnu
# ... repeat for other targets

# 3. Create GitHub Release manually
gh release create v0.1.0 \
  target/x86_64-unknown-linux-gnu/release/wget-faster \
  --title "Release v0.1.0" \
  --notes "See CHANGELOG.md"

# 4. Publish to crates.io
cargo publish --package wget-faster-lib
sleep 60
cargo publish --package wget-faster-cli
```

## Best Practices

### Semantic Versioning

Follow [SemVer](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.0.1): Bug fixes, backwards compatible

### Pre-release Testing

Before creating the release PR:

```bash
# Run all tests
cargo test --all-features

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build release mode
cargo build --release

# Test the binary
./target/release/wget-faster --version
```

### Release Cadence

- **Patch releases**: As needed for bug fixes
- **Minor releases**: Every 2-4 weeks for new features
- **Major releases**: When breaking changes are necessary

## Post-Release

After a successful release:

1. ✅ Announce on social media / blog (optional)
2. ✅ Update documentation if needed
3. ✅ Close related issues/PRs
4. ✅ Start planning next release
5. ✅ Delete the release branch (optional)

```bash
git branch -d release/v0.1.0
git push origin --delete release/v0.1.0
```

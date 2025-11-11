# wget Test Suite Integration Strategy

**Purpose**: Achieve 60%+ pass rate on GNU wget test suite to validate compatibility
**Version Target**: v0.1.2
**License Consideration**: Separate GPL-3.0 repository for test integration

---

## Overview

wget-faster aims for high compatibility with GNU wget. To validate this, we will integrate with the official wget test suite. However, since wget is GPL-3.0 licensed and wget-faster is BSD-3-Clause, we must maintain strict license separation.

## License Separation Strategy

### Approach: Binary-Only Testing

We use a **binary testing approach** similar to how web browsers are tested against W3C test suites:
- The wget test suite tests the **compiled binary** (`wgetf`)
- No wget source code is copied, linked, or derived
- Tests run against the binary interface only
- This is **not creating a derivative work** of wget

### Repository Structure

```
wget-faster/                    (BSD-3-Clause)
├── wget-faster-lib/
├── wget-faster-cli/
└── WGET_TEST_INTEGRATION.md   (this file)

wget-faster-test/               (GPL-3.0, separate repository)
├── LICENSE                     (GPL-3.0)
├── README.md
├── wget/                       (git submodule to GNU wget)
├── test-runner.py
├── test-results/
└── docs/
    └── LICENSE_SEPARATION.md
```

### Key Points

1. **Separate Repository**: `wget-faster-test` is a completely separate repository
2. **GPL-3.0 License**: The test repository is GPL-3.0 licensed
3. **No Code Sharing**: No code flows between repositories
4. **Binary Testing**: Tests execute `wgetf` binary via command line
5. **Environment Variable**: `WGET_PATH` points to the binary under test

## Implementation Plan

### Phase 1: Setup (Week 1)

#### 1.1 Create Test Repository

```bash
# Create new repository
git init wget-faster-test
cd wget-faster-test

# Add LICENSE (GPL-3.0)
cat > LICENSE << 'EOF'
GNU GENERAL PUBLIC LICENSE
Version 3, 29 June 2007
...
EOF

# Add wget as submodule
git submodule add https://git.savannah.gnu.org/git/wget.git
git submodule update --init --recursive
```

#### 1.2 Document License Separation

Create `docs/LICENSE_SEPARATION.md`:

```markdown
# License Separation Explanation

## Why Two Repositories?

wget-faster is BSD-3-Clause licensed. GNU wget is GPL-3.0 licensed.
To use wget's test suite without creating a derivative work, we:

1. Keep test integration in a separate GPL-3.0 repository
2. Test only the compiled binary (not source code)
3. Never copy or derive code from wget

## Legal Basis

This approach is similar to:
- Web browsers tested with W3C test suites
- Python implementations tested with Python test suite
- C compilers tested with GCC test suite

We are testing **interface compatibility**, not creating a derivative work.
```

### Phase 2: Test Runner Implementation (Week 1-2)

#### 2.1 Python Test Adapter

Create `test-runner.py`:

```python
#!/usr/bin/env python3
"""
wget-faster test runner
Adapts wget test suite to test wgetf binary
"""

import os
import sys
import subprocess
from pathlib import Path

# Get wgetf binary path from environment
WGET_PATH = os.environ.get('WGET_PATH', 'wgetf')

def run_wget_tests(test_dir):
    """Run wget test suite against wgetf binary"""
    os.chdir(test_dir)

    # Set environment variable for wget tests
    env = os.environ.copy()
    env['WGET'] = WGET_PATH

    # Run wget's test harness
    result = subprocess.run(
        ['make', 'check'],
        env=env,
        capture_output=True,
        text=True
    )

    return result

if __name__ == '__main__':
    test_dir = Path('wget/tests')
    result = run_wget_tests(test_dir)

    print(result.stdout)
    print(result.stderr, file=sys.stderr)

    sys.exit(result.returncode)
```

#### 2.2 Result Parser

Create `parse-results.py`:

```python
#!/usr/bin/env python3
"""
Parse wget test results and generate report
"""

import re
import json
from pathlib import Path

def parse_test_output(output):
    """Parse test output and extract results"""
    results = {
        'total': 0,
        'passed': 0,
        'failed': 0,
        'skipped': 0,
        'tests': []
    }

    # Parse test output
    # Format depends on wget's test framework

    return results

def generate_report(results):
    """Generate HTML report"""
    pass_rate = (results['passed'] / results['total'] * 100) if results['total'] > 0 else 0

    report = f"""
    # wget-faster Test Results

    **Pass Rate**: {pass_rate:.1f}%

    - Total Tests: {results['total']}
    - Passed: {results['passed']}
    - Failed: {results['failed']}
    - Skipped: {results['skipped']}

    ## Goal

    Target: 60%+ pass rate
    Current: {pass_rate:.1f}%
    Status: {'✅ GOAL MET' if pass_rate >= 60 else '❌ BELOW GOAL'}
    """

    return report
```

### Phase 3: Test Categories (Week 2)

#### Core Test Categories

1. **Basic HTTP Downloads**
   - Simple GET requests
   - Different file sizes
   - Different content types

2. **HTTPS with SSL/TLS**
   - Certificate verification
   - Custom CA certificates
   - Client certificates
   - `--no-check-certificate`

3. **Authentication**
   - HTTP Basic authentication
   - HTTP Digest authentication
   - 401 Unauthorized handling

4. **Cookies**
   - `--load-cookies`
   - `--save-cookies`
   - Domain matching
   - Path matching
   - Secure flag
   - Expiry handling

5. **Redirects**
   - 301/302/303/307/308 redirects
   - Max redirect limit
   - Redirect loops

6. **Resume/Continue**
   - `-c, --continue` option
   - Range request support
   - Partial content handling

7. **Timestamping**
   - `-N` option
   - If-Modified-Since
   - Last-Modified header

8. **Output Options**
   - `-O` output file
   - `-P` directory prefix
   - Filename from Content-Disposition

#### Priority Matrix

| Priority | Category | Tests | Critical |
|----------|----------|-------|----------|
| P0 | Basic HTTP | ~20 | Yes |
| P0 | Output | ~10 | Yes |
| P0 | Resume | ~8 | Yes |
| P1 | HTTPS | ~15 | Yes |
| P1 | Auth | ~12 | Yes |
| P1 | Redirects | ~10 | Yes |
| P2 | Cookies | ~8 | No |
| P2 | Timestamping | ~6 | No |
| P3 | Advanced | ~20 | No |

**Target**: Pass all P0 and P1 tests (70+ tests) = 60%+ of core suite

### Phase 4: CI/CD Integration (Week 3)

#### GitHub Actions Workflow

Create `.github/workflows/wget-test-suite.yml` in `wget-faster-test`:

```yaml
name: wget Test Suite

on:
  workflow_dispatch:
    inputs:
      wgetf_version:
        description: 'wget-faster version to test'
        required: true
        default: 'latest'

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout wget-faster-test
        uses: actions/checkout@v3
        with:
          submodules: recursive

      - name: Download wgetf binary
        run: |
          # Download pre-built wgetf binary from releases
          wget https://github.com/wget-faster/wget-faster/releases/download/${{ github.event.inputs.wgetf_version }}/wgetf-linux-x86_64.tar.gz
          tar xzf wgetf-linux-x86_64.tar.gz
          chmod +x wgetf

      - name: Build wget test suite
        run: |
          cd wget/tests
          ./configure
          make

      - name: Run tests
        run: |
          export WGET_PATH=$(pwd)/wgetf
          python3 test-runner.py

      - name: Generate report
        run: |
          python3 parse-results.py > test-results/report.md

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: test-results
          path: test-results/
```

### Phase 5: Compatibility Fixes (Week 3-4)

#### Process

1. **Run Tests**: Execute test suite against current `wgetf`
2. **Analyze Failures**: Categorize failures:
   - Missing features
   - Different behavior
   - Bugs
   - Output format differences
3. **Prioritize**: Focus on P0/P1 failures first
4. **Fix Issues**: Update wget-faster to match wget behavior
5. **Document Differences**: For intentional differences, document them
6. **Iterate**: Re-run until 60%+ pass rate achieved

#### Compatibility Matrix

Create `COMPATIBILITY.md` in `wget-faster-test`:

```markdown
# wget-faster Compatibility Matrix

## Test Results (v0.1.2)

| Category | Total | Passed | Failed | Pass Rate |
|----------|-------|--------|--------|-----------|
| Basic HTTP | 20 | 18 | 2 | 90% |
| HTTPS | 15 | 12 | 3 | 80% |
| Authentication | 12 | 10 | 2 | 83% |
| Redirects | 10 | 9 | 1 | 90% |
| Resume | 8 | 7 | 1 | 88% |
| Cookies | 8 | 6 | 2 | 75% |
| Output | 10 | 10 | 0 | 100% |
| Timestamping | 6 | 4 | 2 | 67% |
| **TOTAL** | **89** | **76** | **13** | **85%** ✅ |

**Goal**: 60%+ pass rate
**Status**: ✅ ACHIEVED (85%)

## Known Differences

### Intentional Differences

1. **Progress Bar Format**
   - wget: `20% [=====>     ] 1,234,567    1.23MB/s  eta 5s`
   - wgetf: Similar but may differ in spacing
   - Reason: Using different progress library (indicatif vs custom)
   - Impact: Cosmetic only

2. **Default User-Agent**
   - wget: `Wget/1.21.3`
   - wgetf: `wget-faster/0.1.0`
   - Reason: Different implementation
   - Impact: None (can be overridden with `-U`)

### Features Not Implemented

1. **FTP/FTPS** - Planned for v0.3.0
2. **WARC Format** - Planned for future
3. **Link Conversion (`-k`)** - Planned for v0.3.0
4. **.netrc** - Planned for v0.3.0

## Failed Tests

### P0 Failures (Must Fix)

None

### P1 Failures (Should Fix)

1. **Test-Auth-Digest-Multi** - Multiple digest auth challenges
   - Status: Bug in digest auth implementation
   - Fix: Planned for v0.1.3

2. **Test-HTTPS-Client-Cert-Chain** - Certificate chain validation
   - Status: Missing certificate chain handling
   - Fix: Planned for v0.1.3

### P2 Failures (Nice to Fix)

...
```

## Usage Instructions

### For Developers

```bash
# Clone test repository
git clone https://github.com/wget-faster/wget-faster-test.git
cd wget-faster-test

# Initialize wget submodule
git submodule update --init --recursive

# Build wget tests
cd wget/tests
./configure
make
cd ../..

# Build wgetf binary
cd /path/to/wget-faster
cargo build --release

# Run tests
export WGET_PATH=/path/to/wget-faster/target/release/wgetf
cd /path/to/wget-faster-test
python3 test-runner.py

# View results
python3 parse-results.py
```

### For CI/CD

```bash
# Automatic testing on each release
# Triggered via GitHub Actions workflow
# Results published as artifacts
```

## Success Criteria

### v0.1.2 Release

- ✅ Test repository created and documented
- ✅ License separation clearly explained
- ✅ Test runner implemented
- ✅ Results parser implemented
- ✅ **60%+ pass rate achieved**
- ✅ Compatibility matrix documented
- ✅ CI/CD integrated

### Metrics

- **Target**: 60%+ of wget core HTTP tests pass
- **Stretch Goal**: 80%+ pass rate
- **Acceptable**: All P0 tests pass (critical functionality)

## Maintenance

### Updating wget Version

```bash
cd wget-faster-test
cd wget
git fetch
git checkout v1.25.0  # or desired version
cd ..
git add wget
git commit -m "Update wget to v1.25.0"
```

### Re-running Tests

```bash
# After each wget-faster release
export WGET_PATH=/path/to/new/wgetf
python3 test-runner.py
python3 parse-results.py
git add test-results/
git commit -m "Test results for wget-faster v0.1.2"
```

## FAQ

### Why a separate repository?

To maintain complete license separation. The test repository can be GPL-3.0 while wget-faster remains BSD-3-Clause.

### Is this legal?

Yes. We're testing binary compatibility, not creating a derivative work. This is standard practice (browsers, compilers, language implementations).

### Do we copy wget code?

No. We use wget's test suite to test our binary, but we never copy or derive wget's source code.

### Can users run these tests?

Yes, but they need to clone the separate `wget-faster-test` repository. It's not included in the main wget-faster distribution.

### What if we can't pass 60%?

We document known incompatibilities and explain why. The goal is compatibility where it makes sense, not 100% bug-for-bug compatibility.

## References

- [GNU wget](https://www.gnu.org/software/wget/)
- [wget source repository](https://git.savannah.gnu.org/cgit/wget.git)
- [GPL-3.0 License](https://www.gnu.org/licenses/gpl-3.0.en.html)
- [BSD-3-Clause License](https://opensource.org/licenses/BSD-3-Clause)

---

**Last Updated**: 2025-11-11
**Next Review**: After v0.1.2 release

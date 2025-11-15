#!/usr/bin/env python3
"""
Analyze wget test results and generate individual test failure documentation.
"""

import json
import sys
from pathlib import Path
from collections import defaultdict

def load_test_results(json_path):
    """Load test results from JSON file."""
    with open(json_path) as f:
        return json.load(f)

def categorize_failure(test):
    """Categorize the failure reason."""
    category = test.get('failure_category', 'unknown')
    if not category or category.strip() == '':
        category = 'unknown'
    error_msg = test.get('error_message', '')
    stderr = test.get('stderr', '')

    # Auto-categorize from error messages if category is unknown/empty
    if category == 'unknown':
        # Check stderr for patterns
        if '--input-metalink' in error_msg or 'metalink' in stderr.lower():
            return 'missing_feature_metalink'
        if 'Not all files were crawled correctly' in stderr:
            return 'test_framework_crawl_mismatch'
        if 'do not match' in stderr or 'Contents of' in stderr and 'do not match' in stderr:
            return 'test_framework_content_mismatch'
        if 'Expected file' in stderr and 'not found' in stderr:
            return 'test_framework_missing_file'
        if 'No such file or directory' in stderr:
            return 'test_framework_missing_file'
        if test.get('exit_code') == 77:
            return 'skipped_ssl_tls'
        if 'unexpected argument' in stderr.lower():
            return 'missing_cli_option'
        if 'builder error' in stderr.lower() and 'ftp' in stderr.lower():
            return 'missing_feature_ftp'
        # Check for test pass but exit code mismatch
        if 'Test Passed' in stderr or 'passed' in stderr.lower():
            return 'exit_code_mismatch'

    # More detailed categorization for known categories
    if category == 'missing_feature':
        if '--input-metalink' in error_msg or 'metalink' in stderr.lower():
            return 'missing_feature_metalink'
        return 'missing_feature_other'

    if category == 'test_framework_error':
        if 'Not all files were crawled correctly' in stderr:
            return 'test_framework_crawl_mismatch'
        if 'do not match' in stderr:
            return 'test_framework_content_mismatch'
        if 'Expected file' in stderr and 'not found' in stderr:
            return 'test_framework_missing_file'
        return 'test_framework_other'

    if category == 'skipped':
        return 'skipped_ssl_tls'

    if category == 'timeout':
        return 'timeout'

    return category

def generate_test_doc(test, test_type):
    """Generate markdown documentation for a failed test."""
    test_name = test['test_name']

    # Sanitize test name for filename
    safe_name = test_name.replace('--', '_').replace('.', '_')

    doc = f"""# {test_name}

**Test Type**: {test_type}
**Status**: ❌ FAILED
**Category**: {test.get('failure_category', 'unknown')}
**Execution Time**: {test.get('execution_time', 0):.2f}s

## Description

{test.get('description', 'No description available')}

## Error Details

**Error Message**: {test.get('error_message', 'No error message')}

**Exit Code**: {test.get('exit_code', 'unknown')}

## Test Output

### stdout
```
{test.get('stdout', 'No stdout')}
```

### stderr
```
{test.get('stderr', 'No stderr')}
```

## Analysis

"""

    # Add automatic analysis based on error patterns
    stderr = test.get('stderr', '')
    error_msg = test.get('error_message', '')

    if 'Not all files were crawled correctly' in stderr:
        doc += """
**Issue Type**: File crawling mismatch

The test expects certain files to be downloaded/crawled, but wget-faster either:
- Downloaded different files
- Missed some files
- Downloaded extra files

**Possible causes**:
1. Link extraction logic difference
2. Recursive download filtering issue
3. Spider mode behavior difference
4. robots.txt handling difference

**Next steps**:
1. Check which files were expected vs actual
2. Review link extraction in recursive.rs
3. Compare with GNU wget behavior
"""
    elif 'do not match' in stderr:
        doc += """
**Issue Type**: Content mismatch

The downloaded file content doesn't match expected content.

**Possible causes**:
1. Incorrect response handling
2. Encoding issue
3. Header processing difference
4. Content modification issue

**Next steps**:
1. Compare actual vs expected file content
2. Check HTTP response handling in downloader.rs
3. Review content encoding/decoding
"""
    elif 'Expected file' in stderr and 'not found' in stderr:
        doc += """
**Issue Type**: Missing file

Expected file was not created or saved to wrong location.

**Possible causes**:
1. File naming issue
2. Directory structure difference
3. File not downloaded
4. Path resolution issue

**Next steps**:
1. Check file naming logic in recursive.rs
2. Verify directory creation
3. Check if download was attempted
"""
    elif '--input-metalink' in error_msg:
        doc += """
**Issue Type**: Missing feature - Metalink

Metalink support is not implemented in wget-faster.

**Status**: Deferred to v0.2.0+

**Impact**: 32 tests (19% of all tests)

**Priority**: Low - not critical for current goals
"""
    elif test.get('failure_category') == 'skipped':
        doc += """
**Issue Type**: Feature not available (skipped)

Test requires SSL/TLS features that are not configured.

**Possible features**:
- Client certificates
- CRL (Certificate Revocation List)
- Custom CA certificates
- Specific TLS versions

**Status**: Deferred to v0.2.0+

**Priority**: Medium - needed for advanced HTTPS
"""
    elif test.get('failure_category') == 'timeout':
        doc += """
**Issue Type**: Test timeout

Test exceeded maximum execution time (usually 60s).

**Possible causes**:
1. Infinite loop or hang
2. Network issue
3. Auth challenge loop
4. Resource deadlock

**Priority**: High - indicates serious bug

**Next steps**:
1. Run test manually with debug output
2. Check for infinite loops
3. Review auth handling
"""

    doc += """
## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_
"""

    return doc

def main():
    if len(sys.argv) < 2:
        print("Usage: analyze_tests.py <test_results.json>")
        sys.exit(1)

    json_path = Path(sys.argv[1])
    if not json_path.exists():
        print(f"Error: {json_path} not found")
        sys.exit(1)

    print(f"Analyzing test results from {json_path}...")

    data = load_test_results(json_path)

    # Statistics
    perl_tests = data.get('perl_tests', [])
    python_tests = data.get('python_tests', [])

    perl_failed = [t for t in perl_tests if not t['passed']]
    python_failed = [t for t in python_tests if not t['passed']]

    print(f"\nPerl tests: {len(perl_tests)} total, {len(perl_failed)} failed")
    print(f"Python tests: {len(python_tests)} total, {len(python_failed)} failed")

    # Categorize failures
    categories = defaultdict(list)
    for test in perl_failed:
        cat = categorize_failure(test)
        categories[cat].append(('perl', test))
    for test in python_failed:
        cat = categorize_failure(test)
        categories[cat].append(('python', test))

    print(f"\nFailure categories:")
    for cat, tests in sorted(categories.items(), key=lambda x: -len(x[1])):
        print(f"  {cat}: {len(tests)}")

    # Create todo directory
    todo_dir = Path('todo')
    todo_dir.mkdir(exist_ok=True)

    # Generate documentation for each failed test
    print(f"\nGenerating test documentation in {todo_dir}/...")

    generated = 0
    for test_type, tests in [('perl', perl_failed), ('python', python_failed)]:
        for test in tests:
            test_name = test['test_name']
            safe_name = test_name.replace('--', '_').replace('.', '_')

            doc_path = todo_dir / f"{safe_name}.md"
            doc_content = generate_test_doc(test, test_type)

            with open(doc_path, 'w') as f:
                f.write(doc_content)

            generated += 1

    print(f"Generated {generated} test documentation files")

    # Generate summary index
    index_path = todo_dir / 'README.md'
    with open(index_path, 'w') as f:
        f.write(f"""# Test Failure Analysis

**Generated from**: {json_path.name}
**Timestamp**: {data.get('timestamp', 'unknown')}
**Total Failed**: {len(perl_failed) + len(python_failed)} / {len(perl_tests) + len(python_tests)}

## Summary by Category

""")

        for cat, tests in sorted(categories.items(), key=lambda x: -len(x[1])):
            f.write(f"\n### {cat} ({len(tests)} tests)\n\n")
            for test_type, test in tests:
                test_name = test['test_name']
                safe_name = test_name.replace('--', '_').replace('.', '_')
                f.write(f"- [{test_name}](./{safe_name}.md) ({test_type})\n")

    print(f"\nCreated index at {index_path}")
    print("\nDone!")

if __name__ == '__main__':
    main()

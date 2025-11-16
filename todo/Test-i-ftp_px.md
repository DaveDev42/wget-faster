# Test-i-ftp.px

**Test Type**: perl
**Status**: ❌ FAILED
**Category**: 
**Execution Time**: 0.03s

## Description



## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 0

## Test Output

### stdout
```
Running test Test-i-ftp
Calling /Users/dave/.cargo/bin/wgetf -d --no-config -i ftp://localhost:58560/urls.txt
Test failed: wrong code returned (was: 1, expected: 0)

```

### stderr
```
wgetf: failed to read input file from URL: Failed to download input file from URL: ftp://localhost:58560/urls.txt

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

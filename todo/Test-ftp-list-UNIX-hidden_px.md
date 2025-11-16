# Test-ftp-list-UNIX-hidden.px

**Test Type**: perl
**Status**: ❌ FAILED
**Category**: 
**Execution Time**: 0.04s

## Description

2013-10-17 Andrea Urbani (matfanjol) In this ftp test:

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 0

## Test Output

### stdout
```
Running test Test-ftp-list-UNIX-hidden
Calling /Users/dave/.cargo/bin/wgetf -d --no-config --no-directories --recursive --level=1 ftp://localhost:51454/
Test failed: wrong code returned (was: 1, expected: 0)

```

### stderr
```
wgetf: recursive download failed: HTTP request failed: builder error for url (ftp://localhost:51454/)

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

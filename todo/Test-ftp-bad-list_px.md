# Test-ftp-bad-list.px

**Test Type**: perl
**Status**: ❌ FAILED
**Category**: 
**Execution Time**: 0.04s

## Description



## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 0

## Test Output

### stdout
```
Running test Test-ftp-bad-list
Calling /Users/dave/.cargo/bin/wgetf -d --no-config -nH -Nc -r ftp://localhost:51451/
Test failed: wrong code returned (was: 1, expected: 0)

```

### stderr
```
wgetf: recursive download failed: HTTP request failed: builder error for url (ftp://localhost:51451/)

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

# Test-ftp--start-pos.px

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
Running test Test-ftp--start-pos
Calling /Users/dave/.cargo/bin/wgetf -d --no-config --start-pos=1 ftp://localhost:51450/dummy.txt
--2025-11-16 12:50:12--  ftp://localhost:51450/dummy.txt
Resolving localhost... 
Connecting to localhost:51450... connected.
HTTP request sent, awaiting response... 
Saving to: 'dummy.txt'

Test failed: wrong code returned (was: 1, expected: 0)

```

### stderr
```
wget-faster: download failed: HTTP request failed: builder error for url (ftp://localhost:51450/dummy.txt)
wgetf: HTTP request failed: builder error for url (ftp://localhost:51450/dummy.txt)

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

# Test-ftp-pasv-not-supported.px

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
Running test Test-ftp-pasv-not-supported
Calling /Users/dave/.cargo/bin/wgetf -d --no-config -S ftp://localhost:58557/afile.txt
--2025-11-16 13:53:05--  ftp://localhost:58557/afile.txt
Resolving localhost... 
Connecting to localhost:58557... connected.
HTTP request sent, awaiting response... 
Saving to: 'afile.txt'

Test failed: wrong code returned (was: 1, expected: 8)

```

### stderr
```
wget-faster: download failed: HTTP request failed: builder error for url (ftp://localhost:58557/afile.txt)
wgetf: HTTP request failed: builder error for url (ftp://localhost:58557/afile.txt)

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

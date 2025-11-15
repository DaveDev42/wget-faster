# Test-proxy-auth-basic.px

**Test Type**: perl
**Status**: ❌ FAILED
**Category**: 
**Execution Time**: 0.07s

## Description



## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 0

## Test Output

### stdout
```
Running test Test-proxy-auth-basic
Calling /Users/dave/.cargo/bin/wgetf -d --no-config --user=fiddle-dee-dee --password=Dodgson -e http_proxy=localhost:56478 http://no.such.domain/needs-auth.txt
--2025-11-15 13:43:06--  http://no.such.domain/needs-auth.txt
Resolving no.such.domain... 
Connecting to no.such.domain:80... connected.
HTTP request sent, awaiting response... 
Saving to: 'needs-auth.txt'

Test failed: wrong code returned (was: 4, expected: 0)

```

### stderr
```
wget-faster: download failed: HTTP request failed: error sending request for url (http://no.such.domain/needs-auth.txt)
wgetf: HTTP request failed: error sending request for url (http://no.such.domain/needs-auth.txt)

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

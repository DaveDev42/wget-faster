# Test-proxied-https-auth.px

**Test Type**: perl
**Status**: ❌ FAILED
**Category**: 
**Execution Time**: 0.08s

## Description

Simulate a tunneling proxy to a HTTPS URL that needs authentication. Use two connections (Connection: close)

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 0

## Test Output

### stdout
```
--2025-11-16 13:36:44--  https://no.such.domain/needs-auth.txt
Resolving no.such.domain... 
Connecting to no.such.domain:443... connected.
HTTP request sent, awaiting response... 
Saving to: 'needs-auth.txt'

wget-faster: download failed: HTTP request failed: error sending request for url (https://no.such.domain/needs-auth.txt)
wgetf: HTTP request failed: error sending request for url (https://no.such.domain/needs-auth.txt)

```

### stderr
```
Got code: 4

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

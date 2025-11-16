# Test-N--no-content-disposition.px

**Test Type**: perl
**Status**: ❌ FAILED
**Category**: 
**Execution Time**: 0.06s

## Description



## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 0

## Test Output

### stdout
```
Running test Test-N--no-content-disposition
Calling /Users/dave/.cargo/bin/wgetf -d --no-config -N --no-content-disposition http://localhost:51388/dummy.txt
--2025-11-16 12:50:11--  http://localhost:51388/dummy.txt
Resolving localhost... 
Connecting to localhost:51388... connected.
HTTP request sent, awaiting response... 
Saving to: 'dummy.txt'

200 OK
Length: unspecified

2025-11-16 12:50:11 - 'dummy.txt' saved [8]

Test failed: wrong timestamp for file dummy.txt: expected = 1097310600, actual = 1763265011

```

### stderr
```

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

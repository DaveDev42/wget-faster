# Test-N-HTTP-Content-Disposition.px

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
Running test Test-N-HTTP-Content-Disposition
Calling /Users/dave/.cargo/bin/wgetf -d --no-config -N --content-disposition http://localhost:51390/dummy.txt
--2025-11-16 12:50:11--  http://localhost:51390/dummy.txt
Resolving localhost... 
Connecting to localhost:51390... connected.
HTTP request sent, awaiting response... 
Saving to: 'filename.txt'

200 OK
Length: unspecified

2025-11-16 12:50:11 - 'filename.txt' saved [12]

Test failed: wrong timestamp for file filename.txt: expected = 1097310600, actual = 1763265011

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

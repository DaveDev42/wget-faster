# Test-N-current.px

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
Running test Test-N-current
Calling /Users/dave/.cargo/bin/wgetf -d --no-config -N http://localhost:55774/somefile.txt
--2025-11-16 13:36:41--  http://localhost:55774/somefile.txt
Resolving localhost... 
Connecting to localhost:55774... connected.
HTTP request sent, awaiting response... 
Saving to: 'somefile.txt'

200 OK
Length: unspecified

2025-11-16 13:36:41 - 'somefile.txt' saved [295]

Test failed: wrong content for file somefile.txt
Mismatch at line 1, col 21:
    1111111111
    11111x1111

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

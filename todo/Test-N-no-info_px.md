# Test-N-no-info.px

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
Running test Test-N-no-info
Calling /Users/dave/.cargo/bin/wgetf -d --no-config -N http://localhost:55776/somefile.txt
--2025-11-16 13:36:41--  http://localhost:55776/somefile.txt
Resolving localhost... 
Connecting to localhost:55776... connected.
HTTP request sent, awaiting response... 
Saving to: 'somefile.txt'

200 OK
Length: unspecified

2025-11-16 13:36:41 - 'somefile.txt' saved [112]

Test failed: wrong content for file somefile.txt
Sizes don't match: expected = 112, actual = 295
Mismatch at line 3, col 0:
    2222\n
    2222\n33333

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

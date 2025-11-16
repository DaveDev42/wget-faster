# Test-iri-disabled.px

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
Running test Test-iri-disabled
Calling /Users/dave/.cargo/bin/wgetf -d --no-config --no-iri -nH -r http://localhost:55851/
Test failed: wrong code returned (was: 1, expected: 0)

```

### stderr
```
wgetf: recursive download failed: IO error: stream did not contain valid UTF-8

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

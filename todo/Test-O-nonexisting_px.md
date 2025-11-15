# Test-O-nonexisting.px

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
Running test Test-O-nonexisting
Calling /Users/dave/.cargo/bin/wgetf -d --no-config --quiet -O out http://localhost:56398/nonexistent
Test failed: file out not downloaded

```

### stderr
```
wget-faster: download failed: Invalid response status: 400
wgetf: Invalid response status: 400

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

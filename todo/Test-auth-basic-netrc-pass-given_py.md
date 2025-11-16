# Test-auth-basic-netrc-pass-given.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.59s

## Description

This test ensures Wget uses credentials from .netrc for Basic Authorization Negotiation. In this case we test that .netrc credentials are used in case only

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
--2025-11-16 12:50:20--  http://localhost:51563/File1
Resolving localhost... 
Connecting to localhost:51563... connected.
HTTP request sent, awaiting response... 
Saving to: 'File1'

--2025-11-16 12:50:20--  http://localhost:51563/File2
Resolving localhost... 
Connecting to localhost:51563... connected.
HTTP request sent, awaiting response... 
Saving to: 'File2'

Running Test Test-auth-basic-netrc-pass-given.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --password=TheEye http://localhost:51563/File1 http://localhost:51563/File2 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--password=TheEye', 'http://localhost:51563/File1', 'http://localhost:51563/File2']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-auth-basic-netrc-pass-given.py-test'}
Unable to Authenticate
Header Authorization not found
Error: Expected file File1 not found..

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 12:50:20] "HEAD /File1 HTTP/1.1" 401 -
wget-faster: download failed: Invalid response status: 401
wgetf: Invalid response status: 401
127.0.0.1 - - [16/Nov/2025 12:50:20] code 400, message Expected Header Authorization not found
127.0.0.1 - - [16/Nov/2025 12:50:20] "HEAD /File2 HTTP/1.1" 400 -
wget-faster: download failed: Invalid response status: 400
wgetf: Invalid response status: 400
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-auth-basic-netrc-pass-given.py", line 66, in <module>
    ).begin ()
      ~~~~~~^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/test/http_test.py", line 41, in begin
    self.do_test()
    ~~~~~~~~~~~~^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/test/base_test.py", line 198, in do_test
    self.post_hook_call()
    ~~~~~~~~~~~~~~~~~~~^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/test/base_test.py", line 217, in post_hook_call
    self.hook_call(self.post_configs, 'Post Test Function')
    ~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/test/base_test.py", line 207, in hook_call
    conf.find_conf(conf_name)(conf_arg)(self)
    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/conf/expected_files.py", line 66, in __call__
    raise TestFailed("Expected file %s not found." % file.name)
exc.test_failed.TestFailed: Expected file File1 not found.

```

## Analysis


**Issue Type**: Missing file

Expected file was not created or saved to wrong location.

**Possible causes**:
1. File naming issue
2. Directory structure difference
3. File not downloaded
4. Path resolution issue

**Next steps**:
1. Check file naming logic in recursive.rs
2. Verify directory creation
3. Check if download was attempted

## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

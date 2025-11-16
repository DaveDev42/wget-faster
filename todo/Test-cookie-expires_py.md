# Test-cookie-expires.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.60s

## Description

This test ensures that Wget handles Cookie expiry dates correctly. Simultaneuously, we also check if multiple cookies to the same domain

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
--2025-11-16 13:36:54--  http://localhost:56010/File1
Resolving localhost... 
Connecting to localhost:56010... connected.
HTTP request sent, awaiting response... 
Saving to: 'File1'

200 OK
Length: 12 (12B) [text/plain]

2025-11-16 13:36:54 - 'File1' saved [12]

--2025-11-16 13:36:54--  http://localhost:56010/File2
Resolving localhost... 
Connecting to localhost:56010... connected.
HTTP request sent, awaiting response... 
Saving to: 'File2'

--2025-11-16 13:36:54--  http://localhost:56010/File3
Resolving localhost... 
Connecting to localhost:56010... connected.
HTTP request sent, awaiting response... 
Saving to: 'File3'

--2025-11-16 13:36:54--  http://localhost:56010/File4
Resolving localhost... 
Connecting to localhost:56010... connected.
HTTP request sent, awaiting response... 
Saving to: 'File4'

Running Test Test-cookie-expires.py
/Users/dave/.cargo/bin/wgetf --debug --no-config  http://localhost:56010/File1 http://localhost:56010/File2 http://localhost:56010/File3 http://localhost:56010/File4 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', 'http://localhost:56010/File1', 'http://localhost:56010/File2', 'http://localhost:56010/File3', 'http://localhost:56010/File4']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-cookie-expires.py-test'}
Header Cookie not found
Header Cookie not found
Header Cookie not found
Error: Contents of File2 do not match.

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 13:36:54] "HEAD /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:36:54] "GET /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:36:54] "HEAD /File2 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:36:54] code 400, message Expected Header Cookie not found
127.0.0.1 - - [16/Nov/2025 13:36:54] "GET /File2 HTTP/1.1" 400 -
wget-faster: download failed: Invalid response status: 400
wgetf: Invalid response status: 400
127.0.0.1 - - [16/Nov/2025 13:36:54] code 400, message Expected Header Cookie not found
127.0.0.1 - - [16/Nov/2025 13:36:54] "HEAD /File3 HTTP/1.1" 400 -
wget-faster: download failed: Invalid response status: 400
wgetf: Invalid response status: 400
127.0.0.1 - - [16/Nov/2025 13:36:54] code 400, message Expected Header Cookie not found
127.0.0.1 - - [16/Nov/2025 13:36:54] "HEAD /File4 HTTP/1.1" 400 -
wget-faster: download failed: Invalid response status: 400
wgetf: Invalid response status: 400
--- Actual
+++ Expected
@@ -0,0 +1 @@
+'Ello! This is Amazing!
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-cookie-expires.py", line 77, in <module>
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
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/conf/expected_files.py", line 64, in __call__
    raise TestFailed("Contents of %s do not match" % file.name)
exc.test_failed.TestFailed: Contents of File2 do not match

```

## Analysis


**Issue Type**: Content mismatch

The downloaded file content doesn't match expected content.

**Possible causes**:
1. Incorrect response handling
2. Encoding issue
3. Header processing difference
4. Content modification issue

**Next steps**:
1. Compare actual vs expected file content
2. Check HTTP response handling in downloader.rs
3. Review content encoding/decoding

## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

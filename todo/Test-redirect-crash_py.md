# Test-redirect-crash.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.10s

## Description



## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test-redirect-crash.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --recursive -e robots=off http://localhost:56130/File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/index.html 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--recursive', '-e', 'robots=off', 'http://localhost:56130/File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/index.html']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-redirect-crash.py-test'}
Custom Response code sent.
Custom Response code sent.
Error: Return codes do not match.
Expected: 0
Actual: 1.

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/index.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/index.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:37:18] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/directory HTTP/1.1" 301 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/directory/ HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/directory HTTP/1.1" 301 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/directory/ HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/directory/ HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /File%20formats/Images/SVG,%20Scalable%20Vector%20Graphics/html,%20W3C%20v1.2%20rec%20(tiny)/directory/ HTTP/1.1" 200 -
wgetf: recursive download failed: IO error: No such file or directory (os error 2)
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-redirect-crash.py", line 70, in <module>
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
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/conf/expected_ret_code.py", line 27, in __call__
    raise TestFailed(failure)
exc.test_failed.TestFailed: Return codes do not match.
Expected: 0
Actual: 1

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

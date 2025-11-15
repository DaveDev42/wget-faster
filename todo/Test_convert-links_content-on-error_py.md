# Test--convert-links--content-on-error.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.63s

## Description

This test ensures that Wget link conversion works also on HTTP error pages.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test--convert-links--content-on-error.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --no-host-directories -r -l2 --convert-links --content-on-error http://localhost:56482/a/x.html 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--no-host-directories', '-r', '-l2', '--convert-links', '--content-on-error', 'http://localhost:56482/a/x.html']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test--convert-links--content-on-error.py-test'}
Custom Response code sent.
Custom Response code sent.
Error: Contents of a/x.html do not match.

```

### stderr
```
127.0.0.1 - - [15/Nov/2025 13:43:07] "HEAD /a/x.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "GET /a/x.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "HEAD /robots.txt HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "GET /robots.txt HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "HEAD /b/y.html HTTP/1.1" 404 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "GET /b/y.html HTTP/1.1" 404 -
--- Actual
+++ Expected
@@ -6,6 +6,6 @@
 	<title></title>
 </head>
 <body>
-	<a href="b/y.html"></a>
+	<a href="../b/y.html"></a>
 </body>
 </html>

Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test--convert-links--content-on-error.py", line 75, in <module>
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
exc.test_failed.TestFailed: Contents of a/x.html do not match

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

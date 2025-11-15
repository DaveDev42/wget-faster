# Test-k.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.10s

## Description

Test that Wget handles the --convert-links (-k) option correctly. Also tests that the --restrict-file-names option works as expected by using a

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Darwin
Running Test Test-k.py
/Users/dave/.cargo/bin/wgetf --debug --no-config -k -r -nH --restrict-file-names=unix http://localhost:56857/index.html 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '-k', '-r', '-nH', '--restrict-file-names=unix', 'http://localhost:56857/index.html']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-k.py-test'}
Error: Contents of index.html do not match.

```

### stderr
```
127.0.0.1 - - [15/Nov/2025 13:44:45] "HEAD /index.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:44:45] "GET /index.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:44:45] code 404, message Not Found
127.0.0.1 - - [15/Nov/2025 13:44:45] "HEAD /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [15/Nov/2025 13:44:45] code 404, message Not Found
127.0.0.1 - - [15/Nov/2025 13:44:45] "GET /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [15/Nov/2025 13:44:45] "HEAD /site;sub:.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:44:45] "GET /site;sub:.html HTTP/1.1" 200 -
--- Actual
+++ Expected
@@ -4,6 +4,6 @@
     <title>Index</title>
   </head>
   <body>
-    <a href="site;sub:.html">Site</a>
+    <a href="./site%3Bsub:.html">Site</a>
   </body>
 </html>

Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-k.py", line 83, in <module>
    ).begin()
      ~~~~~^^
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
exc.test_failed.TestFailed: Contents of index.html do not match

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

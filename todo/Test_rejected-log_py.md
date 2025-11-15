# Test--rejected-log.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.09s

## Description

This test executed Wget in recursive mode with a rejected log outputted.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test--rejected-log.py
/Users/dave/.cargo/bin/wgetf --debug --no-config -nd -r --rejected-log log.csv http://localhost:56485/index.html 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '-nd', '-r', '--rejected-log', 'log.csv', 'http://localhost:56485/index.html']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test--rejected-log.py-test'}
Error: Contents of log.csv do not match.

```

### stderr
```
127.0.0.1 - - [15/Nov/2025 13:43:07] "HEAD /index.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "GET /index.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "HEAD /robots.txt HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "GET /robots.txt HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "HEAD /secondpage.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "GET /secondpage.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "HEAD /thirdpage.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:43:07] "GET /thirdpage.html HTTP/1.1" 200 -
--- Actual
+++ Expected
@@ -1 +1,4 @@
-http://localhost:56485/dummy.txt: Rejected by robots.txt rules
+REASON	U_URL	U_SCHEME	U_HOST	U_PORT	U_PATH	U_PARAMS	U_QUERY	U_FRAGMENT	P_URL	P_SCHEME	P_HOST	P_PORT	P_PATH	P_PARAMS	P_QUERY	P_FRAGMENT
+BLACKLIST	http%3A//localhost%3A56485/index.html	SCHEME_HTTP	localhost	56485	index.html				http%3A//localhost%3A56485/secondpage.html	SCHEME_HTTP	localhost	56485	secondpage.html			
+ROBOTS	http%3A//localhost%3A56485/dummy.txt	SCHEME_HTTP	localhost	56485	dummy.txt				http%3A//localhost%3A56485/thirdpage.html	SCHEME_HTTP	localhost	56485	thirdpage.html			
+SPANNEDHOST	http%3A//no.such.domain/	SCHEME_HTTP	no.such.domain	80					http%3A//localhost%3A56485/thirdpage.html	SCHEME_HTTP	localhost	56485	thirdpage.html			

Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test--rejected-log.py", line 98, in <module>
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
exc.test_failed.TestFailed: Contents of log.csv do not match

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

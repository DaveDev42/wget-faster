# Test-recursive-redirect.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.59s

## Description

Basic test of --recursive.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test-recursive-redirect.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --recursive --no-host-directories --include-directories=a http://localhost:57032/a/File1.html 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--recursive', '--no-host-directories', '--include-directories=a', 'http://localhost:57032/a/File1.html']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-recursive-redirect.py-test'}
Custom Response code sent.
Custom Response code sent.
{'b/File3.html': {'content': "Surely you're joking Mr. Feynman"}}
Error: Extra files downloaded..

```

### stderr
```
127.0.0.1 - - [15/Nov/2025 13:45:07] "HEAD /a/File1.html HTTP/1.1" 301 -
127.0.0.1 - - [15/Nov/2025 13:45:07] "HEAD /b/File1.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:07] "GET /a/File1.html HTTP/1.1" 301 -
127.0.0.1 - - [15/Nov/2025 13:45:07] "GET /b/File1.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:07] code 404, message Not Found
127.0.0.1 - - [15/Nov/2025 13:45:07] "HEAD /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [15/Nov/2025 13:45:07] code 404, message Not Found
127.0.0.1 - - [15/Nov/2025 13:45:07] "GET /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [15/Nov/2025 13:45:07] "HEAD /a/File2.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:07] "GET /a/File2.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:07] "HEAD /b/File3.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:07] "GET /b/File3.html HTTP/1.1" 200 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-recursive-redirect.py", line 62, in <module>
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
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/conf/expected_files.py", line 69, in __call__
    raise TestFailed("Extra files downloaded.")
exc.test_failed.TestFailed: Extra files downloaded.

```

## Analysis


## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

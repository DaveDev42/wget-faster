# Test-no_proxy-env.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 1.14s

## Description

This test ensures, that domains with and without leftmost dot defined in no_proxy environment variable are accepted by wget. The idea is to use

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
--2025-11-16 13:53:37--  http://working1.localhost:58821/File1
Resolving working1.localhost... 
Connecting to working1.localhost:58821... connected.
HTTP request sent, awaiting response... 
Saving to: 'File1'

200 OK
Length: 24 (24B) [text/plain]

2025-11-16 13:53:37 - 'File1' saved [24]

--2025-11-16 13:53:37--  http://working1.localhost:58821/File2
Resolving working1.localhost... 
Connecting to working1.localhost:58821... connected.
HTTP request sent, awaiting response... 
Saving to: 'File2'

200 OK
Length: 20 (20B) [text/plain]

2025-11-16 13:53:37 - 'File2' saved [20]

--2025-11-16 13:53:37--  http://www.working1.localhost:58824/File1
Resolving www.working1.localhost... 
Connecting to www.working1.localhost:58824... connected.
HTTP request sent, awaiting response... 
Saving to: 'File1'

200 OK
Length: 24 (24B) [text/plain]

2025-11-16 13:53:37 - 'File1' saved [24]

--2025-11-16 13:53:37--  http://www.working1.localhost:58824/File2
Resolving www.working1.localhost... 
Connecting to www.working1.localhost:58824... connected.
HTTP request sent, awaiting response... 
Saving to: 'File2'

200 OK
Length: 20 (20B) [text/plain]

2025-11-16 13:53:37 - 'File2' saved [20]

--2025-11-16 13:53:37--  http://working2.localhost:58827/File1
Resolving working2.localhost... 
Connecting to working2.localhost:58827... connected.
HTTP request sent, awaiting response... 
Saving to: 'File1'

200 OK
Length: 24 (24B) [text/plain]

2025-11-16 13:53:37 - 'File1' saved [24]

--2025-11-16 13:53:37--  http://working2.localhost:58827/File2
Resolving working2.localhost... 
Connecting to working2.localhost:58827... connected.
HTTP request sent, awaiting response... 
Saving to: 'File2'

200 OK
Length: 20 (20B) [text/plain]

2025-11-16 13:53:37 - 'File2' saved [20]

Running Test Test-no_proxy-env.py
/Users/dave/.cargo/bin/wgetf --debug --no-config  http://working1.localhost:58821/File1 http://working1.localhost:58821/File2 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', 'http://working1.localhost:58821/File1', 'http://working1.localhost:58821/File2']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-no_proxy-env.py-test', 'http_proxy': 'nonexisting.localhost:8080', 'no_proxy': 'working1.localhost,.working2.localhost'}
Test Passed.
Running Test Test-no_proxy-env.py
/Users/dave/.cargo/bin/wgetf --debug --no-config  http://www.working1.localhost:58824/File1 http://www.working1.localhost:58824/File2 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', 'http://www.working1.localhost:58824/File1', 'http://www.working1.localhost:58824/File2']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-no_proxy-env.py-test', 'http_proxy': 'nonexisting.localhost:8080', 'no_proxy': 'working1.localhost,.working2.localhost'}
Test Passed.
Running Test Test-no_proxy-env.py
/Users/dave/.cargo/bin/wgetf --debug --no-config  http://working2.localhost:58827/File1 http://working2.localhost:58827/File2 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', 'http://working2.localhost:58827/File1', 'http://working2.localhost:58827/File2']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-no_proxy-env.py-test', 'http_proxy': 'nonexisting.localhost:8080', 'no_proxy': 'working1.localhost,.working2.localhost'}
Error: Return codes do not match.
Expected: 4
Actual: 0.

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 13:53:37] "HEAD /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "GET /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "HEAD /File2 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "GET /File2 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "HEAD /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "GET /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "HEAD /File2 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "GET /File2 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "HEAD /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "GET /File1 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "HEAD /File2 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:37] "GET /File2 HTTP/1.1" 200 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-no_proxy-env.py", line 125, in <module>
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
Expected: 4
Actual: 0

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

# Test-Proto.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.61s

## Description

This is a Prototype Test File. Ideally this File should be copied and edited to write new tests.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test-Proto.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --content-disposition --user=Sauron --password=TheEye http://localhost:51555/File1 http://localhost:51555/File2 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--content-disposition', '--user=Sauron', '--password=TheEye', 'http://localhost:51555/File1', 'http://localhost:51555/File2']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-Proto.py-test'}
Error: Expected file File1 not found..

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 12:50:20] "HEAD /File1 HTTP/1.1" 401 -
----------------------------------------
Exception occurred during processing of request from ('127.0.0.1', 51557)
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 326, in Authentication
    self.handle_auth(auth_rule)
    ~~~~~~~~~~~~~~~~^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 353, in handle_auth
    raise AuthError("Unable to Authenticate")
exc.server_error.AuthError: Unable to Authenticate

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 318, in _handle_request_noblock
    self.process_request(request, client_address)
    ~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 349, in process_request
    self.finish_request(request, client_address)
    ~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 362, in finish_request
    self.RequestHandlerClass(request, client_address, self)
    ~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 766, in __init__
    self.handle()
    ~~~~~~~~~~~^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/http/server.py", line 485, in handle
    self.handle_one_request()
    ~~~~~~~~~~~~~~~~~~~~~~~^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/http/server.py", line 473, in handle_one_request
    method()
    ~~~~~~^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 78, in do_HEAD
    self.send_head("HEAD")
    ~~~~~~~~~~~~~~^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 402, in send_head
    getattr(self, rule_name)(self.rules[rule_name])
    ~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 329, in Authentication
    self.send_challenge(auth_rule.auth_type, auth_rule.auth_parm)
    ~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 236, in send_challenge
    self.send_challenge("digest", auth_parm)
    ~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 248, in send_challenge
    if auth_parm['qop']:
       ~~~~~~~~~^^^^^^^
TypeError: 'NoneType' object is not subscriptable
----------------------------------------
wgetf: Failed to get metadata from: http://localhost:51555/File1
127.0.0.1 - - [16/Nov/2025 12:50:20] "HEAD /File2 HTTP/1.1" 401 -
----------------------------------------
Exception occurred during processing of request from ('127.0.0.1', 51559)
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 326, in Authentication
    self.handle_auth(auth_rule)
    ~~~~~~~~~~~~~~~~^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 353, in handle_auth
    raise AuthError("Unable to Authenticate")
exc.server_error.AuthError: Unable to Authenticate

During handling of the above exception, another exception occurred:

Traceback (most recent call last):
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 318, in _handle_request_noblock
    self.process_request(request, client_address)
    ~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 349, in process_request
    self.finish_request(request, client_address)
    ~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 362, in finish_request
    self.RequestHandlerClass(request, client_address, self)
    ~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/socketserver.py", line 766, in __init__
    self.handle()
    ~~~~~~~~~~~^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/http/server.py", line 485, in handle
    self.handle_one_request()
    ~~~~~~~~~~~~~~~~~~~~~~~^^
  File "/opt/homebrew/Cellar/python@3.14/3.14.0_1/Frameworks/Python.framework/Versions/3.14/lib/python3.14/http/server.py", line 473, in handle_one_request
    method()
    ~~~~~~^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 78, in do_HEAD
    self.send_head("HEAD")
    ~~~~~~~~~~~~~~^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 402, in send_head
    getattr(self, rule_name)(self.rules[rule_name])
    ~~~~~~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 329, in Authentication
    self.send_challenge(auth_rule.auth_type, auth_rule.auth_parm)
    ~~~~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/server/http/http_server.py", line 248, in send_challenge
    if auth_parm['qop']:
       ~~~~~~~~~^^^^^^^
TypeError: 'NoneType' object is not subscriptable
----------------------------------------
wgetf: Failed to get metadata from: http://localhost:51555/File2
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-Proto.py", line 71, in <module>
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

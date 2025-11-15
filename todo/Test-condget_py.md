# Test-condget.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.10s

## Description

Simple test for HTTP Conditional-GET Requests using the -N command

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
--2025-11-15 13:44:44--  http://localhost:56838/UpToDateFile1
Resolving localhost... 
Connecting to localhost:56838... connected.
HTTP request sent, awaiting response... 
Saving to: 'UpToDateFile1'

200 OK
Length: unspecified

2025-11-15 13:44:44 - 'UpToDateFile1' saved [14]

--2025-11-15 13:44:44--  http://localhost:56838/UpToDateFile2
Resolving localhost... 
Connecting to localhost:56838... connected.
HTTP request sent, awaiting response... 
Saving to: 'UpToDateFile2'

200 OK
Length: unspecified

2025-11-15 13:44:44 - 'UpToDateFile2' saved [14]

--2025-11-15 13:44:44--  http://localhost:56838/UpToDateFile3
Resolving localhost... 
Connecting to localhost:56838... connected.
HTTP request sent, awaiting response... 
Saving to: 'UpToDateFile3'

200 OK
Length: unspecified

2025-11-15 13:44:44 - 'UpToDateFile3' saved [14]

--2025-11-15 13:44:44--  http://localhost:56838/NewerFile
Resolving localhost... 
Connecting to localhost:56838... connected.
HTTP request sent, awaiting response... 
Saving to: 'NewerFile'

200 OK
Length: unspecified

2025-11-15 13:44:44 - 'NewerFile' saved [14]

--2025-11-15 13:44:44--  http://localhost:56838/UpdatedFile
Resolving localhost... 
Connecting to localhost:56838... connected.
HTTP request sent, awaiting response... 
Saving to: 'UpdatedFile'

200 OK
Length: 14 (14B) [text/plain]

2025-11-15 13:44:44 - 'UpdatedFile' saved [14]

Running Test Test-condget.py
/Users/dave/.cargo/bin/wgetf --debug --no-config -N http://localhost:56838/UpToDateFile1 http://localhost:56838/UpToDateFile2 http://localhost:56838/UpToDateFile3 http://localhost:56838/NewerFile http://localhost:56838/UpdatedFile 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '-N', 'http://localhost:56838/UpToDateFile1', 'http://localhost:56838/UpToDateFile2', 'http://localhost:56838/UpToDateFile3', 'http://localhost:56838/NewerFile', 'http://localhost:56838/UpdatedFile']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-condget.py-test'}
Conditional get falling to head
Conditional get falling to head
Conditional get falling to head
Conditional get falling to head
{'GET /UpToDateFile2', 'HEAD /UpdatedFile', 'HEAD /NewerFile', 'HEAD /UpToDateFile2', 'GET /UpToDateFile1', 'HEAD /UpToDateFile1', 'GET /UpToDateFile3', 'HEAD /UpToDateFile3', 'GET /NewerFile'}
Error: Not all files were crawled correctly..

```

### stderr
```
127.0.0.1 - - [15/Nov/2025 13:44:44] "HEAD /UpToDateFile1 HTTP/1.1" 304 -
127.0.0.1 - - [15/Nov/2025 13:44:44] "HEAD /UpToDateFile2 HTTP/1.1" 304 -
127.0.0.1 - - [15/Nov/2025 13:44:44] "HEAD /UpToDateFile3 HTTP/1.1" 304 -
127.0.0.1 - - [15/Nov/2025 13:44:44] "HEAD /NewerFile HTTP/1.1" 304 -
127.0.0.1 - - [15/Nov/2025 13:44:44] "HEAD /UpdatedFile HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:44:44] "GET /UpdatedFile HTTP/1.1" 200 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-condget.py", line 136, in <module>
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
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/conf/files_crawled.py", line 27, in __call__
    raise TestFailed('Not all files were crawled correctly.')
exc.test_failed.TestFailed: Not all files were crawled correctly.

```

## Analysis


**Issue Type**: File crawling mismatch

The test expects certain files to be downloaded/crawled, but wget-faster either:
- Downloaded different files
- Missed some files
- Downloaded extra files

**Possible causes**:
1. Link extraction logic difference
2. Recursive download filtering issue
3. Spider mode behavior difference
4. robots.txt handling difference

**Next steps**:
1. Check which files were expected vs actual
2. Review link extraction in recursive.rs
3. Compare with GNU wget behavior

## Implementation Notes

_Add implementation notes here after investigation_

## Related Tests

_List related tests that might have similar issues_

## References

_Add links to relevant code sections or documentation_

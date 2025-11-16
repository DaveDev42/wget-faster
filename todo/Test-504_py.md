# Test-504.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 1.60s

## Description

This test ensures that Wget handles a 504 Gateway Timeout response correctly.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
--2025-11-16 12:50:16--  http://localhost:51535/File1
Resolving localhost... 
Connecting to localhost:51535... connected.
HTTP request sent, awaiting response... 
Saving to: 'File1'

--2025-11-16 12:50:17--  http://localhost:51535/File1
Resolving localhost... 
Connecting to localhost:51535... connected.
HTTP request sent, awaiting response... 
Saving to: 'File1'

--2025-11-16 12:50:17--  http://localhost:51535/File2
Resolving localhost... 
Connecting to localhost:51535... connected.
HTTP request sent, awaiting response... 
Saving to: 'File2'

200 OK
Length: 29 (29B) [text/plain]

2025-11-16 12:50:17 - 'File2' saved [29]

Running Test Test-504.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --tries=2 http://localhost:51535/File1 http://localhost:51535/File2 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--tries=2', 'http://localhost:51535/File1', 'http://localhost:51535/File2']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-504.py-test'}
Custom Response code sent.
Custom Response code sent.
{'HEAD /File1', 'GET /File1', 'HEAD /File2'}
Error: Not all files were crawled correctly..

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 12:50:16] "HEAD /File1 HTTP/1.1" 504 -
wget-faster: download failed: Invalid response status: 504
wgetf: retrying in 1 seconds... (attempt 1/2)
127.0.0.1 - - [16/Nov/2025 12:50:17] "HEAD /File1 HTTP/1.1" 504 -
wget-faster: download failed: Invalid response status: 504
wgetf: Invalid response status: 504
127.0.0.1 - - [16/Nov/2025 12:50:17] "HEAD /File2 HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 12:50:17] "GET /File2 HTTP/1.1" 200 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-504.py", line 68, in <module>
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

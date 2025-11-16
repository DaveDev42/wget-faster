# Test-recursive-basic.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.09s

## Description

Basic test of --recursive.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test-recursive-basic.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --recursive --no-host-directories http://localhost:58830/a/File1.html 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--recursive', '--no-host-directories', 'http://localhost:58830/a/File1.html']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-recursive-basic.py-test'}
{'HEAD /a/File2.html', 'HEAD /a/File1.html', 'HEAD /robots.txt', 'HEAD /b/File3.html'}
Error: Not all files were crawled correctly..

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 13:53:38] "HEAD /a/File1.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:38] "GET /a/File1.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:38] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:53:38] "HEAD /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:53:38] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:53:38] "GET /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:53:38] "HEAD /a/File2.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:38] "GET /a/File2.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:38] "HEAD /b/File3.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:53:38] "GET /b/File3.html HTTP/1.1" 200 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-recursive-basic.py", line 57, in <module>
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

# Test--spider-r.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.61s

## Description

This test executed Wget in Spider mode with recursive retrieval.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test--spider-r.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --spider -r http://localhost:59573/ 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--spider', '-r', 'http://localhost:59573/']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test--spider-r.py-test'}
{'GET /nonexistent', 'GET /dummy.txt', 'HEAD /robots.txt', 'GET /againnonexistent'}
Error: Not all files were crawled correctly..

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD / HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET / HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD / HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET / HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /secondpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /secondpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /secondpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /secondpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /nonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /nonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /nonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /nonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /nonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /thirdpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /thirdpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /thirdpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /thirdpage.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /dummy.txt HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /dummy.txt HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /dummy.txt HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /againnonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /againnonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /againnonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "HEAD /againnonexistent HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:57:04] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:57:04] "GET /againnonexistent HTTP/1.1" 404 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test--spider-r.py", line 102, in <module>
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

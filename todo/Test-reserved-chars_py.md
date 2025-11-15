# Test-reserved-chars.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.63s

## Description

This test ensures that Wget keeps reserved characters in URLs in non-UTF-8 charsets.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test-reserved-chars.py
/Users/dave/.cargo/bin/wgetf --debug --no-config  --spider -r http://localhost:57049/base.html 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--spider', '-r', 'http://localhost:57049/base.html']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-reserved-chars.py-test'}
{'HEAD /robots.txt'}
Error: Not all files were crawled correctly..

```

### stderr
```
127.0.0.1 - - [15/Nov/2025 13:45:08] "HEAD /base.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "GET /base.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "HEAD /base.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "HEAD /base.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "GET /base.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] code 404, message Not Found
127.0.0.1 - - [15/Nov/2025 13:45:08] "HEAD /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [15/Nov/2025 13:45:08] code 404, message Not Found
127.0.0.1 - - [15/Nov/2025 13:45:08] "GET /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "HEAD /a%2Bb.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "GET /a%2Bb.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "HEAD /a%2Bb.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "HEAD /a%2Bb.html HTTP/1.1" 200 -
127.0.0.1 - - [15/Nov/2025 13:45:08] "GET /a%2Bb.html HTTP/1.1" 200 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-reserved-chars.py", line 55, in <module>
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

# Test-recursive-pathmax.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: test_framework_error
**Execution Time**: 0.10s

## Description

Basic test of --recursive.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test-recursive-pathmax.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --recursive --no-host-directories http://localhost:56116/ 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--recursive', '--no-host-directories', 'http://localhost:56116/']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-recursive-pathmax.py-test'}
Error: Expected file deeper/than/zovgafsyfobzxpxaaivibdagicxnddsbcuigsmxylvttklwakytrqajbxdyihvzvmqhpvxhdabjfctqdrftwahgcsdndmmpmrfekcdwrxoqpkweluqvvzfdugkfysnpgfntpahixesxecseortvxzgftxftenlybrubhqygcjtlzxpjogkpckmivboyupqzlmqgchthtyxdqrwfmcjvxyswehrpngxhddsrdrrjkinpy not found..

```

### stderr
```
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD / HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET / HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:37:18] code 404, message Not Found
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /robots.txt HTTP/1.1" 404 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /a/File2.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /a/File2.html HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /deeper/than/sanitryyybkbgjmxnyfjwnaxudkpcwemjmdkvgfdhalibxntasaettcmllgaqqnsigroisidyrosnpmoxuvbbihgmxejvuyoeyhvakysjxnexxszoolqhywueitzewvmzcobjmkbyglsfncckzyzdtpmngxiugfuwugwbhlxqbkopobjzivrfdvjmklnmnotdmqodyobspwlsdhkzwuifdchljfvxuuidrppeilnakos HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /deeper/than/sanitryyybkbgjmxnyfjwnaxudkpcwemjmdkvgfdhalibxntasaettcmllgaqqnsigroisidyrosnpmoxuvbbihgmxejvuyoeyhvakysjxnexxszoolqhywueitzewvmzcobjmkbyglsfncckzyzdtpmngxiugfuwugwbhlxqbkopobjzivrfdvjmklnmnotdmqodyobspwlsdhkzwuifdchljfvxuuidrppeilnakos HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "HEAD /deeper/than/zovgafsyfobzxpxaaivibdagicxnddsbcuigsmxylvttklwakytrqajbxdyihvzvmqhpvxhdabjfctqdrftwahgcsdndmmpmrfekcdwrxoqpkweluqvvzfdugkfysnpgfntpahixesxecseortvxzgftxftenlybrubhqygcjtlzxpjogkpckmivboyupqzlmqgchthtyxdqrwfmcjvxyswehrpngxhddsrdrrjkinpyk HTTP/1.1" 200 -
127.0.0.1 - - [16/Nov/2025 13:37:18] "GET /deeper/than/zovgafsyfobzxpxaaivibdagicxnddsbcuigsmxylvttklwakytrqajbxdyihvzvmqhpvxhdabjfctqdrftwahgcsdndmmpmrfekcdwrxoqpkweluqvvzfdugkfysnpgfntpahixesxecseortvxzgftxftenlybrubhqygcjtlzxpjogkpckmivboyupqzlmqgchthtyxdqrwfmcjvxyswehrpngxhddsrdrrjkinpyk HTTP/1.1" 200 -
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-recursive-pathmax.py", line 77, in <module>
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
exc.test_failed.TestFailed: Expected file deeper/than/zovgafsyfobzxpxaaivibdagicxnddsbcuigsmxylvttklwakytrqajbxdyihvzvmqhpvxhdabjfctqdrftwahgcsdndmmpmrfekcdwrxoqpkweluqvvzfdugkfysnpgfntpahixesxecseortvxzgftxftenlybrubhqygcjtlzxpjogkpckmivboyupqzlmqgchthtyxdqrwfmcjvxyswehrpngxhddsrdrrjkinpy not found.

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

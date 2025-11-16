# Test-metalink-http-xml-type-trust-content.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: missing_feature
**Execution Time**: 0.59s

## Description

This is to test Metalink/HTTP with Content-Type Metalink/XML. With --trust-server-names, trust the metalink:file names.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
Running Test Test-metalink-http-xml-type-trust-content.py
Running Test Test-metalink-http-xml-type-trust-content.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --trust-server-names --metalink-over-http --content-disposition --metalink-index=0 http://localhost:58753/main.metalink 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--trust-server-names', '--metalink-over-http', '--content-disposition', '--metalink-index=0', 'http://localhost:58753/main.metalink']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-metalink-http-xml-type-trust-content.py-test'}
Error: Expected file newname.metalink not found..

```

### stderr
```
error: unexpected argument '--metalink-over-http' found

  tip: a similar argument exists: '--relative'

Usage: wgetf --debug... --no-config --trust-server-names --relative [URL]...

For more information, try '--help'.
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-metalink-http-xml-type-trust-content.py", line 219, in <module>
    err = http_test.begin ()
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
exc.test_failed.TestFailed: Expected file newname.metalink not found.

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

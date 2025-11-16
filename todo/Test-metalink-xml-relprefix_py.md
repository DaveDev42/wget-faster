# Test-metalink-xml-relprefix.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: missing_feature
**Execution Time**: 0.59s

## Description

This is to test Metalink/XML relative directory prefix support in Wget. With --trust-server-names, trust the metalink:file names.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```
<?xml version="1.0" encoding="utf-8"?>
<metalink version="3.0" xmlns="http://www.metalinker.org/">
  <publisher>
    <name>GNU Wget</name>
  </publisher>
  <license>
    <name>GNU GPL</name>
    <url>http://www.gnu.org/licenses/gpl.html</url>
  </license>
  <identity>Wget Test Files</identity>
  <version>1.2.3</version>
  <description>Wget Test Files description</description>
  <files>
    <file name="subdir/File1">
      <verification>
        <hash type="sha256">8f4a392172bca148756f6c4d217e5f80714921314eee803df194ee688277a792</hash>
      </verification>
      <resources>
        <url type="http" preference="35">http://{{SRV_HOST}}:{{SRV_PORT}}/wrong_file</url>
        <url type="http" preference="40">http://{{SRV_HOST}}:{{SRV_PORT}}/404</url>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File1_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File1</url>
      </resources>
    </file>
    <file name="/subdir/File2">
      <verification>
        <hash type="sha256">fac79f893bd735c6b6b2bd5ca9cf49df6145f65a64de55dfc902d98453ab4272</hash>
      </verification>
      <resources>
        <url type="http" preference="35">http://{{SRV_HOST}}:{{SRV_PORT}}/wrong_file</url>
        <url type="http" preference="40">http://{{SRV_HOST}}:{{SRV_PORT}}/404</url>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File2_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File2</url>
      </resources>
    </file>
    <file name="~/subdir/File3">
      <verification>
        <hash type="sha256">4c3dc628b80b607d747ae190aa7783298103cc4c7fa25d72305062469c3d8916</hash>
      </verification>
      <resources>
        <url type="http" preference="35">http://{{SRV_HOST}}:{{SRV_PORT}}/wrong_file</url>
        <url type="http" preference="40">http://{{SRV_HOST}}:{{SRV_PORT}}/404</url>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File3_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File3</url>
      </resources>
    </file>
    <file name="../subdir/File4">
      <verification>
        <hash type="sha256">13d1363f17ec2a5f3000b04a5f328cd0d0be1d5e4f234fd19999ac656ae62314</hash>
      </verification>
      <resources>
        <url type="http" preference="35">http://{{SRV_HOST}}:{{SRV_PORT}}/wrong_file</url>
        <url type="http" preference="40">http://{{SRV_HOST}}:{{SRV_PORT}}/404</url>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File4_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File4</url>
      </resources>
    </file>
    <file name="subdir/File5">
      <verification>
        <hash type="sha256">55098aa7bff3458da8df2892abb80de2ab8e185009f26ad377b73a95e0fc28df</hash>
      </verification>
      <resources>
        <url type="http" preference="35">http://{{SRV_HOST}}:{{SRV_PORT}}/wrong_file</url>
        <url type="http" preference="40">http://{{SRV_HOST}}:{{SRV_PORT}}/404</url>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File5_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File5</url>
      </resources>
    </file>
  </files>
</metalink>

LocalFiles = ['test.metalink']
ServerFiles = [['wrong_file', 'File1_lowPref', 'File1', 'File2_lowPref', 'File2', 'File3_lowPref', 'File3', 'File4_lowPref', 'File4', 'File5_lowPref', 'File5']]
ExpectedFiles = ['test.metalink.#1', 'test.metalink.#2', 'test.metalink']
Running Test Test-metalink-xml-relprefix.py
Running Test Test-metalink-xml-relprefix.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --directory-prefix ../dir --input-metalink test.metalink 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--directory-prefix', '../dir', '--input-metalink', 'test.metalink']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-metalink-xml-relprefix.py-test'}
Error: Expected file test.metalink.#1 not found..

```

### stderr
```
error: unexpected argument '--input-metalink' found

  tip: a similar argument exists: '--input-file'

Usage: wgetf --debug... --no-config --directory-prefix <PREFIX> --input-file <FILE> [URL]...

For more information, try '--help'.
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-metalink-xml-relprefix.py", line 96, in <module>
    err = Meta.http_test (
        "--directory-prefix ../dir " + \
        "--input-metalink " + XmlName, 0
    )
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/misc/metalinkv3_xml.py", line 111, in http_test
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
exc.test_failed.TestFailed: Expected file test.metalink.#1 not found.

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

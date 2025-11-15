# Test-metalink-xml-continue.py

**Test Type**: python
**Status**: ❌ FAILED
**Category**: missing_feature
**Execution Time**: 0.59s

## Description

This is to test Metalink/XML --continue support in Wget. With --trust-server-names, trust the metalink:file names.

## Error Details

**Error Message**: Exit code: 1

**Exit Code**: 1

## Test Output

### stdout
```

LocalFiles = ['test.metalink.#1', 'test.metalink.#2', 'test.metalink.#3', 'test.metalink.#4']
ServerFiles = [[]]
ExpectedFiles = ['test.metalink.#2', 'test.metalink.#3', 'test.metalink.#4']
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
    <file name="File1">
      <verification>
        <hash type="sha256">8f4a392172bca148756f6c4d217e5f80714921314eee803df194ee688277a792</hash>
      </verification>
      <resources>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File1_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File1</url>
      </resources>
    </file>
    <file name="File2">
      <verification>
        <hash type="sha256">d9fab4ef975b0b403696047926dacaad9fddaf39c5f37c8c64cc841823d2ef39</hash>
      </verification>
      <resources>
        <url type="http" preference="35">http://{{SRV_HOST}}:{{SRV_PORT}}/wrong_file</url>
        <url type="http" preference="40">http://{{SRV_HOST}}:{{SRV_PORT}}/404</url>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File2_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File2</url>
      </resources>
    </file>
    <file name="File3">
      <verification>
        <hash type="sha256">4c3dc628b80b607d747ae190aa7783298103cc4c7fa25d72305062469c3d8916</hash>
      </verification>
      <resources>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File3_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File3</url>
      </resources>
    </file>
    <file name="File4">
      <verification>
        <hash type="sha256">e1a09365a1f71702eae28bf5c48af8f9c0044717d922960917f53b004d277d41</hash>
      </verification>
      <resources>
        <url type="http" preference="35">http://{{SRV_HOST}}:{{SRV_PORT}}/wrong_file</url>
      </resources>
    </file>
    <file name="File5">
      <verification>
        <hash type="sha256">55098aa7bff3458da8df2892abb80de2ab8e185009f26ad377b73a95e0fc28df</hash>
      </verification>
      <resources>
        <url type="http" preference="25">http://{{SRV_HOST}}:{{SRV_PORT}}/File5_lowPref</url>
        <url type="http" preference="30">http://{{SRV_HOST}}:{{SRV_PORT}}/File5</url>
      </resources>
    </file>
  </files>
</metalink>

LocalFiles = ['test.metalink.#1', 'test.metalink.#2', 'test.metalink.#3', 'test.metalink.#4', 'test.metalink']
ServerFiles = [['File1_lowPref', 'File1', 'wrong_file', 'File2_lowPref', 'File2', 'File3_lowPref', 'File3', 'File5_lowPref', 'File5']]
ExpectedFiles = ['test.metalink.#2', 'test.metalink.#3', 'test.metalink.#4', 'test.metalink.#1', 'test.metalink.#5', 'test.metalink']
Running Test Test-metalink-xml-continue.py
Running Test Test-metalink-xml-continue.py
/Users/dave/.cargo/bin/wgetf --debug --no-config --continue --input-metalink test.metalink 
['/Users/dave/.cargo/bin/wgetf', '--debug', '--no-config', '--continue', '--input-metalink', 'test.metalink']
{'HOME': '/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-metalink-xml-continue.py-test'}
Error: Contents of test.metalink.#1 do not match.

```

### stderr
```
error: unexpected argument '--input-metalink' found

  tip: a similar argument exists: '--input-file'

Usage: wgetf --debug... --no-config --continue-download --input-file <FILE> [URL]...

For more information, try '--help'.
--- Actual
+++ Expected
@@ -1 +1 @@
-Would you like+Would you like some Tea?
Traceback (most recent call last):
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/Test-metalink-xml-continue.py", line 102, in <module>
    err = Meta.http_test (
        "--continue " + \
        "--input-metalink " + XmlName, 1
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
  File "/Users/dave/Projects/github.com/wget-faster-test/wget-repo/testenv/conf/expected_files.py", line 64, in __call__
    raise TestFailed("Contents of %s do not match" % file.name)
exc.test_failed.TestFailed: Contents of test.metalink.#1 do not match

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

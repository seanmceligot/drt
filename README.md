```console
$ drt --help
Usage: drt [options]

Options:
    -D, --debug         debug logging
    -i, --interactive   ask before overwrite
    -a, --active        overwrite without asking
    -h, --help          print this help menu
$ rm -vf template/out.config
removed 'template/out.config'
$ echo key=@@value@@ > template/test.config
$ drt v value first_value t template/test.config template/out.config
WOULD: create  template/out.config
$ drt --active v value first_value t template/test.config template/out.config
LIVE: create  template/out.config
$ drt v value new_value t template/test.config template/out.config
WOULD: modify template/out.config
```
```diff
1c1
< key=new_value
---
> key=first_value
```
```console
$ drt --interactive v value new_value t template/test.config template/out.config
files don't match: /tmp/.tmp1HfpM6 template/out.config (o)verwrite / (m)erge[vimdiff] / (c)ontinue / (d)iff / merge to (t)emplate
o
LIVE: create  template/out.config
$ drt x chmod 600 template/out.config
WOULD: run  chmod 600 template/out.config
$ drt --interactive x chmod 600 template/out.config
run (y/n): chmod 600 template/out.config
n
WOULD: run  chmod 600 template/out.config
$ drt --active x chmod 600 template/out.config
LIVE: run  chmod 600 template/out.config
status code: 0
```

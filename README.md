
## Project goals

- makes not changes by default
- default mode just reports what would be done
- interactive mode shows what would be done then ask before making the change
- live mode make changes. it should be run after reveiewing default mode
- print what would be run and try to verify if it would succeed by checking file permissions
- tempalte and key/vauleu pairs for creating files 

```console
$ met --help
Usage: met [options]

Options:
    -D, --debug         debug logging
    -i, --interactive   ask before overwrite
    -a, --active        overwrite without asking
    -h, --help          print this help menu

$ echo key=@@value@@ > template/test.config
$ met v value first_value t template/test.config template/out.config
WOULD: create  template/out.config
$ met --active v value first_value t template/test.config template/out.config
LIVE: create  template/out.config
$ met v value new_value t template/test.config template/out.config
WOULD: modify template/out.config
```
```diff
1c1
< key=new_value
---
> key=first_value
```
```console
$ met --interactive v value new_value t template/test.config template/out.config
files don't match: /tmp/.tmp1HfpM6 template/out.config (o)verwrite / (m)erge[vimdiff] / (c)ontinue / (d)iff / merge to (t)emplate
o
LIVE: create  template/out.config
$ met x chmod 600 template/out.config
WOULD: run  chmod 600 template/out.config
$ met --interactive x chmod 600 template/out.config
run (y/n): chmod 600 template/out.config
n
WOULD: run  chmod 600 template/out.config
$ met --active x chmod 600 template/out.config
LIVE: run  chmod 600 template/out.config
status code: 0
```

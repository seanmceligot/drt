
drt_local=cargo run --bin drt --
drt_local=RUST_BACKTRACE=full cargo run --bin drt -- --debug
drt_installed=drt
drt=${drt_local}
default: test

fix:
	cargo fix

t_mkdir:
	$(drt) t drt.sh /tmp/deleteme

errs: err_no_command err_notset er_invalid_command err_novar err_noval err_t_deny

err_no_command: 
	$(drt) -- x lls -l || true
err_notset:
	$(drt) --active v no_value fake_value t template/test.config template/out.config||true
er_invalid_command:
	${drt} foo ||true
err_novar:
	${drt} v||true
err_noval:
	${drt} v x||true
err_t_deny_mkdir:
	$(drt) t drt.sh /root/foo/deleteme || true

passive:
	$(drt) --active v value fake_value t template/test.config template/out.config
	$(drt) v value real_value t template/test.config template/out.config
active:
	$(drt) --active v value fake_value t template/test.config template/out.config
	$(drt) --active v value real_value t template/test.config template/out.config
interactive:
	$(drt) --active v value fake_value t template/test.config template/out.config
	$(drt) --interactive v value real_value t template/test.config template/out.config

x:
	$(drt) --active v value fake_value t template/test.config template/out.config
	$(drt) x chmod 600 template/out.config
x_active:
	$(drt) --active v value fake_value t template/test.config template/out.config
	$(drt) --active x chmod 600 template/out.config

active_env: DRT_ACTIVE=1
active_env:
	$(drt) "--" x ls -l $(MAKE)

x_interactive:
	$(drt) --active v value fake_value t template/test.config template/out.config
	$(drt) --interactive x chmod 600 template/out.config
xvar:
	$(drt) v f template/out.config x chmod 600 @@f@@
create:
	rm -vf template/out.config
	$(MAKE) active	

test:  lint
	RUST_BACKTRACE=1 cargo test

help:
	$(drt) --help

lint:
	cargo clippy

format: 
	cargo fmt

verbose:
	cargo build --verbose

build:
	cargo check 
	cargo build

noargs: 
	echo cp out1/myconfig project/myconfig
	$(drt) --debug

cmd:
	echo drt v mode=600 if Makefile of /tmp/ cp %%if%% %%of%%

clean:
	cargo clean
	rm -rvf out

update:
	cargo build

install:
	cargo install

d:
	./demo.sh

interactive: interactive x_interactive
tests: passive active x x_active active_env xvar cmd 

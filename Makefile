
met_local=cargo run --bin met --
met_local=RUST_BACKTRACE=full cargo run --bin met -- --debug
met_installed=met
met=${met_local}
default: test

fix:
	cargo fix

t_mkdir:
	$(met) t met.sh /tmp/deleteme

errs: err_no_command err_notset er_invalid_command err_novar err_noval err_t_deny

err_no_command: 
	$(met) -- x lls -l || true
err_notset:
	$(met) --active v no_value fake_value t template/test.config template/out.config||true
er_invalid_command:
	${met} foo ||true
err_novar:
	${met} v||true
err_noval:
	${met} v x||true
err_t_deny_mkdir:
	$(met) t met.sh /root/foo/deleteme || true

f:
	$(met) v key1 val1 f template/test.config template/upper.out /usr/bin/tr 'a-z' 'A-Z'
	$(met) --active v key1 val1 f template/test.config template/upper.out /usr/bin/tr 'a-z' 'A-Z'
passive:
	$(met) --active v value fake_value t template/test.config template/out.config
	$(met) v value real_value t template/test.config template/out.config
active:
	$(met) --active v value fake_value t template/test.config template/out.config
	$(met) --active v value real_value t template/test.config template/out.config
interactive:
	$(met) --active v value fake_value t template/test.config template/out.config
	$(met) --interactive v value real_value t template/test.config template/out.config

x:
	$(met) --active v value fake_value t template/test.config template/out.config
	$(met) x chmod 600 template/out.config
x_active:
	$(met) --active v value fake_value t template/test.config template/out.config
	$(met) --active x chmod 600 template/out.config

active_env: MET_ACTIVE=1
active_env:
	$(met) "--" x ls -l $(MAKE)

x_interactive:
	$(met) --active v value fake_value t template/test.config template/out.config
	$(met) --interactive x chmod 600 template/out.config
xvar:
	$(met) v f template/out.config x chmod 600 @@f@@
create:
	rm -vf template/out.config
	$(MAKE) active	

test:  lint
	RUST_BACKTRACE=1 cargo test

help:
	$(met) --help

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
	$(met) --debug

cmd:
	echo met v mode=600 if Makefile of /tmp/ cp %%if%% %%of%%

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

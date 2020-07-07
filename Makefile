
drt_local=RUST_BACKTRACE=1 cargo run --bin drt -- --debug
drt=cargo run --bin drt --
drt_installed=drt
drt=${drt_installed}
default: test

er:
	${drt_local} foo
	${drt_local} v
	${drt_local} v x
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

slapd:
	${drt_local} t openldap/slapd.conf /tmp/slapd.conf

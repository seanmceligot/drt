
drt=RUST_BACKTRACE=1 cargo run --bin drt -- --debug
#drt=cargo run --bin drt --

default: test

passive:
	$(drt) -a v value fake_value t template/test.config template/out.config
	$(drt) v value real_value t template/test.config template/out.config
active:
	$(drt) -a v value fake_value t template/test.config template/out.config
	$(drt) -a v value real_value t template/test.config template/out.config
interactive:
	$(drt) -a v value fake_value t template/test.config template/out.config
	$(drt) -i v value real_value t template/test.config template/out.config

x:
	$(drt) -a v value fake_value t template/test.config template/out.config
	$(drt) x chmod 600 template/out.config
x_active:
	$(drt) -a v value fake_value t template/test.config template/out.config
	$(drt) -a x chmod 600 template/out.config
x_interactive:
	$(drt) -a v value fake_value t template/test.config template/out.config
	$(drt) -i x chmod 600 template/out.config
xvar:
	$(drt) v f template/out.config x chmod 600 @@f@@
create:
	rm -vf template/out.config
	$(MAKE) active	
test: 
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

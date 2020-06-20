
drt=RUST_BACKTRACE=1 cargo run --bin drt -- --debug

default: test

passive:
	$(drt) v value real_value t template/test.config template/out.config
active:
	$(drt) -a v value real_value t template/test.config template/out.config
passive:
	$(drt) -a v value fake_value t template/test.config template/out.config
	$(drt) -i v value real_value t template/test.config template/out.config

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

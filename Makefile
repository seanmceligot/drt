
default: test

test: 
	RUST_BACKTRACE=1 cargo test

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
	RUST_BACKTRACE=1 cargo run --bin drt -- --debug

rund: 
	echo cp out1/myconfig project/myconfig
	RUST_BACKTRACE=1 cargo run --bin drt -- --debug v base.dir=base_dir v test=1 v y=hello v user=myuser of out1/my.config t project/my.config
run: 
	echo cp out1/myconfig project/myconfig
	RUST_BACKTRACE=1 cargo run --bin drt -- v base.dir=base_dir v test=1 v y=hello v user=myuser of out1/my.config t project/my.config

i: 
	echo cp out1/myconfig project/myconfig
	RUST_BACKTRACE=1 cargo run --bin drt -- --interactive v base.dir=base_dir v test=1 v y=hello v user=myuser of out1/my.config t project/my.config

a: 
	echo cp out1/myconfig project/myconfig
	RUST_BACKTRACE=1 cargo run --bin drt -- --active v base.dir=base_dir v test=1 v y=hello v user=myuser of out1/my.config t project/my.config
	
clean:
	cargo clean
	rm -rvf out

update:
	cargo build

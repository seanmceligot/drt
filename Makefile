
default: build

test: build run

verbose:
	cargo build --verbose

build:
	cargo check
	cargo build

run: 
	echo cp out1/myconfig project/myconfig
	cargo run --bin drt -- v n=1 v y=hello of out1/my.config t project/my.config
	#cargo run --bin drt -- v:n=1 v:y=hello  mkdir of out1/my.config mkdir t project/my.config

r: 
	RUST_BACKTRACE=1 cargo run --bin drtrun -- v:n=1 v:y=hello t:project/my.config of:f:mkdir,out1/my.config
	
clean:
	cargo clean
	rm -rvf out

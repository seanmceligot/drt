
default: build

test: build run

verbose:
	cargo build --verbose

build:
	cargo build

run: 
	RUST_BACKTRACE=1 ./target/debug/drt -p system.config "project/*"

r: 
	RUST_BACKTRACE=1 ./target/debug/drtrun v:n=1 v:y=hello t:project/my.config of:f:mkdir,out1/my.config
	
clean:
	cargo clean
	rm -rvf out

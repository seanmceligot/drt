
default: build

test: build run

verbose:
	cargo build --verbose

build:
	cargo build

run: 
	RUST_BACKTRACE=1 ./target/debug/drt -p system.config "project/*"

clean:
	cargo clean
	rm -rvf out
	


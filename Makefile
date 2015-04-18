test: build run

verbose:
	cargo build --verbose

build:
	cargo build

run:
	RUST_BACKTRACE=1 ./target/debug/confsync -p system.config "project/*"

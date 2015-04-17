test: build run

build:
	cargo build

run:
	RUST_BACKTRACE=1 ./target/debug/configsync -p system.config "project/*"

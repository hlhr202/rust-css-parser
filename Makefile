all:
	cargo build --release
run:
	cargo run ${PWD}/test/test.less
clean:
	cargo clean
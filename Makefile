all:
	cargo build --release
cargo-test:
	cargo test -- --nocapture
wasm:
	wasm-pack build --target nodejs
run:
	cargo run ${PWD}/test/test.less
clean:
	cargo clean
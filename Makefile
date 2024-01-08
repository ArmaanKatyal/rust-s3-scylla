run: build
	RUST_LOG=info ./target/debug/rust-s3-scylla

build:
	cargo build

format:
	cargo fmt

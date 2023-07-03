.PHONY: test

target/debug/chunkdrive: src/main.rs
	cargo build

test: target/debug/chunkdrive
	./$<
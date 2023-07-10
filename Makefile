.ONESHELL:  # so I can use cd
.PHONY: run run_shell test

style.css: $(wildcard style/**)
	cd style
	pnpm run build

./target/debug/chunkdrive: $(wildcard src/**)
	cargo build

run: ./target/debug/chunkdrive style.css
	./target/debug/chunkdrive

run_shell: ./target/debug/chunkdrive
	./target/debug/chunkdrive --shell

test: $(wildcard src/**)
	cargo test
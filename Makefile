.ONESHELL:  # so I can use cd
.PHONY: run run_shell test style.css ./target/debug/chunkdrive

style.css:
	cd style
	pnpm run build

./target/debug/chunkdrive:
	cargo build

run: ./target/debug/chunkdrive style.css
	./target/debug/chunkdrive

run_shell: ./target/debug/chunkdrive
	./target/debug/chunkdrive --shell

test: $(wildcard src/**)
	cargo test
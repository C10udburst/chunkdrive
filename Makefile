.ONESHELL:  # so I can use cd
.PHONY: run run_shell test style.css script.js ./target/debug/chunkdrive web

style.css:
	cd web
	pnpm install --frozen-lockfile
	pnpm run build-style

script.js:
	cd web
	pnpm install --frozen-lockfile
	pnpm run build-script

web: style.css script.js

./target/debug/chunkdrive:
	cargo build

run: ./target/debug/chunkdrive web
	./target/debug/chunkdrive

run_shell: ./target/debug/chunkdrive
	./target/debug/chunkdrive --shell

test: $(wildcard src/**)
	cargo test
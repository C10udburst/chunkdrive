.ONESHELL:  # so I can use cd
.PHONY: run run_shell test style.css ./target/debug/chunkdrive docker_shared

style.css:
	cd style
	pnpm run build

./target/debug/chunkdrive:
	cargo build

# the binaries are compiled into the docker container, so we just make the css file
docker_shared: style.css

run: ./target/debug/chunkdrive style.css
	./target/debug/chunkdrive

run_shell: ./target/debug/chunkdrive
	./target/debug/chunkdrive --shell

test: $(wildcard src/**)
	cargo test
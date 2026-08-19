

run:
	cargo run

build:
	cargo build

test:
	cd shelltest && uv run src/main.py

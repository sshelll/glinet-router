.PHONY: build install

build:
	cargo build --release --locked

install:
	cargo install --path . --locked

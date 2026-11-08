.PHONY: build install

build:
	cargo build --release --locked

install:
	cargo install --path . --locked

.PHONY: check-update check-custom-rules
check-update:
	cargo run -- check-update

check-custom-rules:
	cargo run -- check-custom-rules

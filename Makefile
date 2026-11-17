.PHONY: all check build release run test deb clean

all: check

check:
	cargo check

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

test:
	cargo test

deb:
	./scripts/build-deb.sh

clean:
	cargo clean
	rm -rf build-deb *.deb

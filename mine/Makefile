all:
	cargo build

release:
	cargo build --release

step%.rs: all
	@echo "step$*: will use main target"

clean:
	cargo clean

.PHONY: all clean release

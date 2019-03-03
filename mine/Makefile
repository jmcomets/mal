STEPS = step0_repl

all: $(STEPS)

%: %.rs
	cargo build --release --bin $*

clean:
	cargo clean

.PHONY: all clean

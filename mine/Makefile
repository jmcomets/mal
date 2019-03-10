STEPS = step0_repl step1_read_print step2_eval

all: $(STEPS)

%: %.rs
	cargo build --release --bin $*

clean:
	cargo clean

.PHONY: all clean

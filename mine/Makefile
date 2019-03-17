STEPS = step0_repl step1_read_print step2_eval step3_env step4_if_fn_do

all: $(STEPS)

%: %.rs
	cargo build --release --bin $*

clean:
	cargo clean

.PHONY: all clean

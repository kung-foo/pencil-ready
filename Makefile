.PHONY: build release clean clean-output test run

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

# Generate a sample worksheet (debug/dev)
run:
	cargo run --bin mathsheet -- multiply --digits 2,2 --seed 42 --format all --output output/worksheet

.PHONY: clean-all

clean:
	cargo clean

clean-output:
	rm -f output/*.pdf output/*.png output/*.svg output/*.typ

clean-all: clean clean-output

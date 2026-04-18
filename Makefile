.PHONY: build release clean clean-output test run stories-gen stories-diff stories-check stories-approve frontend-build serve-release

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

# Generate a sample worksheet (debug/dev)
run:
	cargo run --bin pencil-ready -- multiply --digits 2,2 --seed 42 --format all --output output/worksheet

.PHONY: clean-all

clean:
	cargo clean

clean-output:
	rm -f output/*.pdf output/*.png output/*.svg output/*.typ

clean-all: clean clean-output

# --- Visual stories ---

stories-gen:
	cargo run -p pencil-ready-stories -- generate

stories-diff:
	cargo run -p pencil-ready-stories -- diff

# regen + diff in one step (fails on any change)
stories-check:
	cargo run -p pencil-ready-stories -- check

stories-approve:
	cargo run -p pencil-ready-stories -- approve

# --- Release bundle + run ---

# Build the React bundle into frontend/dist/.
frontend-build:
	cd frontend && pnpm install --frozen-lockfile && pnpm build

# Bundle the frontend, build the server in release, and run it with the
# bundle wired in — same shape the Docker image uses in production.
serve-release: frontend-build
	cargo build --release --bin pencil-ready-server
	PENCIL_READY_ROOT=. \
	PENCIL_READY_STATIC_DIR=frontend/dist \
	./target/release/pencil-ready-server

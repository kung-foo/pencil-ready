.PHONY: build release clean clean-output test run stories-gen stories-diff stories-check stories-approve react-build astro-build server-release serve-react serve-astro

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

# --- Frontend bundles ---

react-build:
	cd frontend/react && pnpm install --frozen-lockfile && pnpm build

astro-build:
	cd frontend/astro && pnpm install --frozen-lockfile && pnpm build

# --- Prod-shaped local runs ---

# Release binary only; built once and reused across the serve-* targets.
server-release:
	cargo build --release --bin pencil-ready-server

# Build + serve the React SPA on :8080 against the live API.
serve-react: react-build server-release
	./target/release/pencil-ready-server --framework react --port 8080

# Build + serve the Astro pre-rendered site on :8081. Run both targets
# in separate terminals for side-by-side comparison.
serve-astro: astro-build server-release
	./target/release/pencil-ready-server --framework astro --port 8081

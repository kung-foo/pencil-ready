.PHONY: build release clean clean-output test run stories-gen stories-diff stories-check stories-approve frontend-build server-release serve deps-refresh thumbs

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

# --- Homepage thumbnails ---
#
# Compile every frontend/astro/src/assets/thumbs/thumb-<kind>.typ into
# its sibling <kind>.svg. The Astro build inlines those SVGs into the
# homepage card grid (see WorksheetThumb.astro). Run after editing the
# thumb sources or any of the lib/problems/* components they import.
THUMB_DIR := frontend/astro/src/assets/thumbs
THUMB_SOURCES := $(wildcard $(THUMB_DIR)/thumb-*.typ)
THUMB_SVGS := $(patsubst $(THUMB_DIR)/thumb-%.typ,$(THUMB_DIR)/%.svg,$(THUMB_SOURCES))

thumbs: $(THUMB_SVGS)

$(THUMB_DIR)/%.svg: $(THUMB_DIR)/thumb-%.typ
	typst compile --root . --font-path fonts $< $@

# --- Frontend + prod-shaped local run ---

frontend-build:
	cd frontend/astro && pnpm install --frozen-lockfile && pnpm build

server-release:
	cargo build --release --bin pencil-ready-server

# Build the Astro bundle, build the server in release mode, and run it
# serving the bundle + API on :8080 — same shape the Docker image uses.
serve: frontend-build server-release
	./target/release/pencil-ready-server --port 8080

# --- Supply-chain: regenerate Cargo.lock with a 14-day cooldown on new
# crate versions. Mirrors the `minimumReleaseAge` pnpm policy — buys
# time for malicious or compromised releases to be yanked before they
# land in our lockfile.
#
# `cargo-cooldown` runs cargo via a wrapper that filters version
# candidates by publish age (COOLDOWN_MINUTES). Use this instead of
# `cargo update` when bumping dependencies. Fetches the tool on first
# run. 20160 minutes = 14 days.
deps-refresh:
	@command -v cargo-cooldown >/dev/null || cargo install --locked cargo-cooldown
	rm -f Cargo.lock
	COOLDOWN_MINUTES=20160 cargo-cooldown check
	@echo
	@echo "Cargo.lock regenerated. Review the diff before committing:"
	@echo "  git diff Cargo.lock"

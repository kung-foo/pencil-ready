# syntax=docker/dockerfile:1.7

# --- Builder stage ---
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# Build just the server binary. Cargo resolves the workspace from the root
# manifest, so we need the full tree, not just the server crate.
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin pencil-ready-server \
 && cp /app/target/release/pencil-ready-server /usr/local/bin/pencil-ready-server

# --- Runtime stage ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/pencil-ready-server /usr/local/bin/pencil-ready-server

# Runtime assets: typst imports from /lib, loads bundled fonts, and reads
# binary files (rainbow-heart.svg, etc.) from /assets. The server reads
# these via PENCIL_READY_ROOT.
WORKDIR /app
COPY --from=builder /app/lib    /app/lib
COPY --from=builder /app/fonts  /app/fonts
COPY --from=builder /app/assets /app/assets

ENV PORT=8080
ENV PENCIL_READY_ROOT=/app
EXPOSE 8080
CMD ["/usr/local/bin/pencil-ready-server"]

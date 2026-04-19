# syntax=docker/dockerfile:1.7

# --- React SPA bundle stage ---
FROM node:22-slim AS react
WORKDIR /app/frontend/react
RUN corepack enable && corepack prepare pnpm@latest --activate

COPY frontend/react/package.json frontend/react/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY frontend/react/ ./
RUN pnpm build
# Result: /app/frontend/react/dist

# --- Astro bundle stage ---
FROM node:22-slim AS astro
WORKDIR /app/frontend/astro
RUN corepack enable && corepack prepare pnpm@latest --activate

COPY frontend/astro/package.json frontend/astro/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY frontend/astro/ ./
RUN pnpm build
# Result: /app/frontend/astro/dist

# --- Rust builder stage ---
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# utoipa-swagger-ui's build script downloads the Swagger UI bundle via curl
# (or reqwest if the feature is enabled). Install curl so the build works.
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl \
 && rm -rf /var/lib/apt/lists/*

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
# binary files (rainbow-heart.svg, etc.) from /assets. The server resolves
# these relative to --root.
WORKDIR /app
COPY --from=builder /app/lib    /app/lib
COPY --from=builder /app/fonts  /app/fonts
COPY --from=builder /app/assets /app/assets

# Both frontends shipped; --framework at startup picks which one this
# container serves. Running two replicas (one --framework react, one
# --framework astro) lets both live side-by-side.
COPY --from=react /app/frontend/react/dist /app/frontend/react/dist
COPY --from=astro /app/frontend/astro/dist /app/frontend/astro/dist

EXPOSE 8080
# Default to Astro for now (pre-rendered per-type pages). Flip to
# `--framework react` to serve the SPA instead.
CMD ["/usr/local/bin/pencil-ready-server", "--framework", "astro", "--port", "8080"]

# Pencil Ready

Free printable math worksheets. Deployed at [pencilready.com](https://pencilready.com).

## Repo layout

```
crates/
├── core     pencil-ready-core       — worksheet generators + typst template
├── cli      pencil-ready-cli        — `pencil-ready` CLI binary
├── server   pencil-ready-server     — axum HTTP server, OpenAPI/Swagger UI
└── stories  pencil-ready-stories    — visual-regression harness
lib/                                 — typst component library (imported from generated .typ)
fonts/                               — Roboto Slab, Fira Code, Fira Math, STIX Two Text, Crimson Text, Noto Color Emoji
assets/                              — rainbow-heart.svg and other typst-inlined images
frontend/astro/                      — Astro site (static per-route HTML + one React island per worksheet page)
stories/                             — typst story snippets + baselines under stories/baseline/
Dockerfile, fly.toml, Makefile       — deployment + dev
SPEC.md, WORKSHEETS.md               — architecture + worksheet type reference
pencilready-logo-spec.md             — brand mark / pencil underline spec
```

## Running

| What | Command |
|---|---|
| Unit tests | `cargo test --workspace --lib` |
| CLI sample | `cargo run --bin pencil-ready -- multiply --digits 3,2 --seed 42 --output out` |
| Prod-shaped local run (Astro + release server on :8080) | `make serve` |
| Visual-regression check | `make stories-check` |
| Frontend dev server (pair with `cargo run --bin pencil-ready-server` for API) | `cd frontend/astro && pnpm dev` |
| Deploy | `flyctl deploy --remote-only` |

## Conventions

- Kebab-case for typst identifiers (typst convention).
- `em` for layout values inside problem components; `pt` only for
  stroke widths and the `problem-text-size` constants themselves.
  Never `cm` inside a component. (See SPEC.md → Layout rules.)
- When adding a worksheet type: generator in `crates/core/src/<name>.rs`,
  variant in `WorksheetType`, stories under `stories/<name>-*.typ`,
  help content in `frontend/astro/src/lib/worksheet-info.ts`,
  catalog entry in `WORKSHEET_KINDS` in `frontend/astro/src/lib/api.ts`,
  server handler + params struct in `crates/server/src/main.rs`.
- Dedup logic in generators uses `pad_with_duplicates` (in
  `pencil-ready-core`'s `lib.rs`) so narrow constraint spaces still
  fill a page — don't re-introduce "bail on < N unique" guards.
- Frontend is single-framework (Astro). Don't reintroduce a React SPA
  under `frontend/react/` — the Astro site with React islands covers
  both SEO and interactivity.
- PDF metadata (`#set document(...)`) is emitted by
  `crates/core/src/template.rs`. Keep the keyword list tight; noise
  here leaks into filename-indexed searches.
- Brand palette lives at `--color-pencil-no-2` in
  `frontend/astro/src/styles/index.css`. For new accents, add new
  `--color-*` entries to `@theme inline` rather than inlining hex
  values.

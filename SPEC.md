# Pencil Ready — project spec

Background: teaching my daughter 5th-grade math (Norway). I grew up on
repetition-based practice and want the same surface for her. The
ad-clogged worksheet sites aren't usable any more, especially with an
ad blocker. This is the alternative — free, ad-free, crawlable,
deterministic, deployed at [pencilready.com](https://pencilready.com).

## Architecture (current)

### Worksheet generation: Typst

[Typst](https://typst.app/) is the typesetting engine for every
worksheet. Sub-100ms compile, native binary, no VMs or browsers.

- `lib/*.typ` — reusable typst components (header, footer, grid
  layout, per-style problem cells under `lib/problems/*.typ`)
- `fonts/` — the curated font set (Roboto Slab, Fira Mono, Fira Code,
  Fira Math, STIX Two Text, Crimson Text, Noto Color Emoji). The
  server loads these directly and ignores system fonts for
  deterministic output.
- `assets/` — binary assets (`rainbow-heart.svg`) referenced from
  typst via `image(...)`.

### Workspace layout

Cargo workspace at the repo root with four crates under `crates/`:

| Crate | Purpose |
|---|---|
| `pencil-ready-core` | Worksheet generators + typst template rendering |
| `pencil-ready-cli`  | Command-line binary `pencil-ready` |
| `pencil-ready-server` | Axum HTTP server, OpenAPI/Swagger |
| `pencil-ready-stories` | Visual-regression harness |

### Rust HTTP server

`pencil-ready-server` (axum) is the runtime backend.

- `GET /api/worksheets/{kind}?…` — one endpoint per worksheet type;
  query params are typed per kind (utoipa `IntoParams`); response
  body is raw PDF/PNG/SVG bytes with `Content-Type` and
  `Content-Disposition: inline; filename=pencil-ready-<slug>.<ext>`.
- `GET /openapi.json` — OpenAPI spec generated from the typed params.
- `GET /docs` — embedded Swagger UI.
- `--framework`-less static serving: when
  `--static-dir <path>` (default `<root>/frontend/astro/dist`) points
  at an existing `index.html`, the server mounts it with a 200-status
  SPA fallback so deep links work; otherwise runs API-only.
- CORS permissive on all routes (GET-only, no credentials).
- Compression: `tower-http` gzip + brotli.

### Frontend: Astro

[Astro](https://astro.build/) at `frontend/astro/` — static per-route
HTML + a single React island per worksheet page.

- 11 pre-rendered pages at build time: landing, `/about`,
  `/worksheets/<kind>/` for each kind. Each page's full content
  (title, summary, prerequisites, learning goals) sits in the HTML
  source — no JS required for indexability.
- React island (`client:only="react"`) holds the configurator
  (shadcn/ui: Select/Input/Switch/Button/Card/Breadcrumb/
  InputGroup). Type-change navigations go through Astro's
  `ClientRouter` so there's a smooth View-Transitions swap, not a
  full reload.
- Prefetching: `prefetch: { prefetchAll: true, defaultStrategy: "hover" }`.
- Sitemap + robots.txt generated at build time via `@astrojs/sitemap`.

### Output formats

Each worksheet compiles to PDF, PNG, or SVG.

- **PDF** — fonts embedded/subsetted, text selectable, PDF outline
  entries when `--include-answers` is set, `#set document(...)`
  metadata (title, author "Pencil Ready", keywords, subject pointing
  at https://pencilready.com).
- **PNG** — rasterized at 300 PPI.
- **SVG** — glyphs to vector paths, fully self-contained.

Multi-page output (`--pages > 1` or `--include-answers`) requires
PDF. The server rejects the incompatible combinations with a
readable error.

### Deployment

- Dockerfile: `node:22-slim` frontend stage → `rust:1-slim-bookworm`
  builder → `debian:bookworm-slim` runtime. Final image ~56 MB.
- fly.io app `pencil-ready`, region `arn`, custom domain
  `pencilready.com` + `www.pencilready.com` with auto-issued certs.
- Single machine, `auto_stop_machines = "stop"` + `auto_start` so
  idle traffic costs nothing.

## Layout rules

These are the conventions for authoring problem components in `lib/problems/`:

### Units

- **Use `em`, not `cm` or `pt`, for layout values inside problem components.**
  `em` is relative to the current text size, so everything scales proportionally
  when `problem-text-size` changes (enabling a future "big-text" mode).
- **Exceptions:**
  - Stroke widths (`0.8pt`, `1.5pt`) stay in `pt` — stroke thickness is typically
    absolute, independent of text size.
  - `problem-text-size` and other size constants in `shared.typ` stay in `pt`
    since they define the unit itself.
  - Tracking (`problem-tracking`) is in `pt` for fine-grained letter spacing
    control across sizes.
- **Components must not use `cm`.** If you find yourself reaching for `cm`,
  convert to `em` (1em = current font size) or use a text-relative expression
  (e.g. `m.height * 0.5` after measuring rendered text).

### Solve space

Each problem component reserves the writing space needed to solve the problem
in its own bounding box. The component is self-contained — no external padding
required.

- **Vertical problems** (add, subtract, multiply, simple divide):
  - 0.5em **carry space** above the top operand
  - `1.3em × answer-rows` **answer space** below the line
  - `answer-rows` defaults to 1; callers pass higher values when more is needed:
    - Add/subtract/simple-divide: 1 row
    - Multi-digit multiply: `multiplier_digits + 1` rows (one partial product
      per multiplier digit, plus one row for the final sum)
- **Long division**: bracket with overline, with `1em` answer space above the
  overline (for the quotient) and slight overshoot below the text (the curve
  dips past the baseline by the same amount it extends above the text).
- **Horizontal drills**: the answer blank is part of the problem line, so no
  extra solve space is needed.

### Fonts

Constants live in `lib/problems/shared.typ`.

- **Digits**: `Fira Code` (`problem-font`) — monospaced with the
  plain-zero OpenType feature `cv11` enabled so column alignment is
  unambiguous.
- **Operator symbols**: `Fira Math` (`operator-font`) — coverage for
  ×, ÷, ·, :, ± and proper math-spacing metrics.
- **Body / header / footer**: `Roboto Slab` (`body-font`) — slab
  serif that matches the printed-worksheet feel.
- **Algebra variables**: `STIX Two Text` italic — classical LaTeX-
  style variable italic, distinct from the sans digits.
- **Web brand wordmark**: `Crimson Text SemiBold` — display face for
  the "Pencil Ready" header only. Not embedded in PDFs.

### Locale

Locale (`--locale us` / `--locale no`) affects only **horizontal** layouts
(drills), since vertical and bracket notations are universal.

- US: `×` for multiply, `÷` for divide
- Norway: `·` for multiply, `:` for divide

The explicit `--symbol` flag overrides locale for any worksheet.

## Visual regression

See `stories/` and `crates/stories/`. Each story is a `.typ` snippet
that renders a single component in isolation. Baselines are committed
as PNGs under `stories/baseline/`; diffs are computed as directional
RGB images (red = removed, green = added pixels). A change to shared
typst (font, tracking, sizes) automatically invalidates every baseline
— which is the point.

Make targets: `make stories-gen`, `stories-diff`, `stories-check`
(regenerate + diff in one, non-zero exit on change), `stories-approve`
(promote current to baseline).

## Brand

Logo, palette, and the pixel-pencil underline used on the web header
are specced separately in [`pencilready-logo-spec.md`](./pencilready-logo-spec.md).

## TODO

- **Problem box width**: currently computed as
  `max(2.2, digits * 0.55 + 0.6)` cm in `template.rs` — magic numbers
  hand-tuned for the original Cascadia Code at 22pt. Per-cell
  components use `em` correctly; the worksheet-level formula still
  hard-codes `cm`. Worth deriving from measured font metrics the
  next time we touch the template.
- More worksheet types: decimal arithmetic, order of operations,
  negative numbers, area/perimeter, unit conversion.

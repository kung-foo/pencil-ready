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

- Pre-rendered pages at build time, once per locale (see Language
  below): landing, `/about`, `/worksheets/<kind>/` for each kind, plus
  an English-only `/404`. Each page's full content (title, summary,
  prerequisites, learning goals) sits in the HTML source — no JS
  required for indexability.
- React island (`client:only="react"`) holds the configurator
  (shadcn/ui: Select/Input/Switch/Button/Card/Breadcrumb/
  InputGroup). Type-change navigations go through Astro's
  `ClientRouter` so there's a smooth View-Transitions swap, not a
  full reload.
- Prefetching: `prefetch: { prefetchAll: true, defaultStrategy: "hover" }`.
- Sitemap + robots.txt generated at build time via `@astrojs/sitemap`.

### Language

English (default) and Norwegian Bokmål, as separate static pages:
English at the root (`/`, `/about/`, `/worksheets/add/`), Norwegian
under `/no/` with the same English slugs. URL path `no`, language tag
`nb` (`<html lang>`, hreflang, JSON-LD `inLanguage`).

- `frontend/astro/src/i18n/locales.ts` is the single list of locales;
  pages generate every locale through an optional `[...locale]` rest
  param. Nothing assumes exactly two.
- UI copy lives in `src/i18n/ui/<bcp47>.ts`, typed against English and
  key-checked at build time; worksheet help text in
  `src/content/worksheets/<bcp47>/<kind>.md`. A missing translation
  fails the build. The island gets its strings as props.
- Every page carries a self-canonical plus hreflang alternates for
  each locale and `x-default` → English; the sitemap lists the same
  alternates.
- The worksheet API's `locale=us|no` drives the PDF's printed language,
  decimal separator and × / ÷ glyphs; Norwegian pages send `locale=no`.
- "Pencil Ready" and the motto "print → solve → sharpen → repeat" stay
  English in every locale.
- No `Accept-Language` or IP detection. The header's language selector
  links to the same page in the other locale and, on click, sets a
  `lang=<id>` cookie (`Path=/`, one year, `SameSite=Lax`). The server's
  `locale_root_redirect` uses it for exactly one thing: a `GET`/`HEAD`
  of `/` that is an *entry* (`Sec-Fetch-Site` absent, `none` or
  `cross-site`) with `lang=no` gets a 302 to `/no/`. Same-origin
  navigation (logo click, ClientRouter fetch, prefetch) and every other
  path are never redirected. `/` responds `Cache-Control: private,
  no-cache` and `Vary: Cookie, Sec-Fetch-Site` so a cached English root
  can't outlive a switch to Norwegian.
- Unknown paths, `/no/…` included, get the English `404.html`.

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

- Dockerfile: `node:22-slim` frontend stage → `rust:1.99.0-slim-bookworm`
  builder → `debian:bookworm-slim` runtime.
- The Rust version is pinned in **two** places that must move
  together: `rust-toolchain.toml` (local builds) and the Dockerfile
  builder tag (deploys). A floating `rust:1-slim-bookworm` would pick
  up a new compiler the day it ships, which is at odds with the
  14-day cooldown applied to every other dependency.
- fly.io app `pencil-ready`, custom domain `pencilready.com` +
  `www.pencilready.com` with auto-issued certs.
- Two machines — `arn` (primary region) and `sjc` — with
  `auto_stop_machines = "off"` and `min_machines_running = 1`, so at
  least one machine is always warm and first-request latency doesn't
  pay a cold start.

### Analytics: Umami, proxied first-party

Page views go to [Umami](https://umami.is/) Cloud, but never directly
from the browser. The tracker is loaded from our own origin
(`hostUrl: "/umami"` in `Layout.astro`) and the server proxies both
halves:

- `GET /umami/script.js` — fetches `cloud.umami.is/script.js` once and
  caches it in memory for the process lifetime.
- `POST /umami/api/send` — forwards event payloads upstream.

First-party serving means ad blockers and tracker lists don't eat the
script, and no third-party origin sees the visitor directly.

**The `payload.ip` requirement.** Umami Cloud sits behind Cloudflare,
which stamps every inbound request with geo headers (`cf-ipcity`,
`cf-ipcountry`) and `cf-connecting-ip` derived from the *connecting*
IP — our Fly machine, not the visitor. Umami's `getLocation()` checks
those provider headers **before** any forwarded-IP header, so
`X-Forwarded-For` alone is ignored: every session geolocates to the
machine's egress IP. Symptom when this regresses: ~100% of sessions
report one city (it was "Santa Clara", the `sjc` machine).

The fix is `payload.ip` in the JSON body, which Umami treats as
authoritative — it skips the provider headers entirely and runs its own
MaxMind lookup. `handle_umami_send` parses the body and injects it.
Two rules for anyone touching that handler:

- **Overwrite** any client-supplied `ip`; visitors must not be able to
  spoof their location through our proxy.
- **Forward unparseable bodies untouched** rather than dropping them —
  a mislocated event beats a lost one.

The IP also feeds Umami's visitor-identity hash, so this isn't only
cosmetic: with a constant IP, distinct visitors sharing a User-Agent
collapse into a single session and unique-visitor counts undercount.

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
inside its bounding box.

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

### Bounding box

A problem component's outermost bounding rect must satisfy three rules. They
are observable in the visual-regression debug border (`debug: true`).

1. **The bounding box is the solved version's footprint.** Whatever space the
   filled-in version of the problem occupies, the blank (and answer-only)
   versions must occupy the same — so a worksheet grid cell sized to one
   variant fits all of them, and the answer-key page lines up cell-for-cell
   with the problem page.

   To keep the box stable across modes, render solved-only content with
   `hide(...)` in non-solved modes — `hide` reserves the full bounding rect
   but doesn't paint, so measurement stays the same. Don't swap it for empty
   content.

2. **Slot widths are measured, not pinned.** When a component reserves a slot
   for the answer (e.g. a fraction, a number), measure the answer's natural
   width with `context { measure(...) }` and size the slot to that. A pinned
   slot like `width: 2.6em` produces a bounding box wider than the visible
   content, and the slack reads as off-center padding when the cell centers.

3. **The bounding box must include all visible glyphs and strokes.** typst's
   layout often reports a frame smaller than what's actually drawn:
   - `place(...)` content (e.g. the long-division bracket curve) doesn't
     contribute to the parent box's natural size — pin the parent box's
     width/height to include it.
   - Math equations report height by cap-height by default, so superscripts
     and descenders extend past the equation's reported frame. Either pad
     the wrapping box with enough top inset (`0.6em` is the convention in
     `equation-rows.typ`) or render with `top-edge: "bounds"`. Note that
     `top-edge: "bounds"` makes equation rows different heights when one
     has a superscript and another doesn't — and centering by `horizon`
     then misaligns baselines. Prefer the inset.
   - `box(width: slot-w, …)` cells in math equations contribute their full
     box height, but typst still measures the surrounding equation frame
     by cap-height — explicit `height: …` on the wrapping `box` may be
     needed (see `fraction/equivalence.typ`).

4. **No internal padding.** The component returns its tight bounding rect.
   Padding around the cell — for breathing room from neighbors in a
   worksheet grid, or for visual margin in a story — is the caller's job.
   Stories use `#set page(margin: 0.5em)`; the worksheet grid centers each
   cell with `align: center + top` and lets the cell's 1fr width supply the
   spacing.

### Equation-aligned problems

Components built around `=` (algebra one-step / two-step / square-root,
fraction-mult, fraction-simplify) share `lib/problems/_layouts/equation-rows.typ`.
It renders an N-row 3-column grid where:

- **col1 width = col3 width = max(widest LHS, widest RHS)** across all rows.
  Symmetric around `=`, so the equals sign is at the visual horizontal
  center of the bounding rect.
- **col2 = the natural width of `sym.eq`**.

Effects:

- All `=` signs in a problem column-align internally (rows 1, 2, 3 of a
  worked algebra solution all line up).
- When the worksheet grid centers each cell with `align: center + top`,
  the `=` lands at the same x-coordinate in every cell — and since cells
  are uniform 1fr width, `=` signs across all problems in the column line
  up vertically too. No per-page width pre-pass needed.
- The `col-width: auto | length` opt lets a worksheet template pass a
  uniform width across multiple problems if needed; default `auto`
  self-sizes from the problem's own rows.

For solved/blank parity, render unfilled rows' content with `hide(...)`.
The component should pre-build the solved-mode content (so it can be
measured) and conditionally `hide` it for blank/answer-only modes.

### Worksheet grid alignment

`lib/layout.typ`'s `worksheet-grid` lays each problem in a 1fr × 1fr cell
and uses `align: center + top` — horizontal centering for `=`-column
alignment, top alignment so each problem hugs the top of its cell and the
writing space stays below the printed problem.

### Paper independence

A worksheet is the same sheet on every paper: same problems, same page
breaks, same text sizes. Only the whitespace differs.

- Text sizes are fixed (`pt` in `shared.typ`), never scaled to the page.
- Pagination (rows per page) and the column check use
  `common_content_area_cm` — A4's width and US Letter's height, the
  area every paper in `Paper::ALL` holds — not the configured paper.
  The larger paper gets the spare height spread between rows by the
  grid's `1fr` tracks.
- So a seed, a share link or a QR code reproduces the same worksheet
  whatever the printer takes.
- Paper is therefore a per-device preference, not worksheet state: the
  "Paper" section of the header's language menu stores it in
  `localStorage` (`pr.paper`, `lib/paper.ts`; default A4), the
  configurator sends it as `paper=` and shapes the preview to it, and it
  stays out of share URLs — whoever opens a link prints on their own.
- Presets must fit that area on one page. `levels.json` (read by the
  frontend and by the server) holds the level presets, and the server
  tests `levels_fit_one_page_on_every_paper` /
  `defaults_fit_one_page_on_every_paper` render every preset and every
  kind's defaults on each paper, locale and division layout and assert
  one page with identical pagination.

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

Locale (`--locale us` / `--locale no`) sets everything printed that
differs between English and Norwegian worksheets:

- Copy: title, instructions, Name/Date, answer-key title, PDF
  bookmarks and metadata, footer (`crates/core/src/i18n.rs`,
  `meta.rs`). Brand name stays English.
- Symbols: US `×` / `÷`, Norway `·` / `:` — in vertical and horizontal
  layouts alike. Algebra keeps `·` in both.
- Numbers: decimal point vs decimal comma; with a comma, listed values
  (mean data lines) are separated by `; `.
- Remainders: `r=3` vs `rest 3`; the mean answer slot reads `mean =` vs
  `snitt =`.

The explicit `--symbol` flag overrides the operator for any worksheet.

### Division style

How division is laid out is a notation family, `DivisionStyle`, not a
locale: any locale can use either, and each locale picks a default
(`Locale::division_style`).

- `bracket` (US default): long division under a bracket, `4)852`, quotient
  on top (`division/long.typ`); simple division as a vertical stack.
- `equation` (Norwegian default; also Germany, Denmark, Poland, …): the
  problem is one line, `852 : 4 = 213`. Long division writes its working
  under the dividend — each subtraction with a minus sign and a rule,
  then the next digit brought down — and appends `rest N`
  (`division/equation.typ`). Simple division is the drill's one-line
  layout. The mean scaffold puts `___ : n = ___` beside the addition
  stack, with the answer slot under it.

The style lives on the worksheet variant (`LongDivision`,
`SimpleDivision`, `Mean`), so component choice, cell size and pagination
follow it without consulting the locale. API: `division=bracket|equation`
(omitted → locale default); CLI: `--division`. The configurator shows a
"Layout" toggle on those three worksheets; it starts on the page locale's
default and, once picked, the choice lives in the URL and drives the
preview, download and QR link like any other setting.

Equation cells are wider (5–9cm vs 3.8–5.6cm), so the default column
count is computed from the cell width (`long_division_default_cols`), and
levels carry per-style column overrides (`styleParams` in `levels.ts`).
Multiplication keeps the vertical stack in both locales — Norwegian
schoolbooks use it too, with `·`.

## Visual regression

See `stories/` and `crates/stories/`. Each story is a `.typ` snippet
that renders a single component in isolation. Baselines are committed
as PNGs under `stories/baseline/`; diffs are computed as directional
RGB images. A change to shared typst (font, tracking, sizes)
automatically invalidates every baseline — which is the point.

**Diff colors:**

- **red**: pixels darker in baseline than current ("removed")
- **green**: pixels darker in current than baseline ("added")
- **dim gray**: shared content that didn't change
- **orange band**: canvas shrank — baseline had pixels here, current
  doesn't (rendered when the two images have different dimensions; each
  is centered on a union canvas)
- **blue band**: canvas grew — current has pixels here, baseline didn't

**Story conventions:**

- `#set page(width: auto, height: auto, margin: 0.5em)` — uniform 0.5em
  margin around the component's own bounding rect.
- Pass `debug: true` so the component renders its 1pt red bounding-rect
  outline. Stories are the visual-regression gate: the debug border is
  the rule that the bounding rect must match the visible content
  (see Layout rules → Bounding box).

Make targets: `make stories-gen`, `stories-diff`, `stories-check`
(regenerate + diff in one, non-zero exit on change), `stories-approve`
(promote current to baseline). Iterating on a thumb? `make thumb-pngs`
rasterizes each homepage thumb SVG to `output/thumbs/<kind>.png`
without round-tripping through the Astro build.

**Approval is a human review gate.** `make stories-approve` is run by
the user, not by the assistant — there's a corresponding rule in
`CLAUDE.md`. When stories fail in the assistant's flow, surface the
diff paths and stop.

## Homepage thumbnails

Each worksheet kind has a small A4-ratio SVG thumbnail rendered on
the homepage card grid. Sources live in
`frontend/astro/src/assets/thumbs/thumb-<kind>.typ` and compile to
sibling `.svg` files via `make thumbs`; `WorksheetThumb.astro`
inlines or lazy-loads them per route.

### Convention: rendered with a handwritten answer

Every thumbnail renders with `mode: "worked"` and spreads
`..thumb-answer-style` into the problem component's `opts`. That
combination paints the answer in the bundled handwriting font
(`Architects Daughter`) at a graphite-pencil grey (`#4a4a4a`) — so
the card reads as a printed problem with the student's filled-in
answer, not a finished worksheet. It's the brand cue: "you (the
kid) did this."

Concretely, every thumb source looks like:

```typst
#import "/lib/thumb-page.typ": thumb-answer-style, thumb-page
#import "/lib/problems/<folder>/<file>.typ": <component>

#show: thumb-page

#<component>(
  (...data...),
  mode: "worked",
  opts: (operator: [...], ..thumb-answer-style),
)
```

For the convention to actually fire, the problem component must
**read `answer-font` and `answer-color` from `opts`** and apply them
just to the answer (not the whole problem). The shared
`vertical-stack` and `decimal-vertical-stack` primitives already do
this; new components should follow the same pattern (see
`lib/problems/_layouts/decimal-vertical-stack.typ` for the
2-line `set text` block).

### Adding a thumbnail for a new kind

1. Pick a problem with a clean, recognisable shape — small numbers,
   no edge cases, and a final answer that reads at thumbnail size.
2. Write `thumb-<kind>.typ` following the template above.
3. Run `make thumbs` to compile to SVG; `make thumb-pngs` to
   rasterize for visual review without a full Astro build.
4. Register the thumb in `frontend/astro/src/components/WorksheetThumb.astro`
   (both the eager-import and lazy-URL maps).

`make thumbs` only re-runs when the `.typ` source changes; if you
edit a shared component (e.g. the layout primitive) and the thumb
needs re-rendering, `touch` the thumb sources or run
`make thumb-pngs --always-make`.

## Concept levels

Some worksheet kinds expose their configuration through a small set
of named **levels** instead of raw parameter knobs. Each level is a
curated preset — a handful of underlying params bundled under a
concept-flavored label and an example problem — and the configurator
shows them as a vertical picker with one selectable row per level.

- Definitions live in [`frontend/astro/src/lib/levels.ts`](./frontend/astro/src/lib/levels.ts).
- The browser URL stores `?level=N` (1-based index, consistent across
  kinds — `level=1` always means "first concept of whatever you're
  on"). `worksheetUrl()` expands that to the raw query params the
  server expects, so the backend remains unaware of levels.
- Level labels are concept-flavored, not difficulty-flavored
  ("Tenths × whole", "Plus & minus", "Smaller values"). The student
  is expected to work through *all* the levels — labels avoid
  "easy/medium/hard" or "challenge" framing that would imply level 3
  is bonus content.

### When to use levels

Use levels when one or more of these is true:

- **Many invalid combinations.** Decimal multiplication's full param
  matrix (`top digits` × `top dp` × `multiplier digits` × `multiplier
  dp`) has plenty of unproductive combos (4-dp top × 4-dp bottom = 8
  decimal places in the answer, won't fit a cell). A curated set of
  three concrete shapes is cleaner than a slider grid.
- **Curriculum has a natural progression.** Decimal addition: tenths
  → hundredths. One-step equations: additive inverse → multiplicative
  inverse → both. Two-step: smaller values → larger values. Picking
  the right level is "where is the student in the curriculum",
  which a parent can answer.
- **The shape can change between levels, not just magnitude.** Long
  division: 2-digit / 3-digit / 4-digit-with-remainders are different
  enough algorithms to warrant their own preset (and own
  problem-count + columns).

### When NOT to use levels

Stick with raw knobs (text inputs, range sliders, toggles) when:

- **The param space is small and every combo is meaningful.**
  Algebra-square-root has just two toggles (squares, roots) and all
  three combinations (squares only, roots only, both) are valid.
  Levels would add ceremony without simplifying.
- **The teacher's intent is targeted, not progression-shaped.**
  Multiplication / division drills are about picking *which specific
  table* to drill ("the 7s today"). A teacher wants the input box,
  not three preset levels.
- **A single linear axis already maps cleanly to a slider.** Simple
  division's `max_quotient` is already one slider — wrapping it in
  three levels is overhead.

### Adding levels to a new worksheet kind

1. Add an entry under `WORKSHEET_LEVELS` in `lib/levels.ts` — a
   `Level[]` with `label`, optional `example`, and a `params` dict
   that maps to the kind's raw query params.
2. Update the kind's TS config type in `lib/api.ts` to
   `{ kind: "...", level?: string }` and its case in `parseConfig`
   to read just `level: s("level")`.
3. Replace its `case` in `WorksheetConfig.tsx`'s `KindSpecific`
   switch with the shared `<LevelPicker>` call (the existing cases
   for `decimal-multiply` etc. are the template).
4. Add the kind to the `levelKinds` list in
   `WorksheetIsland.tsx`'s `applyFirstVisitDefaults` so first-visit
   loads default to level 1.

The Rust server stays untouched — levels are a frontend concern and
the API still accepts the raw params directly. That escape hatch is
deliberate: a future "advanced mode" UI can reintroduce raw knobs
without server work.

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

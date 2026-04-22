# Layout refactor — ideation

Status: draft / not yet scoped. Updated 2026-04-22.

## Motivation

`crates/core/src/template.rs` is doing three jobs, and the per-type
generators (`add.rs`, `multiply.rs`, …) leak through it:

- Each `generate_typ(params)` calls `template::render_*` with the full
  `WorksheetParams`. The template then handles student name, paper,
  debug, `include_answers`, symbol override, outline headings, page
  break logic, **and** the per-style `box_width` heuristic.
- Per-style dispatch happens three times: Rust picks the template fn
  → Rust picks a `style: "..."` string → typst `worksheet-grid`
  branches on it again.
- Answer-key rendering is a special-case branch inside the page loop
  instead of "same content, different render mode".

New requirements driving this round:

1. **Pagination by overflow.** The user asks for N total problems.
   Pages are derived from the layout's per-page capacity. Asking for 5
   algebra problems yields two pages (4 + 1), not one crammed page or
   a paging error.
2. **Three render modes per problem component.** Every component must
   produce (a) blank with work space, (b) fully worked, (c) just the
   final answer. Answer-only mode is new for several components.

## Units

Three units, three layers. This matches the existing codebase (see
`CLAUDE.md` → Layout rules and `shared.typ`).

| Layer | Unit | Used for |
|---|---|---|
| Page-level | **cm** | Paper dimensions, margins, header/footer, cell bbox, intro height, content-area math |
| Component-internal | **em** | Layout and padding inside a problem component — scales with `problem-text-size` |
| Typographic constants | **pt** | Font sizes, stroke widths, `problem-tracking` |

Conversion only happens at the page ↔ component boundary: a component
lays out in em, and its declared `cell_size_cm` is the bbox after
typst's em→absolute conversion at the component's own text size. Rust
never deals with em; typst never deals with cm *inside* a problem.

## Core idea — "problem = rectangle, grid is dumb"

A worksheet is a grid of rectangles. A problem component is a function
that takes data and returns a self-aligned, self-padded box of its
natural size. The grid doesn't branch on worksheet type; it just drops
boxes into `1fr` cells. Differences between long-division and
algebra-two-step collapse to "algebra reports a bigger rectangle".

Three things have to be true on the typst side for this to work:

1. **Unified component signature.** Every problem component has the
   same shape: `problem(data, mode, opts, debug) → box`.
2. **Components self-contain padding and alignment.** Today
   `layout.typ:48-70` branches on style to set `center+top` vs
   `right+top` and to wrap components in `pad(left: 0.3cm, right:
   1.5cm)`. That all moves inside the components.
3. **Components report their natural size** as a declared
   `cell_size_cm` pair, colocated with the typst function.

With those in place, `worksheet-grid` has no `if style == ...` chain,
and Rust doesn't need a `Style` enum at all.

## Render modes

A first-class three-valued enum replaces the `solved` / `answer-only`
flag pair.

```rust
pub enum RenderMode {
    Blank,       // unsolved, with work space. Default on problem pages.
    Worked,      // fully solved, worked steps visible. Used for solve-first.
    AnswerOnly,  // just the final answer. Used for answer-key pages.
}
```

Every component implements all three. Degenerate cases are fine —
`horizontal-problem` (`a × b = ___`) renders `Worked` and `AnswerOnly`
identically because there are no worked steps.

**Cell envelope rule:** all three modes share the same natural
rectangle. `Blank` dominates (it reserves work space); `Worked` fills
the space in; `AnswerOnly` leaves it empty but preserves the box. This
keeps the grid's cell pitch consistent across pages — problem pages
and answer-key pages have the same layout, just different fill.

We might want tighter answer-key pages later (smaller cells, more
problems per page). Out of scope for this refactor.

## Pagination by overflow

`num_problems` becomes **total** problems across the whole worksheet.
`pages` is derived.

```
cells_per_page = cols × rows_per_page
rows_per_page  = floor(content_area_h / cell_size.h)
pages          = ceil(num_problems / cells_per_page)
```

- `content_area_h` is page height minus margins, header, footer —
  already computed in `layout.typ:37` (`98% - header - footer`). Move
  that number into Rust so pagination can use it.
- `rows_per_page` is derived from `cell_size_cm` + paper + chrome, not
  a user parameter.
- `cols` stays user-facing (with a default per worksheet type and a
  max derived from `cell_size_cm.w × cols ≤ content_area_w`).

**Partial last page.** Last page might not be full (5 algebra
problems: 4 + 1). The grid still uses `rows_per_page` rows even if
partial, so cell pitch matches page 1. Empty cells stay empty.

**Answer-key pages.** One answer-key page per problem page, mirrored
layout with `mode: AnswerOnly`. No separate chunking logic.

**SVG/PNG.** Multi-page worksheets still require PDF. The existing
guard in `lib.rs:299-303` stays.

## Paper independence (A4 ↔ Letter)

Switching paper must be a single-field change with nothing else
touched. The rectangle model handles this cleanly if we're disciplined
about two rules:

1. **`Paper` is an enum, not a string.** Dimensions are owned by the
   type. Today it's `pub paper: String` in `lib.rs:241` — a source of
   string-matching bugs waiting to happen.
2. **Nothing downstream of `Chrome` talks to paper directly.**
   Everything uses the derived content area. Paper size influences
   exactly one number: `content_area_cm`.

### Paper type

```rust
pub enum Paper { A4, Letter }

impl Paper {
    pub fn dimensions_cm(self) -> (f32, f32) {
        match self {
            Paper::A4     => (21.0, 29.7),     // 210 × 297 mm
            Paper::Letter => (21.59, 27.94),   // 8.5 × 11 in
        }
    }
    pub fn typst_name(self) -> &'static str {
        match self { Paper::A4 => "a4", Paper::Letter => "us-letter" }
    }
}
```

### Content-area derivation

One function. Everything — pagination, `max_cols`, validation — keys
off its output.

```rust
impl Chrome {
    pub fn content_area_cm(&self) -> (f32, f32) {
        let (pw, ph) = self.paper.dimensions_cm();
        let m = MARGINS_CM;                     // constant, paper-agnostic
        let chrome_h = HEADER_HEIGHT_CM + FOOTER_HEIGHT_CM;
        (pw - m.left - m.right, ph - m.top - m.bottom - chrome_h)
    }
}
```

`HEADER_HEIGHT_CM = 1.5` and `FOOTER_HEIGHT_CM = 0.8` are the same
numbers as `layout.typ:35-36`. Mirror them as Rust constants so
pagination computes without round-tripping through typst.

### Cell size is absolute (cm), not em

Comparing cell to paper requires a common unit. An em depends on the
component's text size (22pt vertical, 18pt horizontal) — fine
internally, useless for paper math. The declared size is cm, matching
the rest of the page-level layer:

```rust
impl WorksheetType {
    fn cell_size_cm(&self, max_digits: u32) -> (f32, f32) { ... }
}
```

Components still use `em` internally; the declaration is just the
typst-layout bbox expressed as cm.

### What changes when paper switches

*Everything derives.* Picking Letter instead of A4 means:

- `content_area_cm` loses ~1.76cm vertically, gains ~0.59cm horizontally.
- `rows_per_page = floor(content_h / cell_h)` → typically 1 fewer row
  for tall styles (algebra, long-division); unchanged for dense ones.
- `max_cols = floor(content_w / cell_w)` → occasionally +1 for compact
  styles; usually unchanged.
- `pages = ceil(num_problems / (cols × rows_per_page))` → may grow by
  one for large problem counts on Letter.

No per-paper tuning, no per-paper special cases, no per-paper
fixtures. The same worksheet request produces a valid worksheet on
either paper — just possibly with a different page count.

### Margins

Start paper-agnostic with today's values. If printers force
per-paper margin tweaks later (US printers often clip beyond 0.25"),
promote `MARGINS_CM` to `fn margins_cm(paper: Paper)` — a one-line
change that ripples nowhere because everyone downstream reads
`content_area_cm`, not margins.

## Rust types

Four types, one direction of flow.

```rust
// 1. What a generator returns. Zero awareness of chrome/paging.
pub struct Sheet {
    pub worksheet: WorksheetType,        // already captures the type
    pub problems: Vec<Vec<u32>>,         // flat data, same convention as today
    pub opts: ComponentOpts,             // operator, answer_rows, pad_width,
                                         // implicit, variable — a flat bag
}

// 2. Render mode. Three-valued, explicit.
pub enum RenderMode { Blank, Worked, AnswerOnly }

// 3. Everything around the grid. Not a generator concern.
pub struct Chrome {
    pub student_name: Option<String>,
    pub paper: Paper,                    // enum: A4, Letter
    pub title: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub include_answers: bool,
    pub debug: bool,
}

// 4. The thing we actually render. Pages are derived.
pub struct Document {
    pub sheet: Sheet,
    pub cols: u32,                       // validated against cell_size_cm
    pub chrome: Chrome,
}
```

No `Style` enum. `WorksheetType` owns everything that was previously a
style property:

```rust
impl WorksheetType {
    /// Name of the typst function to call (see lib/problems/).
    fn typst_component(&self) -> &'static str { ... }

    /// Natural cell rectangle in cm (absolute, paper-comparable).
    /// Varies with operand digit count for Vertical and LongDivision.
    fn cell_size_cm(&self, max_digits: u32) -> (f32, f32) { ... }

    /// Starting (cols, rows_per_page) hint. User can override cols.
    fn default_cols(&self) -> u32 { ... }
}
```

Generator signature collapses to:

```rust
fn generate(params: &WorksheetParams) -> Result<Sheet>;
```

## Typst contract

Every problem component has the same shape:

```typst
#let <type>-problem(
  data,                        // tuple/dict of problem values
  mode: "blank",               // "blank" | "worked" | "answer-only"
  opts: (:),                   // style-specific knobs
  debug: false,
) = {
  // returns a box, self-aligned + self-padded, of natural cell size
}
```

Each component file also exports its declared cell size so the Rust
side and the typst side share one source of truth:

```typst
#let <type>-cell-size-cm = (w, h)   // natural bbox in cm
```

Rust reads these at build time (or keeps a parallel constant; pick
one). The grid becomes:

```typst
#let worksheet-grid(component, data-list, mode, num-cols, opts, debug: false) = {
  let num-rows = calc.quo(data-list.len() + num-cols - 1, num-cols)
  grid(
    columns: range(num-cols).map(_ => 1fr),
    rows: range(num-rows).map(_ => 1fr),
    ..data-list.map(d => component(d, mode: mode, opts: opts, debug: debug)),
  )
}
```

No `if style == ...`. No per-style padding wrappers. No per-style
alignment. The component owns all of that.

`lib/page.typ::worksheet-page` then wraps `worksheet-header`,
`worksheet-grid`, `worksheet-footer` into a single call; Rust emits
one such call per page plus `#pagebreak()`s.

## Isolated component rendering (stories / snapshot tests)

The rectangle model gives us single-component renders for free. Since
every component returns a self-padded, self-aligned box of its natural
size, wrapping one in a tight-crop page yields exactly the cell
rectangle — no header, footer, grid chrome, or wasted whitespace. The
output is a pixel-perfect baseline image.

### API

Thin wrapper on the existing `compile_typst` in `lib.rs:284`:

```rust
pub fn render_component(
    worksheet: WorksheetType,
    problem: &[u32],
    mode: RenderMode,
    opts: &ComponentOpts,
    format: OutputFormat,
    root: &Path,
    fonts: &Fonts,
) -> Result<Vec<u8>>
```

It emits:

```typst
#import "/lib/problems/shared.typ": body-font
#import "/lib/problems/algebra-two-step.typ": algebra-two-step-problem

#set page(width: auto, height: auto, margin: 0.2em)
#set text(font: body-font, size: 10pt)

#algebra-two-step-problem((4, 5, 3, 17, 0), mode: "blank", opts: (...))
```

`width: auto, height: auto` asks typst to crop the page to the
content. The small margin is breathing room so baseline comparisons
aren't flaky at the edges.

### Story organization

The harness iterates `component × mode × fixture`. Per-component
fixtures are the only per-component test code.

```
stories/
  fixtures/
    algebra-two-step.toml        # simple, implicit, emoji-var, long-numbers, ...
    long-division.toml           # 2-digit, 3-digit, 4-digit, with-remainder, ...
    vertical.toml                # add-2x2, mult-3x2, binary, multi-operand, ...
    horizontal.toml              # multiply, divide-colon, divide-obelus, ...
    horizontal-fraction.toml     # unit-half, non-unit, large-whole, ...
  baseline/
    algebra-two-step-blank-simple.png
    algebra-two-step-worked-simple.png
    algebra-two-step-answer-only-simple.png
    algebra-two-step-blank-implicit.png
    ...
```

Each component picks up `3 modes × N fixtures` of coverage for free —
`AnswerOnly` mode in particular starts getting tested without any new
baseline setup.

### Why this wasn't possible before

Today's components have non-uniform signatures, and padding +
alignment live in `layout.typ`'s grid wrapper. Rendering in isolation
either skips the grid (un-padded output) or includes the grid
(wasteful whitespace around a single cell). The rectangle model fixes
this — after migration step 3 (padding moved inside components),
isolated renders are tight and correct by construction. Step 4
(declared `cell_size_cm`) adds a sanity check: the rendered bounding
box should match the declared size within ±0.1cm (≈ 2.8pt).

## Validation

One function, one place.

```rust
impl Document {
    fn validate(&self) -> Result<()> {
        let max_digits = self.sheet.estimated_max_digits();
        let (cw, _ch) = self.sheet.worksheet.cell_size_cm(max_digits);
        let (w, _h) = self.chrome.content_area_cm();
        let max_cols = (w / cw).floor() as u32;
        if self.cols > max_cols {
            bail!("with {max_digits}-digit operands, {:?} supports max {max_cols} cols on {:?}",
                  self.sheet.worksheet, self.chrome.paper);
        }
        Ok(())
    }
}
```

Called once in `lib.rs::generate()` before emitting typst source. No
per-type capability table, no GridCapability struct — the cell size
is the capability.

## What moves where

| Today | After |
|---|---|
| `template.rs` emits full `.typ` source with per-style branches | `document.rs` emits one `#worksheet-page(...)` per page; no style branches |
| `lib.rs::generate_typ` → per-type `generate_typ` → `template::render_*` | `lib.rs::generate` → per-type `generate() -> Sheet` → `Document::render` |
| Five `render_*` wrappers in `template.rs` | One `Document::render` |
| `worksheet-grid` branches on `style` string | `worksheet-grid` takes a component function and forwards mode+opts |
| Padding/alignment per-style in `layout.typ` | Inside each component |
| Solved/answer-only as two bool flags | `RenderMode` enum |
| `num_problems × pages` | `num_problems` total; pages derived |
| `GridCapability` per style | `cell_size_cm` per worksheet type |
| `paper: String` | `Paper` enum; dimensions owned by the type |
| Paper used by template-string interpolation | Paper used only to derive `content_area_cm`; nothing downstream touches it |

## Migration sketch

Each step compilable on its own. Gate with `cargo test --workspace
--lib` and `make stories-check`.

1. **Introduce `RenderMode`.** Replace `solved` + `answer-only` bool
   pair at the Rust API layer. On the typst side, keep the two bools
   for now — `RenderMode` → `(solved, answer-only)` at the emission
   boundary. No behavior change.
2. **Unify component signatures on the typst side.** Each component
   accepts `(data, mode, opts, debug)`. Internally still does what it
   does today. Update `worksheet-grid` to call them uniformly — still
   branches on style for alignment/padding, still passes the same
   flags. Visual output unchanged.
3. **Push padding + alignment into components.** `layout.typ` stops
   branching; `worksheet-grid` shrinks to `data-list.map(component)`.
   Each component includes its own `pad(...)` and alignment. This is
   the biggest visual-diff risk — gate carefully.
4. **Declare `cell_size_cm` per worksheet type.** Add
   `WorksheetType::cell_size_cm` and `typst_component`. Wire
   `Document::validate` using them.
5. **Promote `Paper` to an enum.** Replace `pub paper: String`;
   add `Paper::dimensions_cm`, `Paper::typst_name`. Add
   `Chrome::content_area_cm` + `MARGINS_CM`, `HEADER_HEIGHT_CM`,
   `FOOTER_HEIGHT_CM` constants. No behavior change on A4; Letter
   starts producing correctly-bounded pagination automatically.
6. **Introduce `Sheet`, `Chrome`, `Document`.** Generators return
   `Sheet`; `template.rs` becomes `document.rs` taking
   `Document`. Collapse the five `render_*` functions into one.
7. **Switch pagination model.** `num_problems` becomes total. Pages
   derived from `content_area_cm / cell_size_cm`. `pages` parameter
   goes away (or becomes a no-op for a release, then is removed).
8. **Add `lib/page.typ::worksheet-page`.** Rust emits one call per
   page. Document preamble stays in Rust.
9. **Delete dead code.** `Style` never existed; remove any shims.

Steps 1–3 are incremental refinements that preserve behavior. Step 4
onward is the structural change. Step 7 is a breaking change to the
public CLI/HTTP API — bundle with a release note.

## Open questions

- **Where does `cell_size_cm` live — typst or Rust?** One source of
  truth per component. Leaning: declare it in the `.typ` file next to
  the component, mirror into Rust as a constant reviewed in the same
  PR. Not automated — two files, one review. Simple.
- **How does `FractionMultiply` decide its cell size?** Depends on
  whole-number range (wide digits widen the LHS). Same treatment as
  `Vertical`: `cell_size_cm(max_digits)`.
- **Should we keep `solve_first`?** It's "render problem 0 in `Worked`
  mode, rest in `Blank`". In the new model it's trivially expressible
  as per-problem mode selection. Keep it as a convenience flag on
  `Chrome`; the emitter translates to per-problem modes.

## Out of scope

- Typed `Problem` enum replacing `Vec<Vec<u32>>`. Positional tuples
  stay — separate cleanup.
- Variable font size / density presets. Add later as a `Density` knob
  that scales `problem-text-size` in `shared.typ`; orthogonal to
  everything here.
- Tighter answer-key pages (different cell size than problem pages).
  Currently both use the same envelope.
- Refactoring `worksheet-grid`'s content-area math to live in Rust.
  Either side can own it; pick in step 4.
- Per-paper margin overrides. `MARGINS_CM` is a constant until a real
  printer-clipping case shows up; at that point it becomes
  `fn margins_cm(paper: Paper)`.
- Additional paper sizes (A5, Legal). The `Paper` enum admits new
  variants trivially once A4/Letter are solid.

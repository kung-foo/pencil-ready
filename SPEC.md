# Math Worksheets

I am teaching my daughter the basic in mathematics. She is currently in 5th grade in Norway. I've been using various sites to generate worksheets that I print out and have her do. I'm 47, and grew up learning math through repetition. So I want to continue that metheodology since "it worked for me". The current issue is that the site that I used up until now, has turning the advertising to 11, and now blocks any users that are using adbockers (which everyone should). So I want to start building a site that replicates what I used to use. But of course, we will make it better, faster, and free.

There are a few example PDFs and screen shots in ./examples

Let's start with the basics and figure out what the architecture will be.

* a React app?
* the general idea is to generate an SVG and then render that to PDF?

Requirements:
* pixel perfect PDF layout
* currated fonts

Example options for a "multiply" page:

* select number of problems (affects layout)
* number of digits on top
* number of digits on bottom
* print an answer sheet?


Questions:
* should we start with just the SVG->PDF first and do some local python programing instead of the full react app?

## Architecture (settled)

### Rendering: Typst

We use [Typst](https://typst.app/) for document generation. Typst is a modern typesetting system (like LaTeX but faster and simpler) that compiles `.typ` source files to PDF, PNG, and SVG. Compilation is sub-100ms, native binary, no VMs or browsers involved.

* Typst library files in `lib/` define reusable components (header, footer, problem cell, grid layout)
* A generated `.typ` file contains only the problem data and imports — all layout logic lives in the library
* Fonts are bundled in `fonts/` (B612, B612 Mono, Noto Sans Math, Noto Color Emoji) — system fonts are ignored via `TYPST_IGNORE_SYSTEM_FONTS=true` for deterministic builds
* Custom SVG assets (e.g. rainbow heart) can be inlined via `image()` in typst

### Output formats

Each worksheet is compiled to all three formats:
* **PDF** — fonts embedded/subsetted, text selectable
* **PNG** — rasterized at 300 PPI for print
* **SVG** — glyphs converted to vector paths, fully self-contained

### Prototyping tool

`tools/gen.py` is a Python CLI that generates worksheets for development/testing. It takes params (operation, digit counts, problem count, etc.), produces a `.typ` file, and optionally compiles it.

### Target architecture

* **Rust binary** with typst embedded as a library (no shelling out)
* **API server** (axum/actix) handles worksheet generation requests
* **React frontend** for the user-facing UI
* Flow: user picks options in React UI → POST to API → Rust generates typst source, compiles in-process → returns PDF/PNG/SVG

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

- **Digits**: `B612 Mono` (via `problem-font` in `shared.typ`) — monospaced so
  columns align cleanly.
- **Operator symbols**: `Noto Sans Math` (via `operator-font`) — better glyph
  centering and coverage for math symbols (×, ÷, ·, :, etc.) than B612.
- **Header/footer/labels**: `B612` (proportional) — cleaner for text.

### Locale

Locale (`--locale us` / `--locale no`) affects only **horizontal** layouts
(drills), since vertical and bracket notations are universal.

- US: `×` for multiply, `÷` for divide
- Norway: `·` for multiply, `:` for divide

The explicit `--symbol` flag overrides locale for any worksheet.

## Visual regression

See `stories/` and `crates/stories/`. Each story is a `.typ` snippet that
renders a single component in isolation. Baselines are committed; diffs are
computed as directional RGB images (red = removed, green = added pixels). A
change to shared typst (font, tracking, sizes) automatically invalidates every
baseline — which is the point.

## TODO

- **Problem box width**: currently computed as `max(2.2, digits * 0.55 + 0.6)` cm — magic numbers hand-tuned for Cascadia Code at 22pt. Needs to be derived from font metrics or made configurable when we support multiple font sizes. (Partially addressed: components now use `em`, so scaling works at the component level. The worksheet-level `box_width` formula in `template.rs` still uses `cm`.)

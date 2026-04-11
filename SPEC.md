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
* Fonts are bundled in `fonts/` (Cascadia Code, Noto Color Emoji) — system fonts are ignored via `TYPST_IGNORE_SYSTEM_FONTS=true` for deterministic builds
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

## TODO

- **Problem box width**: currently computed as `max(2.2, digits * 0.55 + 0.6)` cm — magic numbers hand-tuned for Cascadia Code at 22pt. Needs to be derived from font metrics or made configurable when we support multiple font sizes.

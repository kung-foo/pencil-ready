#let handwriting-font = "Architects Daughter"
#let title-font = "Crimson Text"

// A filled-in field: handwritten name in a zero-height overlay sitting on
// the signature line position. Zero height keeps the grid row at the same
// height as the blank-underline case so nothing else in the header shifts.
// Long names are shrunk to fit the field width so they can't collide with
// the Date column.
#let handwritten-field(name, width) = {
  let base-size = 18pt
  box(width: width, height: 0pt,
    place(bottom + left, dy: 1pt,
      context {
        let natural = measure(text(font: handwriting-font, size: base-size)[#name])
        let size = if natural.width > width {
          base-size * (width / natural.width)
        } else {
          base-size
        }
        text(font: handwriting-font, size: size)[#name]
      }))
}

// Page-chrome header. Two layout modes — the caller picks via
// `title`/`instructions` rather than an explicit flag.
//
//   COMPACT (no title, no instructions): single 1.5cm box with the
//   Name/Date row above a horizontal rule. Used by tests/stories that
//   exercise the chrome in isolation.
//
//   TALL (title or instructions set): 3.2cm box organised into three
//   fixed-height sections so that the chrome's bottom edge — and
//   therefore the body grid's top edge — is in the same y position
//   regardless of whether Name/Date or instructions are rendered:
//     1. Name/Date area    (0.7cm — Name/Date row + breathing room)
//     2. Title / rule area (0.85cm — title flanked by horizontal
//        rules, with the rules at the title's x-height midline; this
//        IS the page divider, so no extra rule is drawn below)
//     3. Instructions area (1.65cm — fits two lines of 11pt italic
//        body text; longer text should be truncated upstream rather
//        than clipped here)
//   Empty sections preserve their declared height. The answer-key
//   page uses TALL with `show-name-date: false, instructions: none`
//   so Name/Date and instructions sections are blank but the title
//   row sits at the same y as on a problem page — letting students
//   line up problems and answers across the page-fold.
//
//   show-name-date: render the Name / Date row. Default true. Set
//                   false on answer-key pages.
//   title:          worksheet title in `title-font` (Crimson Text).
//   instructions:   short imperative body text. Wraps to two lines
//                   max within the chrome; longer text overflows.
//
// The TALL chrome assumes margin.top is set wide enough to fit it
// (≈ 5.2cm). See `MARGINS_CM` in `pencil_ready_core`.
#let worksheet-header(
  student-name: none,
  show-name-date: true,
  title: none,
  instructions: none,
  debug: false,
) = {
  let debug-box = if debug { 1pt + red } else { none }

  let name-field = if student-name != none {
    handwritten-field(student-name, 10cm)
  } else {
    line(length: 10cm, stroke: 0.5pt)
  }

  let name-date-row = {
    set text(size: 12pt)
    grid(
      columns: (auto, 1fr, auto, auto),
      column-gutter: 0.3cm,
      align: bottom,
      [*Name*:], name-field, [*Date*:], line(length: 4cm, stroke: 0.5pt),
    )
  }

  let has-banner = title != none or instructions != none

  if not has-banner {
    // COMPACT mode — preserves the legacy 1.5cm chrome so existing
    // tests/stories that call `worksheet-header()` with only
    // `student-name` keep their pixel-exact baseline.
    box(height: 1.5cm, width: 100%, stroke: debug-box, {
      if show-name-date { name-date-row }
      v(0.3cm)
      line(length: 100%, stroke: 1.5pt)
    })
  } else {
    // TALL mode — three fixed-height sections.
    let name-date-h = 0.7cm
    let title-rule-h = 0.85cm
    let instructions-h = 1.65cm

    // Title flanked by horizontal rules. `align: horizon` puts the
    // rules at the title row's bounding-box centre, which lands on
    // the cap-height midline; the 3pt downward nudge moves them onto
    // the x-height midline (where the eye reads the word's optical
    // centre). The flanking rules ARE the page divider — no extra
    // rule is drawn below.
    let title-rule-grid = if title != none {
      grid(
        columns: (1fr, auto, 1fr),
        column-gutter: 0.5cm,
        align: horizon,
        move(dy: 3pt, line(length: 100%, stroke: 1pt)),
        text(font: title-font, size: 22pt, weight: "semibold")[#title],
        move(dy: 3pt, line(length: 100%, stroke: 1pt)),
      )
    }

    // Stack the three sections in a grid with exact row heights and
    // zero gutter — `block(...)` would insert default paragraph
    // spacing between sections, pushing instructions below their
    // intended slot. The grid pins each section's top edge to the
    // sum of preceding row heights, so content within each section
    // is naturally top-aligned.
    // Stack the three sections in a grid with exact row heights and
    // zero gutter — `block(...)` would insert default paragraph
    // spacing between sections, pushing instructions below their
    // intended slot. The grid pins each section's top edge to the
    // sum of preceding row heights; `align: top + left` ensures
    // content is anchored to the row's top edge rather than its
    // baseline (typst's default for grid cells).
    box(height: 3.2cm, width: 100%, stroke: debug-box, grid(
      columns: 1,
      rows: (name-date-h, title-rule-h, instructions-h),
      row-gutter: 0pt,
      align: top + left,
      // Row 1 — Name/Date
      if show-name-date { name-date-row } else { [] },
      // Row 2 — title between rules
      if title-rule-grid != none { title-rule-grid } else { [] },
      // Row 3 — instructions, inheriting the body font set by the
      // document preamble.
      if instructions != none {
        text(size: 11pt)[#instructions]
      } else { [] },
    ))
  }
}

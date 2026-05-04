#let handwriting-font = "Architects Daughter"
#let title-font = "Crimson Text"

// Section heights for the variable-chrome layout below. Exposed so
// `pencil_ready_core::header` can compute matching `margin.top` and
// pagination math without duplicating the constants.
#let name-date-section-h = 0.7cm
#let title-section-h = 0.85cm
#let instructions-section-h = 1.65cm
#let compact-h = 1.5cm

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

// Page-chrome header. Three optional sections; chrome height adapts to
// which are present:
//
//   show-name-date: Name/Date row. Set true to include the section,
//                   false to omit (e.g. answer-key pages — they no
//                   longer need to align to the matching problem page).
//   title:          worksheet title in `title-font` (Crimson Text),
//                   centred between horizontal rules nudged 3pt down
//                   to the title's x-height midline. The flanking
//                   rules act as the page divider.
//   instructions:   short imperative body text. Wraps to two lines
//                   max; longer text should be truncated upstream.
//
//   student-name:   optional pre-filled name printed on the Name line
//                   in a handwriting font. Ignored when
//                   `show-name-date` is false.
//
// Resulting chrome height = sum of present-section heights. Callers
// must size `margin.top` to match — see
// `pencil_ready_core::header::margin_top_for` (Rust) or the constants
// re-exported above (typst).
//
// One legacy-compat exception: `show-name-date && !title &&
// !instructions` renders the original 1.5cm box (Name/Date + plain
// rule + bottom padding) so existing `header` / `header-named` story
// baselines stay pixel-exact.
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

  // Title flanked by horizontal rules. `align: horizon` puts the rules
  // at the title row's bounding-box centre, which lands on the cap-
  // height midline; the 3pt downward nudge moves them onto the x-
  // height midline (where the eye reads the word's optical centre).
  let title-rule-grid = if title != none {
    grid(
      columns: (1fr, auto, 1fr),
      column-gutter: 0.5cm,
      align: horizon,
      move(dy: 3pt, line(length: 100%, stroke: 1pt)),
      text(font: title-font, size: 22pt, weight: "semibold")[#title],
      move(dy: 3pt, line(length: 100%, stroke: 1pt)),
    )
  } else { none }

  let instructions-block = if instructions != none {
    text(size: 11pt)[#instructions]
  } else { none }

  let has-title = title != none
  let has-instructions = instructions != none

  // Legacy compact case: Name/Date row only, no banner. Preserves the
  // 1.5cm box so the existing `header` / `header-named` story
  // baselines render identically.
  if show-name-date and not has-title and not has-instructions {
    box(height: compact-h, width: 100%, stroke: debug-box, {
      name-date-row
      v(0.3cm)
      line(length: 100%, stroke: 1.5pt)
    })
  } else {
    // Variable layout: build a list of (height, content) sections in
    // order, then stack them in a grid. The grid's row tracks pin
    // each section to its declared height with zero gutter.
    let sections = ()
    if show-name-date {
      sections.push((name-date-section-h, name-date-row))
    }
    if has-title {
      sections.push((title-section-h, title-rule-grid))
    } else if has-instructions and show-name-date {
      // Name + instructions, no title — need a plain rule to divide
      // Name/Date from the instructions block. Borrow a slim slice of
      // the instructions-section budget for it (the rule itself plus
      // ~5pt of padding); the instructions text starts just below.
      sections.push((0.35cm, {
        v(0.05cm)
        line(length: 100%, stroke: 1.5pt)
      }))
    }
    if has-instructions {
      sections.push((instructions-section-h, instructions-block))
    }

    if sections.len() == 0 {
      // Degenerate: no sections requested. Render nothing.
      []
    } else {
      let total-h = sections.fold(0cm, (acc, s) => acc + s.at(0))
      box(height: total-h, width: 100%, stroke: debug-box, grid(
        columns: 1,
        rows: sections.map(s => s.at(0)),
        row-gutter: 0pt,
        align: top + left,
        ..sections.map(s => s.at(1)),
      ))
    }
  }
}

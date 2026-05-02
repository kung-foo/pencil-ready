// Shared layout for problems built around column-aligned `=` signs:
// algebra-one-step, algebra-two-step, fraction-multiplication, etc.
//
// Renders a 3-column grid with col1 = col3 (symmetric around `=`).
// col1 = max(widest LHS, widest RHS) across all rows so equals signs
// line up vertically AND every row's LHS/RHS sits in the same slot
// on either side of the `=`. col2 is the natural width of the `=`
// glyph.
//
//   row1-lhs |  =  | row1-rhs
//   row2-lhs |  =  | row2-rhs
//   ...
//
// `rows` is an array of `(lhs, rhs)` tuples. To keep the bounding box
// identical between blank and solved modes, callers should hide
// solved-only content with `hide(...)` rather than swapping it out
// for empty: `hide` reserves the full bounding rect of its argument
// but doesn't paint anything, so the col-width measurement is
// stable regardless of mode.
//
// `col-width` is auto by default — each problem self-sizes from its
// own rows. Pass an explicit length (e.g. from a worksheet-page
// pre-pass that measures every problem on the page) to force uniform
// column widths across multiple problems so their `=` signs align.

#let equation-rows(rows, col-width: auto, row-gutter: 1.3em, debug: false) = {
  context {
    // Measure each row's LHS and RHS to find the widest cell on each side.
    // col-width then collapses both sides to a single symmetric value.
    let lhs-max = 0pt
    let rhs-max = 0pt
    for (lhs, rhs) in rows {
      lhs-max = calc.max(lhs-max, measure(lhs).width)
      rhs-max = calc.max(rhs-max, measure(rhs).width)
    }
    let col-w = if col-width == auto {
      calc.max(lhs-max, rhs-max)
    } else {
      col-width
    }
    let eq-w = measure(sym.eq).width

    // Build grid cells: each row contributes (LHS right-aligned in
    // col1) (= centered in col2) (RHS left-aligned in col3).
    let cells = ()
    for (lhs, rhs) in rows {
      cells.push(align(right + horizon, lhs))
      cells.push(align(center + horizon, sym.eq))
      cells.push(align(left + horizon, rhs))
    }

    // Top inset is generous (0.6em) so superscripts like `x²` get
    // breathing room above the bounding rect — typst's math layout
    // measures equations by cap-height, so an exponent's ascender
    // isn't part of the equation's reported height. Tightening the
    // top inset makes the `²` poke against the box's top edge.
    // Bottom inset stays small since descenders below the baseline
    // are rare in this grid.
    box(
      stroke: if debug { 1pt + red } else { none },
      inset: (top: 0.6em, bottom: 0.4em, x: 0.2em),
      grid(
        columns: (col-w, eq-w, col-w),
        column-gutter: 0.3em,
        row-gutter: row-gutter,
        ..cells,
      ),
    )
  }
}

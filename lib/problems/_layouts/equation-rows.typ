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

#let equation-rows(
  rows,
  col-width: auto,
  row-gutter: 1.3em,
  symmetric: true,
  rhs-align: left + horizon,
  debug: false,
) = {
  context {
    // Measure each row's LHS and RHS to find the widest cell on each side.
    let lhs-max = 0pt
    let rhs-max = 0pt
    for (lhs, rhs) in rows {
      lhs-max = calc.max(lhs-max, measure(lhs).width)
      rhs-max = calc.max(rhs-max, measure(rhs).width)
    }
    // `symmetric: true` (default): col1 = col3 = max(lhs-max, rhs-max).
    // Puts `=` at the visual center of the bounding rect — when the
    // worksheet grid centers each cell, every problem's `=` lands at
    // the same x in its column.
    //
    // `symmetric: false`: col1 = lhs-max, col3 = rhs-max. Tighter
    // bounding rect with no right-side slack, but `=` no longer sits
    // at the bbox center. Use for single-problem renderings (thumbs)
    // where the worksheet-grid alignment doesn't apply.
    //
    // `col-width` (when not auto) overrides both — useful for a
    // worksheet template that pre-measures and forces a uniform
    // width across multiple problems on a page.
    let (col1-w, col3-w) = if col-width != auto {
      (col-width, col-width)
    } else if symmetric {
      let w = calc.max(lhs-max, rhs-max)
      (w, w)
    } else {
      (lhs-max, rhs-max)
    }
    let eq-w = measure(sym.eq).width

    // Build grid cells: each row contributes (LHS right-aligned in
    // col1) (= centered in col2) (RHS aligned per `rhs-align` in
    // col3 — default `left + horizon` puts the RHS adjacent to `=`).
    // Pass `center + horizon` for layouts where col3 holds varied
    // content widths across rows (e.g. fraction-mult: "60/4" then
    // "15") and you want the values to stack vertically aligned by
    // their centers rather than by their left edges.
    let cells = ()
    for (lhs, rhs) in rows {
      cells.push(align(right + horizon, lhs))
      cells.push(align(center + horizon, sym.eq))
      cells.push(align(rhs-align, rhs))
    }

    // Top inset so above-cap-height content (superscripts, fraction
    // numerators) gets breathing room above the bounding rect. Typst's
    // math layout reports equation height by cap-height, so an
    // exponent's ascender or a math.frac's numerator isn't part of the
    // reported frame — without padding they poke against the box's
    // top edge. 0.5em is enough for typical superscripts and
    // single-line fractions.
    box(
      stroke: if debug { 1pt + red } else { none },
      inset: (top: 0.5em, bottom: 0.4em, x: 0.2em),
      grid(
        columns: (col1-w, eq-w, col3-w),
        column-gutter: 0.3em,
        row-gutter: row-gutter,
        ..cells,
      ),
    )
  }
}

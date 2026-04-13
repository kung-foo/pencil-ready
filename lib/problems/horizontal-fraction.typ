// Horizontal fraction problem: `whole × num/den = ___`
//
//   8 × 1/2 = ___
//
// The fraction is rendered via math.frac (auto-sizes for inline math).

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features

#let horizontal-fraction-problem(numbers, operator, debug: false) = {
  // numbers = (whole, numerator, denominator)
  // NOTE: do NOT set tracking on the outer text — math.frac inherits the
  // outer text settings and inserts a visible gap between digits of multi-
  // digit numerators/denominators (e.g. "10" rendered as "1 0"). Apply
  // tracking only to the whole-number where we need it.
  set text(
    font: problem-font,
    size: problem-text-size-horizontal,
    features: problem-features,
  )
  // Use Fira Math for equations (math font controls the digit rendering
  // inside math.frac — the outer text font doesn't propagate there).
  show math.equation: set text(font: "Fira Math", features: ())
  let debug-box = if debug { 1pt + red } else { none }
  let whole = str(numbers.at(0))
  let n = numbers.at(1)
  let d = numbers.at(2)

  let answer-blank = box(width: 2em, height: 1em, stroke: (bottom: 0.5pt))

  let op-box = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    operator
  }))

  // Single non-breaking inline box so nothing wraps.
  //
  // The inline math fraction's denominator descends below the text baseline,
  // so wrap the equation in its own box to pin its full extent, then give
  // the outer box bottom inset so the debug stroke and layout grid contain
  // the descender. The top already has plenty of room from the text's ascent.
  box(stroke: debug-box, inset: (top: 0.1em, bottom: 0.6em), {
    text(tracking: problem-tracking, whole)
    op-box
    box($#str(n)/#str(d)$)
    h(0.3em)
    sym.eq
    h(0.3em)
    answer-blank
  })
}

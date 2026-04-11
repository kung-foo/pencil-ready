// Horizontal single-line layout: used by multiplication drill.
//
//   7 × 3 = _____
//
// The entire problem is one non-breaking box so nothing wraps.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking

#let horizontal-problem(numbers, operator, width: 6cm, debug: false) = {
  set text(font: problem-font, size: problem-text-size-horizontal, tracking: problem-tracking)
  let debug-box = if debug { 1pt + red } else { none }
  let a = str(numbers.at(0))
  let b = str(numbers.at(1))

  // Answer blank: an empty box with a bottom border, fully inline.
  let answer-blank = box(width: 2em, height: 1em, stroke: (bottom: 0.5pt))

  // Fixed-width box for the operator so × and · take the same space.
  // Use Noto Sans Math for operator symbols — better glyph centering.
  let op-box = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    operator
  }))

  // Everything in one inner box so it can never line-break.
  let problem = box({
    text(a)
    op-box
    text(b)
    h(0.3em)
    sym.eq
    h(0.3em)
    answer-blank
  })

  box(width: 100%, height: 2em, stroke: debug-box, align(right + horizon, problem))
}

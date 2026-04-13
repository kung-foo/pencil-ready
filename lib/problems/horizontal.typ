// Horizontal single-line layout: used by multiplication drill.
//
//   7 × 3 = _____
//
// The entire problem is one non-breaking box so nothing wraps.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features

// Renders the natural-width problem content. The caller (worksheet grid
// or story page) is responsible for any fill/alignment behavior.
#let horizontal-problem(numbers, operator, debug: false) = {
  set text(font: problem-font, size: problem-text-size-horizontal, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let a = str(numbers.at(0))
  let b = str(numbers.at(1))

  let answer-blank = box(width: 2em, height: 1em, stroke: (bottom: 0.5pt))

  let op-box = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    operator
  }))

  // Single non-breaking inline box so nothing wraps.
  box(stroke: debug-box, {
    text(a)
    op-box
    text(b)
    h(0.3em)
    sym.eq
    h(0.3em)
    answer-blank
  })
}

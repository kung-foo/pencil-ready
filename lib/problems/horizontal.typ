// Horizontal single-line layout: used by multiplication drill.
//
//   7 × 3 = _____
//
// The entire problem is one non-breaking box so nothing wraps.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features

// Renders the natural-width problem content. The caller (worksheet grid
// or story page) is responsible for any fill/alignment behavior.
//
// `numbers` = (a, b, answer). The answer is pre-computed by the generator
// so this component doesn't need to know the operation.
#let horizontal-problem(numbers, operator, debug: false, solved: false) = {
  set text(font: problem-font, size: problem-text-size-horizontal, tracking: problem-tracking, features: problem-features)
  let debug-box = if debug { 1pt + red } else { none }
  let a = str(numbers.at(0))
  let b = str(numbers.at(1))
  let answer = if numbers.len() > 2 { str(numbers.at(2)) } else { "" }

  // Fixed-width slot so solved and unsolved problems share the same bounding
  // box. When solved, the answer sits on the slot; otherwise the slot shows
  // a blank line. Use bottom alignment so the answer text sits on the
  // baseline of the surrounding "a × b =" flow — center+horizon put the
  // answer above the baseline, making it look like it was floating.
  let answer-slot = box(
    width: 2em,
    height: 1em,
    stroke: if solved { none } else { (bottom: 0.5pt) },
    align(center + bottom, if solved { text(answer) }),
  )

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
    answer-slot
  })
}

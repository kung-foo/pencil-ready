// Horizontal fraction problem: two-step layout
//
//   8 × 1/2 = ___
//           = ___
//
// Row 1 holds the multiply-across intermediate (a fraction).
// Row 2 holds the simplified integer answer.
// The `=` signs align vertically to reinforce the equals-column habit.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size-horizontal, problem-tracking, problem-features, problem-line-height

#let horizontal-fraction-problem(numbers, operator, debug: false, solved: false, answer-only: false) = {
  // numbers = (whole, numerator, denominator)
  // solved  = if true, fill in the worked answer (multiply-across + simplified)
  //           as a demonstration example.
  // answer-only = when solved, suppress the multiply-across intermediate
  //               and show only the simplified integer answer.
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
  let whole-v = numbers.at(0)
  let whole = str(whole-v)
  let n = numbers.at(1)
  let d = numbers.at(2)

  // Worked-answer values (only rendered when solved: true).
  let inter-num = whole-v * n
  let final = inter-num / d  // integer division: answers are always whole

  // Reserve fixed horizontal space on the right so solved and unsolved
  // problems occupy the same width — the worksheet grid stays aligned.
  let slot-width = 3.2em
  let row1-right = box(width: slot-width, height: 1em, align(left + horizon, {
    if solved and not answer-only { $#str(inter-num)/#str(d)$ }
  }))
  let row2-right = box(width: slot-width, height: 1em, align(left + horizon, {
    if solved {
      text(tracking: problem-tracking, str(int(final)))
    }
  }))

  let op-box = box(width: 1.2em, align(center, {
    set text(font: operator-font)
    operator
  }))

  let lhs = {
    text(tracking: problem-tracking, whole)
    op-box
    box($#str(n)/#str(d)$)
  }

  // 3-column grid keeps the `=` signs aligned across both rows.
  // Row 1: [whole × n/d] [=] [intermediate slot]
  // Row 2: [          ] [=] [integer      slot]
  box(stroke: debug-box, inset: (top: 0.2em, bottom: 0.4em, x: 0.2em), grid(
    columns: (auto, auto, auto),
    column-gutter: 0.3em,
    row-gutter: problem-line-height,
    align: (right + horizon, center + horizon, left + horizon),
    lhs, sym.eq, row1-right,
    [], sym.eq, row2-right,
  ))
}

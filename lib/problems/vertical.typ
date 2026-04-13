// Vertical stacked layout: used by add, subtract, multiply, simple divide.
//
//     123
//   + 456
//   ─────
//
// Supports N operands. Only the last operand gets the operator symbol.

#import "/lib/problems/shared.typ": problem-font, operator-font, problem-text-size, problem-tracking

// `answer-rows` = how many rows of writing space to reserve below the line.
//   1 for add/subtract (single answer line)
//   ~partial products + 1 for multiply (e.g. 2×2 needs 3: two partials + sum)
#let vertical-problem(
  numbers,
  operator,
  width: 2.8em,
  answer-rows: 1,
  debug: false,
) = {
  set text(font: problem-font, size: problem-text-size, tracking: problem-tracking)
  let debug-box = if debug { 1pt + red } else { none }
  let first = str(numbers.at(0))
  let rest = numbers.slice(1)

  // 1.3em ≈ one typeset line at this size (font em + leading).
  let carry-space = 0.5em
  let answer-space = 1.3em * answer-rows

  box(
    width: width,
    stroke: debug-box,
    inset: (top: carry-space, bottom: answer-space),
    {
      set par(leading: 0.3em)
      align(right, text(first))
      let middle = rest.slice(0, rest.len() - 1)
      let last = rest.last()
      for n in middle {
        v(-0.65em)
        align(right, text(str(n)))
      }
      v(-0.65em)
      grid(
        columns: (auto, 1fr),
        column-gutter: 0.25em,
        align(left, {set text(font: operator-font); operator}), align(right, text(str(last))),
      )
      v(-0.9em)
      line(length: 100%, stroke: 0.8pt)
    },
  )
}

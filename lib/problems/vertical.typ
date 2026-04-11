// Vertical stacked layout: used by add, subtract, multiply, simple divide.
//
//     123
//   + 456
//   ─────
//
// Supports N operands. Only the last operand gets the operator symbol.

#import "/lib/problems/shared.typ": problem-font, problem-text-size, problem-tracking

#let vertical-problem(numbers, operator, width: 2.2cm, debug: false) = {
  set text(font: problem-font, size: problem-text-size, tracking: problem-tracking)
  let debug-box = if debug { 1pt + red } else { none }
  let first = str(numbers.at(0))
  let rest = numbers.slice(1)

  box(width: width, stroke: debug-box, {
    set par(leading: 0.3em)
    align(right, text(first))
    let middle = rest.slice(0, rest.len() - 1)
    let last = rest.last()
    for n in middle {
      v(-0.5cm)
      align(right, text(str(n)))
    }
    v(-0.5cm)
    grid(
      columns: (auto, 1fr),
      column-gutter: 0.2cm,
      align(left, operator), align(right, text(str(last))),
    )
    v(-0.7cm)
    line(length: 100%, stroke: 0.8pt)
  })
}

#import "/lib/problems/decimal/multiply.typ": decimal-multiply-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Worked: 2.5 × 3 = 7.5, 12.34 × 7 = 86.38, and 4.7 × 5.6 = 26.32 —
// points at different positions, so every row has two spacers.
#grid(
  columns: 3,
  column-gutter: 1em,
  decimal-multiply-problem(
    (25, 3, 75),
    mode: "worked",
    opts: (operator: [#sym.times], width: 3.5em, decimal-places: (1, 0, 1)),
    debug: true,
  ),
  decimal-multiply-problem(
    (1234, 7, 8638),
    mode: "worked",
    opts: (operator: [#sym.times], width: 3.5em, decimal-places: (2, 0, 2)),
    debug: true,
  ),
  decimal-multiply-problem(
    (47, 56, 2632),
    mode: "worked",
    opts: (operator: [#sym.times], width: 3.5em, answer-rows: 3, decimal-places: (1, 1, 2), decimal-sep: ","),
    debug: true,
  ),
)

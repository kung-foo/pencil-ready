// Answer-key rendering: the sum and the quotient, no long-division
// scratch work. Same footprint as the blank cell so the answer page
// lines up cell-for-cell with the problem page.
#set page(width: auto, height: auto, margin: 0.5em)
#import "/lib/problems/statistics/mean.typ": mean-problem

#mean-problem(
  (132, 128, 141, 135, 536, 134),
  mode: "answer-only",
  opts: (
    operator: [#sym.plus],
    stack-width: 2.6cm,
    div-width: 3.8cm,
    div-rows: 6,
    data-height: 1.4cm,
    body-height: 9cm,
    scaffold: true,
  ),
  debug: true,
)

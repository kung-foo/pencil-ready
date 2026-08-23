// Blank, scaffold off: only the data line paints. Both halves are
// hidden, so the cell is the same size as the scaffolded version — the
// student decides to sum and divide, and does all the writing herself.
#set page(width: auto, height: auto, margin: 0.5em)
#import "/lib/problems/statistics/mean.typ": mean-problem

#mean-problem(
  (132, 128, 141, 135, 536, 134),
  opts: (
    operator: [#sym.plus],
    stack-width: 2.6cm,
    div-width: 3.8cm,
    div-rows: 6,
    data-height: 1.4cm,
    body-height: 9cm,
    scaffold: false,
  ),
  debug: true,
)

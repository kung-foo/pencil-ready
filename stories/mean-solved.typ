// Worked example: sum filled in, and the full long-division algorithm
// (brought-down digits, subtractions, final remainder 0) under the
// bracket with the mean as the quotient.
#set page(width: auto, height: auto, margin: 0.5em)
#import "/lib/problems/statistics/mean.typ": mean-problem

#mean-problem(
  (132, 128, 141, 135, 536, 134),
  mode: "worked",
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

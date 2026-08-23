// Blank, scaffold on: data line, pre-filled addition stack, the division
// bracket with its divisor but no dividend (the count is the hint worth
// giving; the total is the work), and the final answer slot. The
// dividend and quotient keep their footprint via `hide`, so the space
// left for the student is exactly what the worked solution occupies.
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
    scaffold: true,
  ),
  debug: true,
)

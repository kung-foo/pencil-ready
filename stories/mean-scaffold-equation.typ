// Blank, scaffold on, equation division (Norwegian notation): the
// pre-filled addition stack, and beside it `___ : 4 = ___` with the
// dividend held back and the working space reserved under it. The
// answer slot sits under the division, whose result it is.
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
    body-height: 6.5cm,
    scaffold: true,
    division-style: "equation",
    div-operator: [#sym.colon],
    decimal-sep: ",",
    mean-label: "snitt",
  ),
  debug: true,
)

// Worked, equation division (Norwegian notation): sum filled in, then
// `536 : 4 = 134` with the working (minus signs, rules, brought-down
// digits) under the dividend, and the mean in the answer slot. The
// answer slot sits under the division, whose result it is.
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
    body-height: 6.5cm,
    scaffold: true,
    division-style: "equation",
    div-operator: [#sym.colon],
    decimal-sep: ",",
    mean-label: "snitt",
  ),
  debug: true,
)

// Three values instead of four: the stack is one row shorter and the
// divisor is 3. A page mixes both counts, so this shape has to hold its
// own bounding rect independent of the 4-value one.
#set page(width: auto, height: auto, margin: 0.5em)
#import "/lib/problems/statistics/mean.typ": mean-problem

#mean-problem(
  (38, 66, 58, 162, 54),
  mode: "worked",
  opts: (
    operator: [#sym.plus],
    stack-width: 2.25cm,
    div-width: 3.0cm,
    div-rows: 6,
    data-height: 1.4cm,
    body-height: 9cm,
    scaffold: true,
  ),
  debug: true,
)

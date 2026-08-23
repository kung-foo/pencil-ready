// Blank, scaffold on: data line + pre-filled addition stack. The
// division half is hidden but still reserves its full rect — the empty
// area to the right of the stack is the student's work space, and it is
// exactly the size the worked solution occupies.
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

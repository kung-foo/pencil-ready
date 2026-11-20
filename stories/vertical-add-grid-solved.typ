#import "/lib/problems/addition/basic.typ": addition-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 3-digit addition with the answer grid, worked: one row, 4 columns
// (3 digits + 1 for the carry out of the top column).
#addition-basic-problem((903, 542, 1445), mode: "worked", opts: (operator: [#sym.plus], width: 2.25cm, grid: true, grid-cols: 4), debug: true)

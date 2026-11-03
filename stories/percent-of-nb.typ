#import "/lib/problems/percent/of.typ": percent-of-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved, Norwegian, with the working: 48 · 0,37 = 17,76
#percent-of-problem((37, 48, 1776, 2), mode: "worked", opts: (operator: [#sym.dot.op], percent-suffix: "\u{a0}%", of: "av", decimal-sep: ","), debug: true)

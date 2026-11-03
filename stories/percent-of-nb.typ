#import "/lib/problems/percent/of.typ": percent-of-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved, Norwegian, decimal answer, with the working: 48 · 37 = 1776, 1776 : 100 = 17,76
#percent-of-problem((37, 48, 1776, 2), mode: "worked", opts: (operator: [#sym.dot.op], div-operator: [#sym.colon], percent-suffix: "\u{a0}%", of: "av", decimal-sep: ","), debug: true)

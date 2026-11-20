#import "/lib/problems/subtraction/basic.typ": subtraction-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 4-digit − 3-digit subtraction with the answer grid: one row, as wide
// as the minuend (the difference is never wider).
#subtraction-basic-problem((9031, 542, 8489), opts: (operator: [#sym.minus], width: 2.8cm, grid: true, grid-cols: 4), debug: true)

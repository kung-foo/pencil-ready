#import "/lib/problems/division/equation.typ": division-equation-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Equation layout with a zero quotient digit: 816 : 4 = 204. The step where 4 doesn't fit in 1 still writes -0 and brings the next digit down.
#division-equation-problem((816, 4, 204), mode: "worked", opts: (operator: [#sym.colon]), debug: true)

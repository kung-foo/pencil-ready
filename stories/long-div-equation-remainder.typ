#import "/lib/problems/division/equation.typ": division-equation-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Equation layout with a remainder and a two-digit first chunk: 758 : 9 = 84 rest 2 (9 doesn't fit in 7, so the working starts at 75).
#division-equation-problem((758, 9, 84), mode: "worked", opts: (operator: [#sym.colon], reserve-remainder: true, remainder-prefix: "rest\u{a0}"), debug: true)

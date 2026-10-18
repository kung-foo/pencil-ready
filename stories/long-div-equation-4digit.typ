#import "/lib/problems/division/equation.typ": division-equation-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Equation layout, 4-digit dividend with remainder: 9825 : 6 = 1637 rest 3 (the matematikk.org example).
#division-equation-problem((9825, 6, 1637), mode: "worked", opts: (operator: [#sym.colon], reserve-remainder: true, remainder-prefix: "rest\u{a0}"), debug: true)

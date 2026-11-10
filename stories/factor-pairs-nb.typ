#import "/lib/problems/factors/pairs.typ": factor-pairs-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Norwegian multiply sign; a three-digit number with a square pair.
#factor-pairs-problem((100,), mode: "worked", opts: (operator: [#sym.dot.op]), debug: true)

#import "/lib/problems/factors/prime.typ": prime-factors-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Norwegian: 100 (three digits) with the · sign.
#prime-factors-problem((100,), mode: "worked", opts: (operator: [#sym.dot.op]), debug: true)

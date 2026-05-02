#import "/lib/problems/algebra/one-step.typ": algebra-one-step-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Worked example: 5 · x = 30, x = 6.
#algebra-one-step-problem((2, 5, 6, 30), mode: "worked", opts: (mult-operator: [#sym.dot.op], div-operator: [#sym.div]), debug: true)

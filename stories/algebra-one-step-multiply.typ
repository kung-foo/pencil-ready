#import "/lib/problems/algebra/one-step.typ": algebra-one-step-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Multiplication form: 5 · x = 30 (form 2, x = 6).
#algebra-one-step-problem((2, 5, 6, 30), opts: (mult-operator: [#sym.dot.op], div-operator: [#sym.div]), debug: true)

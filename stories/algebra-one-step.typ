#import "/lib/problems/algebra/one-step.typ": algebra-one-step-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Blank one-step problem, addition form: x + 7 = 12.
// data = (form, p, x-val, c) — form 0 is `x + b = c`.
#algebra-one-step-problem((0, 7, 5, 12), opts: (mult-operator: [#sym.dot.op], div-operator: [#sym.div]))

#import "/lib/problems/algebra/two-step.typ": algebra-two-step-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Canonical form, explicit × operator, 11×x to exercise multi-digit coefficient.
// numbers = (a, b, x, c, form)
#algebra-two-step-problem((11, 25, 17, 212, 0), opts: (operator: [#sym.times]), debug: true)

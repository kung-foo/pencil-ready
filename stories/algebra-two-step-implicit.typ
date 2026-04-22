#import "/lib/problems/algebra-two-step.typ": algebra-two-step-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Implicit coefficient-variable juxtaposition (`4x` instead of `4 × x`).
#algebra-two-step-problem((4, 5, 4, 21, 0), opts: (operator: [#sym.times], implicit: true))

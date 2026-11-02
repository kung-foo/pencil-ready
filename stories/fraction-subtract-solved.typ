#import "/lib/problems/fraction/subtraction.typ": fraction-subtraction-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked — multiple denominators, reduces: 5/6 − 1/3 = 5/6 − 2/6 = 3/6 = 1/2
#fraction-subtraction-problem((5, 6, 1, 3, 0, 1, 2), mode: "worked", opts: (operator: [#sym.minus], subtract: true), debug: true)

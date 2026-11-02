#import "/lib/problems/fraction/addition.typ": fraction-addition-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked — same denominator, two rows: 3/8 + 1/8 = 4/8 = 1/2
#fraction-addition-problem((3, 8, 1, 8, 0, 1, 2), mode: "worked", opts: (operator: [#sym.plus], rows: 2), debug: true)

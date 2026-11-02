#import "/lib/problems/fraction/addition.typ": fraction-addition-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Blank — unrelated denominators: 1/2 + 1/3, three reserved rows
#fraction-addition-problem((1, 2, 1, 3, 0, 5, 6), opts: (operator: [#sym.plus]), debug: true)

#import "/lib/problems/fraction/addition.typ": fraction-addition-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked — rewrite, combine, then improper → mixed: 2/3 + 3/4 = 8/12 + 9/12 = 17/12 = 1 5/12
#fraction-addition-problem((2, 3, 3, 4, 1, 5, 12), mode: "worked", opts: (operator: [#sym.plus]), debug: true)

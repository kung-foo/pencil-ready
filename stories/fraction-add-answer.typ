#import "/lib/problems/fraction/addition.typ": fraction-addition-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Answer-only — final answer on its own row, steps hidden: 1/4 + 1/6 = 5/12
#fraction-addition-problem((1, 4, 1, 6, 0, 5, 12), mode: "answer-only", opts: (operator: [#sym.plus]), debug: true)

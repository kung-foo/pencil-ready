#import "/lib/problems/algebra/one-step.typ": algebra-one-step-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Answer-key rendering — same as worked for one-step (no intermediate
// row to suppress). Answer page lines up cell-for-cell with the problem
// page.
#algebra-one-step-problem((2, 5, 6, 30), mode: "answer-only", opts: (mult-operator: [#sym.dot.op], div-operator: [#sym.div]), debug: true)

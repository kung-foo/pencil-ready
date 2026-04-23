#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 3-digit × 3-digit needs 4 answer rows: 3 partial products + 1 final sum.
// Last element (100163) is the product — the component's answer.
#multiplication-basic-problem((287, 349, 100163), opts: (operator: [#sym.times], width: 3.4em, answer-rows: 4))

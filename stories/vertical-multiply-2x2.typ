#import "/lib/problems/vertical.typ": vertical-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit needs 3 answer rows: 2 partial products + 1 final sum.
// Last element (1813) is the product — the component's answer.
#vertical-problem((37, 49, 1813), opts: (operator: [#sym.times], answer-rows: 3))

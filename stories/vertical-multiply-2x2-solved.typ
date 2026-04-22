#import "/lib/problems/vertical.typ": vertical-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit, solved with partial products + final sum.
// Last element (1813) is the product.
#vertical-problem((37, 49, 1813), mode: "worked", opts: (operator: [#sym.times], answer-rows: 3))

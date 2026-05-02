#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit, solved with partial products + final sum.
// Last element (1813) is the product.
#multiplication-basic-problem((37, 49, 1813), mode: "worked", opts: (operator: [#sym.times], answer-rows: 3), debug: true)

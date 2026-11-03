#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked decimal multiplication, the written method: 48 × 0,37 = 17,76.
// Partial products are whole numbers; the point is placed in the product.
#multiplication-basic-problem((48, 37, 1776), mode: "worked", opts: (operator: [#sym.dot.op], width: 3.8em, answer-rows: 3, decimal-places: (0, 2, 2), decimal-sep: ","), debug: true)
#multiplication-basic-problem((11, 5, 55), mode: "worked", opts: (operator: [#sym.times], width: 3.8em, answer-rows: 1, decimal-places: (0, 2, 2)), debug: true)

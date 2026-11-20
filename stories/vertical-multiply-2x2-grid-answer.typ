#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit with the answer grid, answer key: just the product
// in the first row; the grid keeps the blank cell's size.
#multiplication-basic-problem((37, 49, 1813), mode: "answer-only", opts: (operator: [#sym.times], answer-rows: 3, grid: true, grid-cols: 5), debug: true)

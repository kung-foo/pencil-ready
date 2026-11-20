#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 3-digit × 3-digit with the answer grid, worked: 7 columns (6 product
// digits plus `+`), 4 rows (3 partial products + sum).
#multiplication-basic-problem((287, 349, 100163), mode: "worked", opts: (operator: [#sym.times], answer-rows: 4, grid: true, grid-cols: 7), debug: true)

#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit with the answer grid, worked: one digit per cell,
// `+` before the last partial product, rule above the sum.
#multiplication-basic-problem((37, 49, 1813), mode: "worked", opts: (operator: [#sym.times], answer-rows: 3, grid: true, grid-cols: 5), debug: true)

#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit with the answer grid, answer key: no grid, just the
// product in the normal answer style; the box keeps the grid cell's size
// so answer pages line up with the problem pages.
#multiplication-basic-problem((37, 49, 1813), mode: "answer-only", opts: (operator: [#sym.times], answer-rows: 3, grid: true, grid-cols: 5), debug: true)

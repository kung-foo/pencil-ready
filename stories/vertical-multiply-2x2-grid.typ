#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit with the answer grid: 5 columns (4 product digits
// plus the `+` column), 3 rows (2 partial products + sum). The box widens
// to hold the grid; `grid-cols` comes from the sheet in production.
#multiplication-basic-problem((37, 49, 1813), opts: (operator: [#sym.times], answer-rows: 3, grid: true, grid-cols: 5), debug: true)

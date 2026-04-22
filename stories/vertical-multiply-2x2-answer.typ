#import "/lib/problems/vertical.typ": vertical-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit, answer-key rendering: just the final product, no
// partial products. Answer-rows stays at 3 so the box has the same height
// as a blank (student-facing) cell — keeps the grid aligned between
// problem and answer pages.
#vertical-problem((37, 49, 1813), mode: "answer-only", opts: (operator: [#sym.times], answer-rows: 3))

#import "/lib/problems/vertical.typ": vertical-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 2-digit × 2-digit needs 3 answer rows: 2 partial products + 1 final sum.
#vertical-problem((37, 49), [#sym.times], answer-rows: 3)

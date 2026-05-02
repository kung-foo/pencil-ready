#import "/lib/problems/decimal/multiply.typ": decimal-multiply-problem
#set page(width: auto, height: auto, margin: 0.5em)

// 2.5 × 3 = 7.5 — top has dp=1, bottom is whole (dp=0), answer dp=1.
#decimal-multiply-problem(
  (25, 3, 75),
  opts: (operator: [#sym.times], decimal-places: (1, 0, 1)),
  debug: true,
)

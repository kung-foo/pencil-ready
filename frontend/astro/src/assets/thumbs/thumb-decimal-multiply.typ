// Thumbnail: decimal multiplication (2.5 × 3 = 7.5).
#import "/lib/thumb-page.typ": thumb-answer-style, thumb-page
#import "/lib/problems/decimal/multiply.typ": decimal-multiply-problem

#show: thumb-page

#decimal-multiply-problem(
  (25, 3, 75),
  mode: "worked",
  opts: (operator: [#sym.times], decimal-places: (1, 0, 1), ..thumb-answer-style),
)

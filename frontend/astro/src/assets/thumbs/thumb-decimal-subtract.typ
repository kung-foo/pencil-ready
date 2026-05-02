// Thumbnail: decimal subtraction (4.56 − 1.23 = 3.33).
#import "/lib/thumb-page.typ": thumb-answer-style, thumb-page
#import "/lib/problems/decimal/subtract.typ": decimal-subtract-problem

#show: thumb-page

#decimal-subtract-problem(
  (456, 123, 333),
  mode: "worked",
  opts: (operator: [#sym.minus], decimal-places: (2, 2, 2), ..thumb-answer-style),
)

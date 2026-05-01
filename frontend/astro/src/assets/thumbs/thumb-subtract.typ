// Thumbnail: subtraction with a borrow case (53 − 29 = 24).
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/subtraction/basic.typ": subtraction-basic-problem

#show: thumb-page

#subtraction-basic-problem(
  (53, 29, 24),
  mode: "worked",
  opts: (operator: [#sym.minus], ..thumb-answer-style),
)

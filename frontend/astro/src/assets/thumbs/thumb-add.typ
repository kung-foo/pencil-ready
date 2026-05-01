// Thumbnail: addition with a carry case (47 + 28 = 75).
#import "/lib/thumb-page.typ": thumb-answer-style, thumb-page
#import "/lib/problems/addition/basic.typ": addition-basic-problem

#show: thumb-page

#addition-basic-problem(
  (42, 25, 67),
  mode: "worked",
  opts: (operator: [#sym.plus], ..thumb-answer-style),
)

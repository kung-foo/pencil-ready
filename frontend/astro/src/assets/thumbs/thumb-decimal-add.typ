// Thumbnail: decimal addition (1.23 + 4.56 = 5.79).
#import "/lib/thumb-page.typ": thumb-answer-style, thumb-page
#import "/lib/problems/decimal/add.typ": decimal-add-problem

#show: thumb-page

#decimal-add-problem(
  (123, 456, 579),
  mode: "worked",
  opts: (operator: [#sym.plus], decimal-places: (2, 2, 2), ..thumb-answer-style),
)

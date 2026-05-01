// Thumbnail: 2-digit × 1-digit multiply (24 × 3 = 72).
// answer-rows pinned to 1 so the shape stays consistent with the other
// arithmetic thumbs (no partial-products grid for the thumbnail).
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem

#show: thumb-page

#multiplication-basic-problem(
  (24, 3, 72),
  mode: "worked",
  opts: (operator: [#sym.times], answer-rows: 1, ..thumb-answer-style),
)

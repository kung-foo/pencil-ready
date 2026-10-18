// Norwegian variant of thumb-decimal-multiply.typ: same problem in Norwegian notation
// (· and :, decimal comma). Served on /no/ by WorksheetThumb.astro.
// Thumbnail: decimal multiplication (2.5 × 3 = 7.5).
#import "/lib/thumb-page.typ": thumb-answer-style, thumb-page
#import "/lib/problems/decimal/multiply.typ": decimal-multiply-problem

#show: thumb-page

#decimal-multiply-problem(
  (25, 3, 75),
  mode: "worked",
  opts: (operator: [#sym.dot.op], decimal-places: (1, 0, 1), decimal-sep: ",", ..thumb-answer-style),
)

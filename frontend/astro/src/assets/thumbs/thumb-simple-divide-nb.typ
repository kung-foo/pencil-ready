// Norwegian variant of thumb-simple-divide.typ: same problem in Norwegian notation
// (· and :, decimal comma). Served on /no/ by WorksheetThumb.astro.
// Thumbnail: simple division — times-table fact (56 ÷ 7 = 8).
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/division/simple.typ": division-simple-problem

#show: thumb-page

#division-simple-problem(
  (56, 7, 8),
  mode: "worked",
  opts: (operator: [#sym.colon], ..thumb-answer-style),
)

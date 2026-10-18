// Norwegian variant of thumb-simple-divide.typ: the same 56 : 7 = 8 on one
// line, the equation layout Norwegian simple division uses. Served on
// /no/ by WorksheetThumb.astro.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/division/drill.typ": division-drill-problem

#show: thumb-page.with(width: 200pt)

#division-drill-problem(
  (56, 7, 8),
  mode: "worked",
  opts: (operator: [#sym.colon], align: center + horizon, ..thumb-answer-style),
)

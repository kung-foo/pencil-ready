// Norwegian variant of thumb-long-divide.typ: the same 96 : 4 = 24 in the
// equation layout (working under the dividend). Served on /no/ by
// WorksheetThumb.astro.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/division/equation.typ": division-equation-problem

#show: thumb-page.with(width: 130pt)

#division-equation-problem(
  (96, 4, 24),
  mode: "worked",
  opts: (
    operator: [#sym.colon],
    align: center + horizon,
    pad-left: 0pt,
    ..thumb-answer-style,
  ),
)

// Thumbnail: five division-drill problems. Same layout reasoning as
// thumb-mult-drill.typ — see that file for the page-width/gutter
// rationale.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/division/drill.typ": division-drill-problem

#show: thumb-page.with(width: 180pt)

#let drill-opts = (
  operator: [#sym.div],
  align: center + horizon,
  ..thumb-answer-style,
)
#let drill(data) = division-drill-problem(data, mode: "worked", opts: drill-opts)

#grid(
  columns: 1fr,
  rows: (1fr, 1fr, 1fr, 1fr, 1fr),
  drill((21, 3, 7)),
  drill((36, 9, 4)),
  drill((48, 8, 6)),
  drill((25, 5, 5)),
  drill((54, 6, 9)),
)

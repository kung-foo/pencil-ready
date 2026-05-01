// Thumbnail: five multiplication-drill problems stacked on an A4
// page. Page is narrower than the typical "fits-3-comfortably" width
// so the problems appear larger when the SVG is scaled to the card
// slot. 1fr × 5 rows distributes the problems evenly across the full
// page height; `align: center + horizon` on each cell centers the
// problem within its row.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/multiplication/drill.typ": multiplication-drill-problem

#show: thumb-page.with(width: 180pt)

#let drill-opts = (
  operator: [#sym.times],
  align: center + horizon,
  ..thumb-answer-style,
)
#let drill(data) = multiplication-drill-problem(data, mode: "worked", opts: drill-opts)

#grid(
  columns: 1fr,
  rows: (1fr, 1fr, 1fr, 1fr, 1fr),
  drill((7, 3, 21)),
  drill((4, 9, 36)),
  drill((6, 8, 48)),
  drill((5, 5, 25)),
  drill((9, 6, 54)),
)

// Thumbnail: one long-division problem with the worked algorithm
// (96 ÷ 4 = 24, two work steps). The bracket + quotient + work-rows
// is the iconic "long division" silhouette at thumbnail scale, so we
// show it filled in rather than blank. Page is wider than the
// arithmetic thumbs to fit the bracket curve + dividend + work grid.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/division/long.typ": division-long-problem

#show: thumb-page.with(width: 130pt)

#division-long-problem(
  (96, 4, 24),
  mode: "worked",
  opts: (
    // `width: auto` lets the box hug the natural content (including
    // the bracket overline now that the component bounds it). With
    // pad-left: 0pt, `align: center + horizon` symmetrically centers
    // the visible content on the page.
    width: auto,
    answer-rows: 4,
    align: center + horizon,
    "pad-left": 0pt,
    ..thumb-answer-style,
  ),
)

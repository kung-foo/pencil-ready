// Thumbnail: factor pairs of 24 (1 × 24, 2 × 12, 3 × 8, 4 × 6), in
// handwriting.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/factors/pairs.typ": factor-pairs-problem

#show: thumb-page.with(width: 150pt)

#align(center + horizon, factor-pairs-problem(
  (24,),
  mode: "worked",
  opts: (rows: 4, align: center + horizon, ..thumb-answer-style),
))

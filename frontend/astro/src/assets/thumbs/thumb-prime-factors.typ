// Thumbnail: the factor tree for 24 (24 → 4 · 6 → 2 · 2 · 2 · 3), and
// the answer, in handwriting.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/factors/prime.typ": prime-factors-problem

#show: thumb-page.with(width: 280pt)

#align(center + horizon, prime-factors-problem(
  (24,),
  mode: "worked",
  opts: (align: center + horizon, ..thumb-answer-style),
))

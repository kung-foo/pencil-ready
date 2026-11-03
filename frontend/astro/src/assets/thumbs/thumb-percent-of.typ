// Thumbnail: one percent-of problem (25% of 84 = 21) with the answer
// handwritten on its line.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/percent/of.typ": percent-of-problem

#show: thumb-page.with(width: 260pt)

#align(center + horizon, percent-of-problem(
  (25, 84, 21, 0),
  mode: "worked",
  opts: (
    align: center + horizon,
    work: false,
    ..thumb-answer-style,
  ),
))

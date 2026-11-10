// Thumbnail: one GCF problem (GCF(12, 18) = 6) with both factor lists,
// the common factors ringed, in handwriting.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/factors/gcf.typ": gcf-problem

#show: thumb-page.with(width: 260pt)

#align(center + horizon, gcf-problem(
  (12, 18, 6),
  mode: "worked",
  opts: (
    align: center + horizon,
    line-width: 8.5em,
    ..thumb-answer-style,
  ),
))

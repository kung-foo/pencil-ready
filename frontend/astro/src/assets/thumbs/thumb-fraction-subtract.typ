// Thumbnail: one fraction-subtraction problem (3/4 − 1/2 = 3/4 − 2/4
// = 1/4) — the rewrite over the common denominator, then the
// difference, in handwriting.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/fraction/subtraction.typ": fraction-subtraction-problem

#show: thumb-page.with(width: 175pt)

#align(center + horizon, fraction-subtraction-problem(
  (3, 4, 1, 2, 0, 1, 4),
  mode: "worked",
  opts: (
    operator: [#sym.minus],
    subtract: true,
    rows: 2,
    symmetric: false,
    row-gutter: 1.8em,
    ..thumb-answer-style,
  ),
))

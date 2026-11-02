// Thumbnail: one fraction-addition problem (1/2 + 1/3 = 3/6 + 2/6 = 5/6)
// — the rewrite over the common denominator, then the sum, in
// handwriting.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/fraction/addition.typ": fraction-addition-problem

#show: thumb-page.with(width: 175pt)

// symmetric: false → the bbox tracks the visible content, so the
// centered thumb is balanced (see thumb-fraction-mult.typ).
#align(center + horizon, fraction-addition-problem(
  (1, 2, 1, 3, 0, 5, 6),
  mode: "worked",
  opts: (
    operator: [#sym.plus],
    rows: 2,
    symmetric: false,
    ..thumb-answer-style,
  ),
))

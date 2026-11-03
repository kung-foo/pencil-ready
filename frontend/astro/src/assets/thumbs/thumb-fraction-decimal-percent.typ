// Thumbnail: one fraction/decimal/percent problem (3/4 = 0.75 = 75%)
// with the decimal and percent handwritten on their lines.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/fraction/conversion.typ": fraction-conversion-problem

#show: thumb-page.with(width: 250pt)

#align(center + horizon, fraction-conversion-problem(
  (3, 4, 0, 75, 2, 75, 0),
  mode: "worked",
  opts: (
    align: center + horizon,
    ..thumb-answer-style,
  ),
))

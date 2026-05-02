// Thumbnail: one fraction-simplification problem (6/8 = 3/4) with
// the simplified form handwritten in the answer slot.
#import "/lib/thumb-page.typ": thumb-page, thumb-answer-style
#import "/lib/problems/fraction/simplification.typ": fraction-simplification-problem

#show: thumb-page.with(width: 95pt)

#fraction-simplification-problem(
  (6, 8),
  mode: "worked",
  opts: (
    align: center + horizon,
    // narrow slot so the visible "3/4" sits near the page center.
    // The default 3.5em is sized for "11 3/4" mixed numbers in
    // blank-mode handwriting; the thumb only renders one short
    // answer and needs the bounding box to track the visible width.
    "slot-width": 1.2em,
    ..thumb-answer-style,
  ),
)

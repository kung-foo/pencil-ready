#import "/lib/problems/fraction/equivalence.typ": fraction-equivalence-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved — answer filled inside the box.
#grid(
  columns: 2,
  column-gutter: 0.5cm,
  row-gutter: 0.3cm,
  fraction-equivalence-problem((1, 3, 2, 6,   0), mode: "worked"),
  fraction-equivalence-problem((1, 3, 2, 6,   1), mode: "worked"),
  fraction-equivalence-problem((1, 3, 2, 6,   2), mode: "worked"),
  fraction-equivalence-problem((1, 3, 2, 6,   3), mode: "worked"),
)

#import "/lib/problems/decimal/add.typ": decimal-add-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Decimal comma: 1,23 + 4,56 = 5,79.
#decimal-add-problem(
  (123, 456, 579),
  mode: "worked",
  opts: (operator: [#sym.plus], decimal-places: (2, 2, 2), decimal-sep: ","),
  debug: true,
)

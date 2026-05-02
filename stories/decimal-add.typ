#import "/lib/problems/decimal/add.typ": decimal-add-problem
#set page(width: auto, height: auto, margin: 0.5em)

// 1.23 + 4.56 = 5.79 — encoded as scaled integers (123, 456, 579) with
// decimal-places=(2,2,2).
#decimal-add-problem(
  (123, 456, 579),
  opts: (operator: [#sym.plus], decimal-places: (2, 2, 2)),
  debug: true,
)

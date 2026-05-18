#import "/lib/problems/expression/order-of-ops.typ": order-of-ops-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Form 2 (parens): (3 + 4) × 5 = 35
#order-of-ops-problem(
  (2, 3, 0, 4, 2, 5, 35),
  opts: (times-op: [#sym.times], divide-op: [#sym.div]),
  debug: true,
)

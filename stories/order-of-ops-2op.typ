#import "/lib/problems/expression/order-of-ops.typ": order-of-ops-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Form 0 (2-op flat): 5 + 6 × 2 = ___
#order-of-ops-problem(
  (0, 5, 0, 6, 2, 2, 17),
  opts: (times-op: [#sym.times], divide-op: [#sym.div]),
  debug: true,
)

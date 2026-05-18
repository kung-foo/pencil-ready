#import "/lib/problems/expression/order-of-ops.typ": order-of-ops-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Form 3 (parens around RHS): 4 × (3 + 2) = 20
#order-of-ops-problem(
  (3, 4, 2, 3, 0, 2, 20),
  opts: (times-op: [#sym.times], divide-op: [#sym.div]),
  debug: true,
)

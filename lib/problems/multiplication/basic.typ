// Multi-digit multiplication — stacked operands with partial-products
// machinery in the solve space below. Thin wrapper around the
// `vertical-stack` layout primitive (which has the partial-products
// grid built in when `answer-rows > 1`).
#import "/lib/problems/_layouts/vertical-stack.typ": vertical-stack-problem as multiplication-basic-problem

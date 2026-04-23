// Subtraction — multi-digit stacked operands with a difference line below.
// Thin wrapper around the `vertical-stack` layout primitive; the
// worksheet's generator passes `operator: [#sym.minus]` via opts.
#import "/lib/problems/_layouts/vertical-stack.typ": vertical-stack-problem as subtraction-basic-problem

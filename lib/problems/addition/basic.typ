// Addition — multi-digit stacked operands with a sum line below.
// Thin wrapper around the `vertical-stack` layout primitive; the
// worksheet's generator passes `operator: [#sym.plus]` via opts.
#import "/lib/problems/_layouts/vertical-stack.typ": vertical-stack-problem as addition-basic-problem

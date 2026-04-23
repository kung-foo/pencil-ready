// Simple division — stacked dividend/divisor with a quotient line
// below. Thin wrapper around the `vertical-stack` layout primitive;
// the worksheet's generator passes `operator: [#sym.div]` via opts.
#import "/lib/problems/_layouts/vertical-stack.typ": vertical-stack-problem as division-simple-problem

#import "/lib/problems/fraction/multiplication.typ": fraction-multiplication-problem
#set page(width: auto, height: auto, margin: 0.5em)

// Two-digit denominator exercises the multi-digit-in-fraction rendering path.
#fraction-multiplication-problem((30, 7, 10), opts: (operator: [#sym.times]), debug: true)

#import "/lib/problems/long-division.typ": long-division-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// 3-digit dividend with full worked steps. numbers = [dividend, divisor, quotient].
#long-division-problem((756, 3, 252), mode: "worked", opts: (answer-rows: 6))

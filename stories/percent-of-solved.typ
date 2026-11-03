#import "/lib/problems/percent/of.typ": percent-of-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved: 99% of 999 = 989.01 — the widest working (two partial products)
#percent-of-problem((99, 999, 98901, 2), mode: "worked", debug: true)

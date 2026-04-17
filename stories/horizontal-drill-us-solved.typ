#import "/lib/problems/horizontal.typ": horizontal-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved drill problem: third element (56) is the pre-computed answer so
// the component doesn't have to know whether this is × or ÷.
#horizontal-problem((7, 8, 56), [#sym.times], solved: true)

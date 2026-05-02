#import "/lib/problems/multiplication/drill.typ": multiplication-drill-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Solved drill problem: third element (56) is the pre-computed answer so
// the component doesn't have to know whether this is × or ÷.
#multiplication-drill-problem((7, 8, 56), mode: "worked", opts: (operator: [#sym.times]), debug: true)

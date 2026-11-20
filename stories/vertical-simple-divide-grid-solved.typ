#import "/lib/problems/division/simple.typ": division-simple-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Stacked simple division with the answer grid, worked: the quotient
// under the dividend's columns.
#division-simple-problem((45, 5, 9), mode: "worked", opts: (operator: [#sym.div], width: 2.2cm, grid: true, grid-cols: 2), debug: true)

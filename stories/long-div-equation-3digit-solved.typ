#import "/lib/problems/division/equation.typ": division-equation-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Equation layout, worked: 852 : 4 = 213. Working under the dividend with minus signs and rules; first step subtracts from a single digit.
#division-equation-problem((852, 4, 213), mode: "worked", opts: (operator: [#sym.colon]), debug: true)

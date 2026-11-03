#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Worked decimal multiplication, the written method: multiply as whole
// numbers, then place the point in the product. The half-width point gets a
// spacer at the same position in every row, so columns stay aligned.
#multiplication-basic-problem((48, 37, 1776), mode: "worked", opts: (operator: [#sym.dot.op], width: 3.8em, answer-rows: 3, decimal-places: (0, 2, 2), decimal-sep: ","), debug: true)
#multiplication-basic-problem((81, 33, 2673), mode: "worked", opts: (operator: [#sym.times], width: 3.8em, answer-rows: 3, decimal-places: (0, 2, 2)), debug: true)
#multiplication-basic-problem((11, 5, 55), mode: "worked", opts: (operator: [#sym.times], width: 3.8em, answer-rows: 1, decimal-places: (0, 2, 2)), debug: true)

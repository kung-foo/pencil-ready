#import "/lib/problems/vertical.typ": vertical-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Last element (100) is the sum — the component treats numbers.last()
// as the answer. Operands are the preceding 21 + 34 + 45.
#vertical-problem((21, 34, 45, 100), [#sym.plus])

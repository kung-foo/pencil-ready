#import "/lib/problems/addition/basic.typ": addition-basic-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Binary addition: operands are stored as their base-2 digits interpreted
// as decimal (e.g. 0b1011 → u32 1011). pad-width left-pads operands with
// zeros so column alignment is preserved even for values like 0b0110
// (which would otherwise render as "110", losing a column).
//
// Example: 0b1010 + 0b0110 = 0b10000 (decimal 10 + 6 = 16). Wider width
// than the default since 4-bit operands + operator won't fit in 2.8em.
#addition-basic-problem((1010, 110, 10000), mode: "worked", opts: (operator: [#sym.plus], width: 4.5em, pad-width: 4), debug: true)

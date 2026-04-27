#import "/lib/problems/algebra/one-step.typ": algebra-one-step-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Division form: x ÷ 6 = 4 (form 3, x = 24). Locale-respecting glyph
// (US ÷ here; Norwegian renders : via the generator).
#algebra-one-step-problem((3, 6, 24, 4), opts: (mult-operator: [#sym.dot.op], div-operator: [#sym.div]))

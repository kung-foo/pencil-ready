#import "/lib/problems/shared.typ": problem-font, problem-text-size-horizontal, problem-features
#set page(width: auto, height: auto, margin: 0.5cm)

// Spike: can we embed a blank box inside Typst math-mode fractions?
// Tests four missing-slot variants using the same font overrides as other
// fraction components.

#set text(font: problem-font, size: problem-text-size-horizontal, features: problem-features)
#show math.equation: set text(font: "Fira Math", features: ())

#let blank = box(width: 2.0em, height: 1.6em, stroke: 0.8pt)

#let equiv-problem(left-num, left-den, right-num, right-den, missing, solved: false) = {
  // missing: "left-num" | "left-den" | "right-num" | "right-den"
  let slot(val, slot-name) = if missing == slot-name {
    box(width: 2.0em, height: 1.6em, stroke: 0.8pt, align(center + horizon, if solved { str(val) }))
  } else { str(val) }
  let ln = slot(left-num,  "left-num")
  let ld = slot(left-den,  "left-den")
  let rn = slot(right-num, "right-num")
  let rd = slot(right-den, "right-den")

  box(inset: (x: 0.4em, y: 0.3em), $#ln / #ld = #rn / #rd$)
}

// Blank variants
#equiv-problem(1, 3, 2, 6,   "right-den")
#h(1em)
#equiv-problem(4, 8, 8, 16,  "right-num")
#h(1em)
#equiv-problem(2, 2, 4, 4,   "left-num")
#h(1em)
#equiv-problem(1, 3, 2, 6,   "left-den")

#v(0.5em)

// Solved variants
#equiv-problem(1, 3, 2, 6,   "right-den", solved: true)
#h(1em)
#equiv-problem(4, 8, 8, 16,  "right-num", solved: true)
#h(1em)
#equiv-problem(2, 2, 4, 4,   "left-num",  solved: true)
#h(1em)
#equiv-problem(1, 3, 2, 6,   "left-den",  solved: true)

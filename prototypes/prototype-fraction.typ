// Standalone prototype: comparing typst's built-in math.frac vs a custom
// rolled fraction component. Copy-paste this whole file into the typst
// playground (https://typst.app) — it uses only system fonts so it works
// anywhere.
//
// If you want to use the Pencil Ready fonts locally, run with:
//   typst compile --font-path fonts/ stories/prototype-fraction.typ output/frac.png --ppi 300

#set page(width: 18cm, height: auto, margin: 1cm)
#set text(font: "B612 Mono", size: 14pt)

= Approach 1: typst's built-in `math.frac` (default math font)

Uses typst math mode. The `/` in math mode auto-creates a stacked fraction.
Default math font is New Computer Modern (Latin Modern-like).

#set text(size: 22pt)
$ 8 times 1/2 = space.quad square $

$ 20 times 4/5 = space.quad square $

$ 9 times 2/3 = space.quad square $

#set text(size: 14pt)

= Approach 1b: `math.frac` with Fira Math

Fira Math is a sans-serif math font that pairs well with Fira Sans / B612.
Has a proper OpenType MATH table — no "not designed for math" warning.

#set text(size: 22pt)
#show math.equation: set text(font: "Fira Math")

*US symbols:*

$ 8 times 1/2 = space.quad square $

$ 20 times 4/5 = space.quad square $

*Norwegian `dot.c` for multiply:*

$ 8 dot.c 1/2 = space.quad square $

$ 20 dot.c 4/5 = space.quad square $

*Norwegian `colon` for divide (and `:` literal):*

$ 12 colon 3 = space.quad square $   \   // `colon` symbol name
$ 12 : 3 = space.quad square $   \        // literal `:` in math mode
$ 12 div 3 = space.quad square $          // default `÷`

// reset math font for the rest of the file
#show math.equation: set text(font: "New Computer Modern Math")
#set text(size: 14pt)

= Approach 2: custom rolled fraction

A small typst function that stacks num/denom with a line between.
Uses whatever font is set in the surrounding text context.

// Numerator/denominator are rendered at 80% of surrounding text size
// (standard math typography convention: fraction digits smaller than body).
#let frac(n, d) = context {
  let n-content = text(size: 0.8em, str(n))
  let d-content = text(size: 0.8em, str(d))
  let w = calc.max(measure(n-content).width, measure(d-content).width) + 3pt
  box(baseline: 0.5em, {
    stack(
      dir: ttb,
      spacing: 0.1em,
      align(center + horizon, box(width: w, align(center, n-content))),
      line(length: w, stroke: 0.5pt),
      align(center + horizon, box(width: w, align(center, d-content))),
    )
  })
}

#set text(size: 22pt)

// Try different fonts by uncommenting:
// #set text(font: "Liberation Mono")
// #set text(font: "DejaVu Sans Mono")

8 #sym.times #frac(1, 2) = #h(0.4em) #box(width: 2em, stroke: (bottom: 0.5pt), height: 1em)

#v(0.4cm)

20 #sym.times #frac(4, 5) = #h(0.4em) #box(width: 2em, stroke: (bottom: 0.5pt), height: 1em)

#v(0.4cm)

9 #sym.times #frac(2, 3) = #h(0.4em) #box(width: 2em, stroke: (bottom: 0.5pt), height: 1em)

#set text(size: 14pt)

= Approach 3: math.frac with custom font override

Forces the math equation to use the same font as body text.

#set text(size: 22pt)
#show math.equation: set text(font: "B612 Mono")

$ 8 times 1/2 = space.quad square $

$ 20 times 4/5 = space.quad square $

$ 9 times 2/3 = space.quad square $

#set text(size: 14pt)

= Observations to look for

- *Bar thickness and length* — does the bar extend past the digits?
- *Vertical positioning* — does the fraction baseline align with the whole number?
- *Digit font* — do the fraction digits match the whole number's font?
- *Spacing around the `×`* — does math mode add extra space?
- *Answer blank alignment* — does the blank line sit at the right height?

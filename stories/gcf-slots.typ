#import "/lib/problems/factors/gcf.typ": gcf-problem
#set page(width: auto, height: auto, margin: 0.3cm)

// Slots: a blank per factor, so the count is a hint. Blank, worked, and
// the twelve-factor case (60, 96) that sizes the blanks.
#stack(spacing: 0.5em,
  gcf-problem((12, 18, 6), opts: (slots: true), debug: true),
  gcf-problem((12, 18, 6), mode: "worked", opts: (slots: true), debug: true),
  gcf-problem((60, 96, 12), mode: "worked", opts: (slots: true), debug: true),
)

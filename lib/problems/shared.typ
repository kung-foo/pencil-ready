// Shared constants for all problem layouts.

// Page body / label font — used for headers, footers, labels, and the
// default text font. Problem digits use `problem-font`.
#let body-font = "B612"
#let problem-font = "Fira Code"
#let operator-font = "Fira Math"
#let problem-text-size = 22pt
#let problem-text-size-horizontal = 18pt
#let problem-tracking = 2pt
// Fira Code OpenType features: cv11 = plain zero (no slash, no dot).
#let problem-features = (cv11: 1)
// One typeset line at problem text size (font em + leading). Used as the
// stack-spacing unit for both vertical operand stacks and step-by-step
// equation rows.
#let problem-line-height = 1.3em

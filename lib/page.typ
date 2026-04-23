// Per-page emission macro — wraps the PDF-outline heading and the
// `worksheet-grid` call that would otherwise be emitted as two separate
// typst blocks from Rust. One `#worksheet-page(...)` call per page
// keeps `document.rs`'s emission symmetric and gives the typst side a
// clear public entry point for "render one page of a worksheet".
//
// The page chrome (header, footer) is NOT handled here — it lives in
// `page.header` / `page.footer` callbacks set in the document
// preamble, so the body of each page is just what `worksheet-page`
// produces.

#import "/lib/layout.typ": worksheet-grid

#let worksheet-page(
  problems,
  component,
  cols: 4,
  modes: none,
  opts: (:),
  debug: false,
  // Outline-entry key, used to seed the PDF sidebar bookmarks when
  // --include-answers is on. "" = no heading; "problems" / "answer-key"
  // emit a level-1 heading that the document-preamble show-rule
  // visually suppresses.
  outline: "",
) = {
  if outline == "problems" {
    heading(outlined: true, bookmarked: true, level: 1)[Problems]
  } else if outline == "answer-key" {
    heading(outlined: true, bookmarked: true, level: 1)[Answer Key]
  }
  worksheet-grid(
    problems,
    component,
    num-cols: cols,
    debug: debug,
    modes: modes,
    opts: opts,
  )
}

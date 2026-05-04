#import "/lib/zebra/lib.typ": qrcode

// Page footer. The text body sits centred in a fixed 0.8cm box (left
// to its baseline so existing visual stories stay pixel-stable). When
// `share-url` is set, a 1.4cm QR is `place()`d in the bottom-right
// page corner with equal margins on both sides:
//   dy: +0.7cm puts the QR's bottom at FOOTER_DESCENT_CM (0.4cm) above
//     the page edge — that's box_bottom (page_bottom − descent − pad =
//     page_bottom − 1.1cm) + 0.7cm = page_bottom − 0.4cm.
//   dx: +1.1cm pulls the QR's right edge inward by the same 0.4cm
//     from the page edge. Box right is at MARGINS_CM.right (1.5cm)
//     from the page edge; subtracting the desired 0.4cm right margin
//     gives the dx offset 1.5 − 0.4 = 1.1cm.
// Equal margins keep the QR visually anchored to the page corner
// rather than to the body grid's right gutter.
#let worksheet-footer(body, share-url: none, debug: false) = {
  let debug-box = if debug { 1pt + red } else { none }
  set text(size: 8pt, fill: rgb("#999999"))
  box(height: 0.8cm, width: 100%, stroke: debug-box, {
    if share-url != none {
      place(bottom + right, dx: 1.1cm, dy: 0.7cm,
        qrcode(share-url, width: 1.4cm))
    }
    align(center + horizon, body)
  })
}

// Canonical footer content used by every generated worksheet. The baseline
// nudges on the heart and the flag pull those glyphs up into the x-height
// of the surrounding text — without them, both sit noticeably lower than
// the letters (Noto Color Emoji's baseline is near the bottom of each
// glyph, which clashes with the body text's metrics).
#let pencil-ready-content = [
  *Pencil Ready* — made with #box(height: 1.2em, baseline: 20%, image("/assets/rainbow-heart.svg")) in Oslo, #box(baseline: -20%)[🇳🇴] — #link("https://pencilready.com")[pencilready.com] — #link("https://creativecommons.org/licenses/by/4.0/")[CC BY 4.0]
]

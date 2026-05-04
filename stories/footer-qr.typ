#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font

// Mirrors how `worksheet-footer` is invoked in production: as the
// page-footer callback wrapped in `pad(bottom: 0.7cm, ...)`. Setting
// `share-url` exercises the QR placement path. Page is A4-shaped but
// short, so the rendered baseline only captures the bottom-right
// corner where the QR lands; the body area above is empty content.
#set page(
  width: 21cm,
  height: 4cm,
  margin: (left: 1.5cm, right: 1.5cm, top: 0pt, bottom: 2.2cm),
  footer-descent: 0.4cm,
  footer: pad(bottom: 0.7cm, worksheet-footer(
    pencil-ready-content,
    share-url: "https://pencilready.com/worksheets/add/?level=basic&seed=42",
    debug: true,
  )),
)
#set text(font: body-font)

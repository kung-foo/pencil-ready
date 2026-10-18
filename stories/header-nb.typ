#import "/lib/header.typ": worksheet-header
#import "/lib/problems/shared.typ": body-font
#set page(width: 18cm, height: auto, margin: 0.3cm)
#set text(font: body-font, lang: "nb")

// Norwegian chrome: localized Name/Date labels plus a title and a
// two-line instruction — the longest drafted nb instruction (mean),
// to catch a third-line wrap.
#worksheet-header(
  title: "Gjennomsnitt",
  instructions: "Finn gjennomsnittet av hver tallrekke: legg sammen tallene, og del på hvor mange tall det er.",
  name-label: "Navn",
  date-label: "Dato",
  debug: true,
)

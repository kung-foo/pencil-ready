# Norwegian (Bokmål) translation — pending native review

Every Norwegian string on the site and in the worksheets was drafted
without a native speaker's review. It aims for natural primary-school
Bokmål (addisjon, subtraksjon, multiplikasjon, divisjon, brøk,
desimaltall, likninger, mente, låne). Nothing here blocks shipping, but
please read through and correct anything that sounds off.

"Pencil Ready" and the motto "print → solve → sharpen → repeat" stay in
English on purpose, in every locale.

## Files

| Where | What |
|---|---|
| `src/i18n/ui/nb.ts` | All UI strings: page titles/descriptions, header, footer, homepage, worksheet page, 404, About headings, language selector, configurator labels, level names and examples |
| `src/content/worksheets/nb/*.md` | Help text for all 18 worksheets (title, summary, prerequisites, learning goals) |
| `src/components/about/AboutNb.astro` | About page prose |
| `crates/core/src/meta.rs` (`title_nb`, `instructions_nb`) | Worksheet titles and instruction lines printed on the PDFs |
| `crates/core/src/i18n.rs` (`NB`) | PDF chrome: Navn/Dato, Fasit, Oppgaver, footer, PDF metadata, `rest` remainder prefix, `snitt =` mean label |
| `src/assets/thumbs/thumb-*-nb.typ` | Homepage thumbnails in Norwegian notation (no words, just `·`, `:`, decimal comma) |

## Choices worth a second opinion

- **Grade range:** "1st–6th grade" became "1.–7. trinn" (Norwegian
  school starts a year earlier relative to US grades). Check it matches
  the worksheets' actual level.
- **Long division:** titled "Lang divisjon". Some textbooks avoid the
  term. The worksheets still use the US bracket layout. A Norwegian
  `852 : 4 = 213` layout is planned separately.
- **Remainder:** printed as `rest 3` (US: `r=3`).
- **Mean answer slot:** `snitt =` rather than `gjennomsnitt =`, which
  doesn't fit under the addition stack.
- **Drills:** "Gangetrening" / "Divisjonstrening"; homepage section "Pugg".
- **"Seed":** translated literally as "Frø". It's a technical control,
  so a parent may not recognize it in either language.
- **Mastery heading:** "Når dette sitter, kan eleven …", with learning
  goals written as bare infinitives that complete it.
- **Student name placeholder:** "f.eks. Mattehesten" (US: "Math Hippo").
- **Division layout toggle:** "Oppstilling" with the options "På én
  linje" (852 : 4 =), "Med bue" (the US bracket) and, for simple
  division, "Under hverandre" (stacked).
- **Fraction × whole title:** "Brøk ganger heltall" (US: "Fraction
  multiplication"), which describes what the sheet actually drills.

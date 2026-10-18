//! Locale-dependent worksheet copy that isn't per worksheet kind:
//! page-chrome labels, the footer, PDF metadata and the few words
//! problem components print. Titles and instructions live with the
//! other per-kind naming in `meta.rs`.
//!
//! Norwegian strings are drafted, not native-reviewed — see
//! `frontend/astro/TRANSLATION-REVIEW.md`.

use crate::Locale;

/// Every locale-dependent string `document.rs` interpolates into the
/// generated typst source. One `&'static` table per locale so adding a
/// locale is a new table plus a `Locale::strings` arm; a missing field
/// is a compile error.
pub struct LocaleStrings {
    /// Typst `text(lang:)` / BCP-47 language tag. Drives hyphenation and
    /// smart quotes in instruction text.
    pub lang: &'static str,
    pub name_label: &'static str,
    pub date_label: &'static str,
    /// Answer-key page title and its PDF bookmark.
    pub answer_key: &'static str,
    /// PDF bookmark for the problem pages (only emitted with answers).
    pub problems: &'static str,
    /// Footer: "made with ♥ in Oslo" split around the heart image.
    pub footer_made_with: &'static str,
    pub footer_in: &'static str,
    pub pdf_description: &'static str,
    /// Generic PDF keywords; the kind slug and domain are appended.
    pub pdf_keywords: [&'static str; 2],
    /// Decimal separator in rendered numbers.
    pub decimal_sep: &'static str,
    /// Text before the digit in a long-division remainder (`r=3`).
    pub remainder_prefix: &'static str,
    /// Lead-in for the mean worksheet's answer slot (`mean = ___`).
    pub mean_label: &'static str,
}

const EN: LocaleStrings = LocaleStrings {
    lang: "en",
    name_label: "Name",
    date_label: "Date",
    answer_key: "Answer Key",
    problems: "Problems",
    footer_made_with: "made with",
    footer_in: "in",
    pdf_description: "Printable math worksheet — https://pencilready.com — licensed under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)",
    pdf_keywords: ["math", "worksheet"],
    decimal_sep: ".",
    remainder_prefix: "r=",
    mean_label: "mean",
};

const NB: LocaleStrings = LocaleStrings {
    lang: "nb",
    name_label: "Navn",
    date_label: "Dato",
    answer_key: "Fasit",
    problems: "Oppgaver",
    footer_made_with: "laget med",
    footer_in: "i",
    pdf_description: "Matteark til utskrift — https://pencilready.com — lisensiert under CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)",
    pdf_keywords: ["matematikk", "oppgaveark"],
    decimal_sep: ",",
    // No-break space: the slot is sized for one line and "rest" must
    // never wrap away from its digit.
    remainder_prefix: "rest\u{a0}",
    // "gjennomsnitt" is too wide for the slot under the addition stack;
    // "snitt" is the everyday short form.
    mean_label: "snitt",
};

impl Locale {
    pub fn strings(&self) -> &'static LocaleStrings {
        match self {
            Locale::Us => &EN,
            Locale::No => &NB,
        }
    }
}

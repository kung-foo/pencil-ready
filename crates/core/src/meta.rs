//! Human-readable title and filename-safe slug for a worksheet.
//!
//! Kept deliberately minimal: title is the operation name plus "(worked
//! example)" when the worksheet is solve-first; slug is the operation in
//! kebab-case plus `-solved` and `-seed{N}` suffixes so filenames stay
//! unique and deterministic. Digit counts, carry/borrow modes, max
//! quotients, and similar knobs are *not* in the title or slug — they're
//! already in the URL, and surfacing them in filenames buys noise.
//!
//! The only "variant" modifier that affects the name is `binary` for
//! addition, since the page looks fundamentally different.

use crate::{WorksheetParams, WorksheetType};

impl WorksheetType {
    /// Plain-English title. Adds " (worked example)" when `solve_first`
    /// is on.
    pub fn title(&self, solve_first: bool) -> String {
        let mut t = base_name(self, Form::Title).to_string();
        if solve_first {
            t.push_str(" (worked example)");
        }
        t
    }

    /// Kind-only slug, no solved/seed suffixes — suitable for PDF
    /// metadata keywords that shouldn't drift per generation.
    pub fn kind_slug(&self) -> &'static str {
        base_name(self, Form::Slug)
    }
}

impl WorksheetParams {
    /// Plain-English title suitable for page headers or UI labels.
    pub fn title(&self) -> String {
        self.worksheet.title(self.solve_first)
    }

    /// Filename-safe slug. Callers prefix (e.g. "pencil-ready-") and append
    /// the extension.
    pub fn slug(&self) -> String {
        let mut s = base_name(&self.worksheet, Form::Slug).to_string();
        if self.solve_first {
            s.push_str("-solved");
        }
        if let Some(seed) = self.seed {
            s.push_str(&format!("-seed{seed}"));
        }
        s
    }

    /// Kind-only slug, no solved/seed suffixes — suitable for PDF
    /// metadata keywords that shouldn't drift per generation.
    pub fn kind_slug(&self) -> &'static str {
        self.worksheet.kind_slug()
    }
}

#[derive(Clone, Copy)]
enum Form {
    Title,
    Slug,
}

fn base_name(ws: &WorksheetType, form: Form) -> &'static str {
    match (ws, form) {
        (WorksheetType::Add { binary: true, .. }, Form::Title) => "Binary addition",
        (WorksheetType::Add { binary: true, .. }, Form::Slug) => "binary-addition",
        (WorksheetType::Add { .. }, Form::Title) => "Addition",
        (WorksheetType::Add { .. }, Form::Slug) => "addition",
        (WorksheetType::Subtract { .. }, Form::Title) => "Subtraction",
        (WorksheetType::Subtract { .. }, Form::Slug) => "subtraction",
        (WorksheetType::Multiply { .. }, Form::Title) => "Multiplication",
        (WorksheetType::Multiply { .. }, Form::Slug) => "multiplication",
        (WorksheetType::SimpleDivision { .. }, Form::Title) => "Division",
        (WorksheetType::SimpleDivision { .. }, Form::Slug) => "division",
        (WorksheetType::LongDivision { .. }, Form::Title) => "Long division",
        (WorksheetType::LongDivision { .. }, Form::Slug) => "long-division",
        (WorksheetType::MultiplicationDrill { .. }, Form::Title) => "Multiplication drill",
        (WorksheetType::MultiplicationDrill { .. }, Form::Slug) => "multiplication-drill",
        (WorksheetType::DivisionDrill { .. }, Form::Title) => "Division drill",
        (WorksheetType::DivisionDrill { .. }, Form::Slug) => "division-drill",
        (WorksheetType::FractionMultiply { .. }, Form::Title) => "Fraction multiplication",
        (WorksheetType::FractionMultiply { .. }, Form::Slug) => "fraction-multiplication",
        (WorksheetType::FractionSimplify { .. }, Form::Title) => "Fraction simplification",
        (WorksheetType::FractionSimplify { .. }, Form::Slug) => "fraction-simplification",
        (WorksheetType::AlgebraTwoStep { .. }, Form::Title) => "Two-step equations",
        (WorksheetType::AlgebraTwoStep { .. }, Form::Slug) => "two-step-equations",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CarryMode, DigitRange, Locale};

    fn params(ws: WorksheetType) -> WorksheetParams {
        WorksheetParams {
            worksheet: ws,
            num_problems: 12,
            cols: 4,
            paper: crate::Paper::A4,
            debug: false,
            seed: None,
            symbol: None,
            locale: Locale::Us,
            solve_first: false,
            include_answers: false,
            student_name: None,
        }
    }

    #[test]
    fn plain_titles_and_slugs() {
        let p = params(WorksheetType::Add {
            digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
            carry: CarryMode::Any,
            binary: false,
        });
        assert_eq!(p.title(), "Addition");
        assert_eq!(p.slug(), "addition");
    }

    #[test]
    fn binary_addition_is_distinct() {
        let p = params(WorksheetType::Add {
            digits: vec![DigitRange::fixed(4), DigitRange::fixed(4)],
            carry: CarryMode::Any,
            binary: true,
        });
        assert_eq!(p.title(), "Binary addition");
        assert_eq!(p.slug(), "binary-addition");
    }

    #[test]
    fn solve_first_and_seed_suffixes() {
        let mut p = params(WorksheetType::Multiply {
            digits: vec![DigitRange::fixed(3), DigitRange::fixed(2)],
        });
        p.solve_first = true;
        p.seed = Some(42);
        assert_eq!(p.title(), "Multiplication (worked example)");
        assert_eq!(p.slug(), "multiplication-solved-seed42");
    }
}

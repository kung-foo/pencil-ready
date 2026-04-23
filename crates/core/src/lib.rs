//! pencil-ready-core: worksheet generation and typst compilation.
//!
//! Each worksheet type lives in its own module.
//! They share a common template renderer and the typst World implementation.

mod add;
mod algebra_two_step;
mod div_drill;
mod divide;
mod fraction_mult;
mod meta;
mod mult_drill;
mod multiply;
mod subtract;

mod document;
mod world;

pub use world::Fonts;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const MAX_DIGITS: u32 = 5;
const MAX_PROBLEMS: u32 = 16;
const MAX_PROBLEMS_DRILL: u32 = 40;
const MAX_OPERANDS: usize = 4;

/// A digit count that can be a fixed value or a range.
///
/// `DigitRange { min: 2, max: 4 }` means each problem randomly picks
/// 2, 3, or 4 digits for that operand. When min == max, it's fixed.
///
/// In Go this would be a struct with Min/Max fields. Same here.
/// Serialized as its `Display` form ("3" or "2-4") so it round-trips
/// through JSON/query strings the same way the CLI writes it. OpenAPI
/// schema is overridden at the point of use via
/// `#[param(value_type = String)]` to avoid exposing the internal shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DigitRange {
    pub min: u32,
    pub max: u32,
}

impl TryFrom<String> for DigitRange {
    type Error = String;
    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<DigitRange> for String {
    fn from(d: DigitRange) -> String {
        d.to_string()
    }
}

impl DigitRange {
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    pub fn fixed(n: u32) -> Self {
        Self { min: n, max: n }
    }

    /// Pick a random digit count in the range, then generate a random
    /// number with that many digits.
    pub fn random(&self, rng: &mut impl rand::Rng) -> u32 {
        let d = rng.gen_range(self.min..=self.max);
        let lo = 10u32.pow(d - 1);
        let hi = 10u32.pow(d) - 1;
        rng.gen_range(lo..=hi)
    }
}

impl std::fmt::Display for DigitRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.min == self.max {
            write!(f, "{}", self.min)
        } else {
            write!(f, "{}-{}", self.min, self.max)
        }
    }
}

/// Parse "3" or "2-4" into a DigitRange.
impl std::str::FromStr for DigitRange {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some((lo, hi)) = s.split_once('-') {
            let min = lo
                .parse::<u32>()
                .map_err(|e| format!("bad digit range: {e}"))?;
            let max = hi
                .parse::<u32>()
                .map_err(|e| format!("bad digit range: {e}"))?;
            if min > max {
                return Err(format!("invalid range: {min} > {max}"));
            }
            Ok(DigitRange { min, max })
        } else {
            let n = s
                .parse::<u32>()
                .map_err(|e| format!("bad digit count: {e}"))?;
            Ok(DigitRange::fixed(n))
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CarryMode {
    None,
    #[default]
    Any,
    Force,
    Ripple,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BorrowMode {
    None,
    NoAcrossZero,
    #[default]
    Any,
    Force,
    Ripple,
}

#[derive(Debug, Clone)]
pub enum WorksheetType {
    Add {
        digits: Vec<DigitRange>,
        carry: CarryMode,
        /// Binary mode: operand digit counts become **bit** counts, the
        /// operands take values in `[0, 2^d − 1]`, and the rendered
        /// numbers appear in base 2 (with leading-zero padding).
        binary: bool,
    },
    Subtract {
        digits: Vec<DigitRange>,
        borrow: BorrowMode,
    },
    Multiply {
        digits: Vec<DigitRange>,
    },
    SimpleDivision {
        max_quotient: u32,
    },
    LongDivision {
        digits: DigitRange,
        remainder: bool,
    },
    MultiplicationDrill {
        /// Which tables to drill (e.g. [2,3] or [1-10]).
        multiplicand: Vec<DigitRange>,
        /// Range of the other factor (default 1-10).
        multiplier: DigitRange,
    },
    DivisionDrill {
        /// Which divisors to drill (e.g. [2,3] or [1-10]).
        divisor: Vec<DigitRange>,
        /// Range of the quotient (default 1-10).
        max_quotient: DigitRange,
    },
    FractionMultiply {
        /// Allowed denominators (e.g. [2, 3, 4, 5, 10]).
        denominators: Vec<u32>,
        /// Whole-number range (inclusive).
        min_whole: u32,
        max_whole: u32,
        /// If true, numerator is always 1 (unit fractions only).
        unit_only: bool,
    },
    AlgebraTwoStep {
        /// Coefficient range (default 2-12).
        a_range: DigitRange,
        /// Constant range (default 1-30).
        b_range: DigitRange,
        /// Answer range (default 0-20). 0 and 1 are deliberately included.
        x_range: DigitRange,
        /// The variable glyph to solve for. Typically "x", but any string
        /// (letter, symbol, emoji) works as long as a loaded font renders it.
        variable: String,
        /// Implicit coefficient-variable juxtaposition (`4x`) when true,
        /// explicit operator (`4 · x` or `4 × x`) when false.
        implicit: bool,
        /// Randomly mix canonical (`ax + b = c`) and const-first (`b + ax = c`).
        mix_forms: bool,
    },
}

impl WorksheetType {
    /// Name of the typst component function this worksheet renders
    /// with. Matches the wrapper files under `lib/problems/<folder>/`.
    pub(crate) fn component_typst_name(&self) -> &'static str {
        match self {
            WorksheetType::Add { .. } => "addition-basic-problem",
            WorksheetType::Subtract { .. } => "subtraction-basic-problem",
            WorksheetType::Multiply { .. } => "multiplication-basic-problem",
            WorksheetType::SimpleDivision { .. } => "division-simple-problem",
            WorksheetType::LongDivision { .. } => "division-long-problem",
            WorksheetType::MultiplicationDrill { .. } => "multiplication-drill-problem",
            WorksheetType::DivisionDrill { .. } => "division-drill-problem",
            WorksheetType::FractionMultiply { .. } => "fraction-multiplication-problem",
            WorksheetType::AlgebraTwoStep { .. } => "algebra-two-step-problem",
        }
    }

    /// Natural cell rectangle (width, height) in cm. Mirror of the
    /// measured + ceiled values in `crates/core/generated/cell-sizes.toml`
    /// (that file is the source of truth; re-run `cargo run --example
    /// measure_cells` and update this table in the same review).
    ///
    /// `max_digits` is the largest operand digit count across all
    /// problems on a page (or the dividend digit count for long-division,
    /// the whole-number digit count for fraction, the coefficient /
    /// constant magnitude for algebra).
    ///
    /// The rectangle is the same across all render modes — "Cell envelope
    /// rule" (LAYOUT_REFACTOR.md § "Render modes"): `Blank` dominates,
    /// `Worked` fills it in, `AnswerOnly` leaves it empty. This keeps
    /// grid pitch consistent between problem pages and answer-key pages.
    pub fn cell_size_cm(&self, max_digits: u32) -> (f32, f32) {
        match self {
            // Vertical stack, single answer line — width grows with
            // digit count, height fixed at 3.5cm.
            WorksheetType::Add { .. }
            | WorksheetType::Subtract { .. }
            | WorksheetType::SimpleDivision { .. } => (vertical_stack_width(max_digits), 3.5),

            // Vertical stack with partial-products machinery. Height
            // depends on multiplier digit count (one partial row per
            // multiplier digit, plus a final sum row). We key off the
            // smaller of the two operand digit counts — that's the
            // multiplier by convention in the generator.
            WorksheetType::Multiply { digits } => {
                let min_dig = digits.iter().map(|r| r.max).min().unwrap_or(2).max(1);
                let height = match min_dig {
                    1..=2 => 5.5, // 2 partials + sum + padding
                    3 => 6.5,     // 3 partials + sum + padding
                    _ => 7.5,     // 4+ partials; conservative
                };
                (vertical_stack_width(max_digits), height)
            }

            // Long division: unique layout. Height grows with dividend
            // digits (each digit contributes ~2 rows of work space).
            WorksheetType::LongDivision { digits, .. } => match digits.max {
                1..=2 => (3.8, 6.0),
                3 => (3.8, 8.0),
                _ => (4.4, 10.0), // 4+ dividend digits
            },

            // Horizontal drills — same primitive for multiplication and
            // division. d1x1 fits in 4.5cm; d2x1 needs 5.0cm.
            WorksheetType::MultiplicationDrill { .. } | WorksheetType::DivisionDrill { .. } => {
                let w = if max_digits <= 1 { 4.5 } else { 5.0 };
                (w, 1.0)
            }

            // Fraction × whole — width is fixed (the fraction slot
            // dominates). 2-row vertical layout gives 2.8cm height.
            WorksheetType::FractionMultiply { .. } => (6.0, 2.8),

            // Algebra two-step — width grows with coefficient / constant
            // magnitude (more digits in the LHS expression).
            WorksheetType::AlgebraTwoStep {
                a_range,
                b_range,
                x_range,
                ..
            } => {
                let max = a_range.max.max(b_range.max).max(x_range.max);
                let w = if max <= 30 { 7.3 } else { 8.0 };
                (w, 4.1)
            }
        }
    }

    /// Worst-case operand digit count implied by the worksheet config,
    /// computed WITHOUT generating any problems. Feeds `cell_size_cm`
    /// for `Document::validate` — we want an upper bound, not an
    /// exact figure.
    pub fn max_digits_bound(&self) -> u32 {
        match self {
            WorksheetType::Add { digits, .. }
            | WorksheetType::Subtract { digits, .. }
            | WorksheetType::Multiply { digits } => {
                digits.iter().map(|r| r.max).max().unwrap_or(2)
            }
            WorksheetType::LongDivision { digits, .. } => digits.max,
            WorksheetType::MultiplicationDrill { multiplicand, multiplier } => {
                let a = multiplicand.iter().map(|r| r.max).max().unwrap_or(1);
                a.max(multiplier.max)
            }
            WorksheetType::DivisionDrill { divisor, max_quotient } => {
                let a = divisor.iter().map(|r| r.max).max().unwrap_or(1);
                a.max(max_quotient.max)
            }
            // A simple-division problem is dividend÷divisor = quotient
            // where quotient ≤ max_quotient and divisor is 1-digit.
            // The widest operand is the product (dividend), up to
            // ~9 * max_quotient — at most 3 digits for sane inputs.
            WorksheetType::SimpleDivision { max_quotient } => document::digit_count(9 * *max_quotient),
            // Fraction-mult's width is driven by the whole-number LHS.
            WorksheetType::FractionMultiply { max_whole, .. } => document::digit_count(*max_whole),
            // Algebra's width bound is the largest numeric literal in
            // the LHS / intermediate / solution lines — worst case the
            // a*x+b product or `c` itself.
            WorksheetType::AlgebraTwoStep {
                a_range,
                b_range,
                x_range,
                ..
            } => document::digit_count(
                (a_range.max * x_range.max).max(b_range.max).max(a_range.max),
            ),
        }
    }
}

/// Shared with Add/Subtract/Multiply/SimpleDivision. Vertical-stack
/// width is driven by the widest operand.
fn vertical_stack_width(max_digits: u32) -> f32 {
    match max_digits {
        0..=2 => 2.5,
        3 => 2.6,
        4 => 3.1,
        _ => 3.7, // 5+ digits
    }
}

/// Regional defaults for operator symbols in horizontal layouts.
///
/// Vertical/bracket layouts use universal notation regardless of locale.
/// Locale only affects horizontal layouts (drills).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Locale {
    #[default]
    Us,
    No,
}

impl Locale {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "us" => Some(Locale::Us),
            "no" => Some(Locale::No),
            _ => None,
        }
    }

    /// Default multiplication symbol for horizontal layouts.
    pub fn multiply_symbol(&self) -> &'static str {
        match self {
            Locale::Us => "sym.times",
            Locale::No => "sym.dot.op",
        }
    }

    /// Default division symbol for horizontal layouts.
    pub fn divide_symbol(&self) -> &'static str {
        match self {
            Locale::Us => "sym.div",
            Locale::No => "sym.colon",
        }
    }
}

/// Physical paper size. Owns the page dimensions so nothing else has
/// to string-match "a4" vs "us-letter". Switching paper is a single
/// field change: all downstream layout math keys off
/// `Paper::dimensions_cm()` and `content_area_cm(paper)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
pub enum Paper {
    #[default]
    #[serde(rename = "a4")]
    A4,
    /// US-Letter, 8.5 × 11 in. Accepts both `"us-letter"` (canonical,
    /// matches typst's `#set page(paper: "us-letter")`) and `"letter"`
    /// (alias for query-string convenience) on the wire.
    #[serde(rename = "us-letter", alias = "letter")]
    Letter,
}

impl Paper {
    /// (width, height) in centimeters.
    pub fn dimensions_cm(self) -> (f32, f32) {
        match self {
            Paper::A4 => (21.0, 29.7),       // 210 × 297 mm
            Paper::Letter => (21.59, 27.94), // 8.5 × 11 in
        }
    }

    /// String passed to typst's `#set page(paper: ...)`.
    pub fn typst_name(self) -> &'static str {
        match self {
            Paper::A4 => "a4",
            Paper::Letter => "us-letter",
        }
    }
}

impl std::fmt::Display for Paper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.typst_name())
    }
}

impl std::str::FromStr for Paper {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "a4" | "A4" => Ok(Paper::A4),
            "us-letter" | "letter" | "Letter" => Ok(Paper::Letter),
            _ => Err(format!(
                "unknown paper size {s:?}; expected \"a4\" or \"us-letter\""
            )),
        }
    }
}

/// Page-chrome geometry — the header/footer bands live in the page
/// margins (via typst's `page.header` / `page.footer`) rather than in
/// body flow, so these constants MUST stay in sync with:
///   - `lib/header.typ`'s `box(height: 1.5cm, ...)`
///   - `lib/footer.typ`'s `box(height: 0.8cm, ...)`
///   - `crates/core/src/document.rs`'s `#set page(margin: ..., header-ascent: ..., footer-descent: ...)` emission.
///
/// Template.rs interpolates these into the generated typst source so
/// the Rust side is the single source of truth; changing a value here
/// changes the emitted page layout too.
#[derive(Debug, Clone, Copy)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

pub const MARGINS_CM: Margins = Margins {
    top: 3.2,
    bottom: 2.2,
    left: 1.5,
    right: 1.5,
};

/// Height of `worksheet-header`'s box in `lib/header.typ`.
pub const HEADER_HEIGHT_CM: f32 = 1.5;
/// Height of `worksheet-footer`'s box in `lib/footer.typ`.
pub const FOOTER_HEIGHT_CM: f32 = 0.8;
/// typst `header-ascent` — distance from page top to the top of the
/// header box within the top margin.
pub const HEADER_ASCENT_CM: f32 = 0.8;
/// typst `footer-descent` — distance from page bottom to the bottom of
/// the footer box within the bottom margin.
pub const FOOTER_DESCENT_CM: f32 = 0.4;
/// `pad(top: ...)` applied around `worksheet-header` in the page.header
/// callback — pushes the header content down so the Name/Date row
/// doesn't kiss the top margin.
pub const HEADER_PAD_TOP_CM: f32 = 0.7;
/// `pad(bottom: ...)` applied around `worksheet-footer`.
pub const FOOTER_PAD_BOTTOM_CM: f32 = 0.7;

/// Body content area (grid region) on the given paper, in cm. Driven
/// entirely by paper dimensions + `MARGINS_CM`; chrome lives in the
/// margins via `page.header` / `page.footer` so the body area already
/// excludes it. Used by pagination in step 7.
pub fn content_area_cm(paper: Paper) -> (f32, f32) {
    let (pw, ph) = paper.dimensions_cm();
    (
        pw - MARGINS_CM.left - MARGINS_CM.right,
        ph - MARGINS_CM.top - MARGINS_CM.bottom,
    )
}

/// Per-problem rendering choice. Emitted to typst as a string tag
/// ("blank" | "worked" | "answer-only"); `lib/layout.typ` derives
/// the per-problem `solved` and `answer-only` flags from the tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderMode {
    Blank,
    Worked,
    AnswerOnly,
}

impl RenderMode {
    pub(crate) fn as_tag(self) -> &'static str {
        match self {
            RenderMode::Blank => "blank",
            RenderMode::Worked => "worked",
            RenderMode::AnswerOnly => "answer-only",
        }
    }
}

/// Page-level surroundings of a worksheet. Everything the grid doesn't
/// care about lives here — paper size, student name overlay, whether
/// to emit an answer-key section, debug borders. `solve_first` lives
/// here per LAYOUT_REFACTOR.md § "Open questions": kept as a chrome
/// convenience flag; `Document::render` translates it to per-problem
/// modes at emission time.
#[derive(Debug, Clone)]
pub struct Chrome {
    pub student_name: Option<String>,
    pub paper: Paper,
    pub include_answers: bool,
    pub debug: bool,
    pub solve_first: bool,
}

impl Chrome {
    /// Copy the chrome-relevant fields out of a flat `WorksheetParams`.
    pub fn from_params(p: &WorksheetParams) -> Self {
        Self {
            student_name: p.student_name.clone(),
            paper: p.paper,
            include_answers: p.include_answers,
            debug: p.debug,
            solve_first: p.solve_first,
        }
    }

    /// Body content area (grid region) in cm on the configured paper.
    pub fn content_area_cm(&self) -> (f32, f32) {
        content_area_cm(self.paper)
    }
}

/// Per-component knobs fed into `worksheet-grid` at emission time.
/// Flat bag — which keys a given component reads is up to the typst
/// side. `Document::render` emits only the keys that the current
/// worksheet's component uses.
///
/// TODO: revisit this shape. Today it's flat with every possible key,
/// but most keys are dead for any given worksheet:
///   - `operator`        — everyone except long-div
///   - `width_cm`        — vertical-stack + long-div
///   - `answer_rows`     — vertical-stack + long-div
///   - `pad_width`       — vertical-stack (binary add only)
///   - `implicit`        — algebra-two-step only
///   - `variable`        — algebra-two-step only
///
/// A variant enum (`ComponentOpts::VerticalStack { .. }`,
/// `::LongDivision { .. }`, `::HorizontalInline { .. }`,
/// `::AlgebraTwoStep { .. }`, …) would encode the shape honestly —
/// each variant carries only its relevant keys, `Document::render`
/// matches once. The redundancy with `WorksheetType`'s discriminator
/// is real but represents a different dimension (which layout
/// primitive, not which problems).
#[derive(Debug, Clone)]
pub struct ComponentOpts {
    /// Typst expression yielding the operator content (e.g.
    /// `"sym.plus"`). Empty string for long-division.
    pub operator: String,
    /// Cell width for the vertical-stack / long-division layouts.
    /// Other components ignore it.
    pub width_cm: f64,
    /// Rows of writing space below the answer line (vertical +
    /// long-division).
    pub answer_rows: u32,
    /// Left-pad operands with "0" up to this many digits. Binary
    /// addition only; 0 elsewhere.
    pub pad_width: u32,
    /// Implicit coefficient-variable juxtaposition (`4x` vs `4·x`).
    /// Algebra two-step only.
    pub implicit: bool,
    /// Variable glyph in algebra problems. Validated to a single
    /// unicode scalar upstream.
    pub variable: String,
}

/// Pure data a generator produces. Zero awareness of chrome or paging
/// — just the problems, the worksheet type, and the component options.
#[derive(Debug, Clone)]
pub struct Sheet {
    pub worksheet: WorksheetType,
    pub problems: Vec<Vec<u32>>,
    pub opts: ComponentOpts,
}

/// A worksheet ready to validate and render. Owns its `Sheet`. The
/// `num_problems` / `pages` fields are transitional until step 7
/// switches pagination to an overflow-based model — at that point
/// both become derived from `cols`, `cell_size_cm`, and `content_area_cm`.
#[derive(Debug, Clone)]
pub struct Document {
    pub sheet: Sheet,
    pub cols: u32,
    pub chrome: Chrome,
    /// Number of problems per page.
    pub num_problems: u32,
    /// Number of problem pages.
    pub pages: u32,
}

impl Document {
    /// Build a Document from legacy `WorksheetParams` — runs the
    /// appropriate per-worksheet generator to produce the `Sheet`,
    /// wraps it up, and runs `validate()`.
    pub fn from_params(params: &WorksheetParams) -> Result<Self> {
        let sheet = match &params.worksheet {
            WorksheetType::Add { .. } => add::generate(params)?,
            WorksheetType::Subtract { .. } => subtract::generate(params)?,
            WorksheetType::Multiply { .. } => multiply::generate(params)?,
            WorksheetType::SimpleDivision { .. } => divide::generate_simple(params)?,
            WorksheetType::LongDivision { .. } => divide::generate_long(params)?,
            WorksheetType::MultiplicationDrill { .. } => mult_drill::generate(params)?,
            WorksheetType::DivisionDrill { .. } => div_drill::generate(params)?,
            WorksheetType::FractionMultiply { .. } => fraction_mult::generate(params)?,
            WorksheetType::AlgebraTwoStep { .. } => algebra_two_step::generate(params)?,
        };
        let doc = Document {
            sheet,
            cols: params.cols,
            chrome: Chrome::from_params(params),
            num_problems: params.num_problems,
            pages: params.pages,
        };
        doc.validate()?;
        Ok(doc)
    }

    /// Check the user-supplied `cols` fits on the configured paper for
    /// the worst-case operand digit count.
    pub fn validate(&self) -> Result<()> {
        let max_digits = self.sheet.worksheet.max_digits_bound();
        let (cell_w, _) = self.sheet.worksheet.cell_size_cm(max_digits);
        let (area_w, _) = self.chrome.content_area_cm();
        let max_cols = (area_w / cell_w).floor() as u32;
        if self.cols > max_cols {
            bail!(
                "{} cols is too wide for {} on {}: at {max_digits} digits \
                 each cell is {cell_w:.2}cm but content area is only \
                 {area_w:.2}cm wide (max {max_cols} cols)",
                self.cols,
                self.sheet.worksheet.component_typst_name(),
                self.chrome.paper,
            );
        }
        Ok(())
    }

    /// Emit the complete typst source for this document.
    pub fn render(&self) -> Result<String> {
        document::render_document(self)
    }
}

#[derive(Debug, Clone)]
pub struct WorksheetParams {
    pub worksheet: WorksheetType,
    pub num_problems: u32,
    pub cols: u32,
    pub paper: Paper,
    pub debug: bool,
    pub seed: Option<u64>,
    /// Explicit symbol override. Takes precedence over locale.
    pub symbol: Option<String>,
    pub locale: Locale,
    /// Number of pages. Each page has `num_problems` unique problems.
    /// pages > 1 requires PDF output.
    pub pages: u32,
    /// Render the first problem as a worked example — applies to any
    /// worksheet type whose problem component understands a `solved` flag.
    pub solve_first: bool,
    /// Append an answer-key page (or pages, one per problem page) showing
    /// just the correct answer for each problem. Requires PDF output since
    /// multi-page layouts don't fit in PNG/SVG.
    pub include_answers: bool,
    /// Pre-filled student name rendered on the Name line in a handwriting
    /// font. `None` leaves the blank signature line.
    pub student_name: Option<String>,
}

impl WorksheetParams {
    /// Total number of unique problems across all pages.
    pub fn total_problems(&self) -> u32 {
        self.num_problems * self.pages
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    Pdf,
    Png,
    Svg,
}

pub struct Worksheet {
    pub bytes: Vec<u8>,
    pub format: OutputFormat,
}

/// Compile arbitrary .typ source and export to the given format.
/// Used by visual testing / Storybook generation to render isolated components.
pub fn compile_typst(
    typ_source: &str,
    format: OutputFormat,
    root: &std::path::Path,
    fonts: &Fonts,
) -> Result<Vec<u8>> {
    world::compile_and_export(typ_source, format, root, fonts)
}

/// Compile a .typ source and return the first page's natural bounding
/// box in centimeters. Pair with `#set page(width: auto, height: auto,
/// margin: 0pt)` to measure a single rendered component.
pub fn measure_typst(
    typ_source: &str,
    root: &std::path::Path,
    fonts: &Fonts,
) -> Result<(f32, f32)> {
    world::measure(typ_source, root, fonts)
}

pub fn generate(
    params: &WorksheetParams,
    format: OutputFormat,
    root: &std::path::Path,
    fonts: &Fonts,
) -> Result<Worksheet> {
    if (params.pages > 1 || params.include_answers) && !matches!(format, OutputFormat::Pdf) {
        bail!(
            "pages > 1 or include_answers requires PDF output (PNG/SVG are single-image formats)"
        );
    }
    // Typst memoizes compilation via the global `comemo` cache (shared
    // across all `World`s in the process). Without periodic eviction the
    // cache grows unbounded under load — on a 256 MB Fly machine that
    // OOMs within minutes. Scoped here rather than inside `compile_typst`
    // so callers measuring raw cache behavior (see examples/cache_growth)
    // can render without side-effects.
    typst::comemo::evict(10);
    let typ_source = generate_typst_source(params)?;
    let bytes = world::compile_and_export(&typ_source, format, root, fonts)?;
    Ok(Worksheet { bytes, format })
}

/// Build the typst source for a worksheet without compiling it. Exposed
/// so the CLI can concatenate several worksheets into a single multi-page
/// PDF (the `all` subcommand).
pub fn generate_typst_source(params: &WorksheetParams) -> Result<String> {
    if params.pages == 0 {
        bail!("pages must be at least 1");
    }

    let is_drill = matches!(
        &params.worksheet,
        WorksheetType::MultiplicationDrill { .. } | WorksheetType::DivisionDrill { .. }
    );
    let max_problems = if is_drill {
        MAX_PROBLEMS_DRILL
    } else {
        MAX_PROBLEMS
    };
    // Drills allow 0 = "all problems" (auto-sized from table enumeration).
    if is_drill {
        if params.num_problems > max_problems {
            bail!(
                "number of problems must be 0-{max_problems}, got {}",
                params.num_problems
            );
        }
    } else if params.num_problems == 0 || params.num_problems > max_problems {
        bail!(
            "number of problems must be 1-{max_problems}, got {}",
            params.num_problems
        );
    }

    // Per-worksheet range / shape validation. These are cheap checks
    // on the config itself, separate from the paper-fit check in
    // `Document::validate()` and from generator-internal checks.
    match &params.worksheet {
        WorksheetType::Add { digits, .. } => validate_digit_ranges(digits)?,
        WorksheetType::Subtract { digits, .. } => {
            validate_digit_ranges(digits)?;
            if digits.len() > 2 {
                bail!("subtract supports max 2 operands, got {}", digits.len());
            }
        }
        WorksheetType::Multiply { digits } => {
            validate_digit_ranges(digits)?;
            if digits.len() > 2 {
                bail!("multiply supports max 2 operands, got {}", digits.len());
            }
        }
        WorksheetType::SimpleDivision { max_quotient } => {
            validate_max_quotient(*max_quotient)?;
        }
        WorksheetType::LongDivision { digits, .. } => {
            validate_digit_range(*digits, "long division dividend")?;
            if digits.min < 2 || digits.max > 4 {
                bail!(
                    "long division dividend must be 2-4 digits, got {}-{}",
                    digits.min,
                    digits.max
                );
            }
        }
        WorksheetType::MultiplicationDrill {
            multiplicand,
            multiplier,
        } => {
            if params.cols > 3 {
                bail!(
                    "multiplication drill supports max 3 columns, got {}",
                    params.cols
                );
            }
            for r in multiplicand {
                if r.min < 1 || r.max > 12 {
                    bail!("multiplicand must be 1-12, got {}-{}", r.min, r.max);
                }
            }
            if multiplier.min < 1 || multiplier.max > 12 {
                bail!(
                    "multiplier must be 1-12, got {}-{}",
                    multiplier.min,
                    multiplier.max
                );
            }
        }
        WorksheetType::DivisionDrill {
            divisor,
            max_quotient,
        } => {
            if params.cols > 3 {
                bail!(
                    "division drill supports max 3 columns, got {}",
                    params.cols
                );
            }
            for r in divisor {
                if r.min < 1 || r.max > 12 {
                    bail!("divisor must be 1-12, got {}-{}", r.min, r.max);
                }
            }
            if max_quotient.min < 1 || max_quotient.max > 12 {
                bail!(
                    "max-quotient must be 1-12, got {}-{}",
                    max_quotient.min,
                    max_quotient.max
                );
            }
        }
        WorksheetType::FractionMultiply {
            denominators,
            min_whole,
            max_whole,
            ..
        } => {
            if denominators.is_empty() {
                bail!("denominators must have at least one value");
            }
            for &d in denominators {
                if d < 2 || d > 12 {
                    bail!("denominator must be 2-12, got {d}");
                }
            }
            if *min_whole < 2 || *max_whole > 99 || min_whole > max_whole {
                bail!(
                    "whole range must be 2-99 with min ≤ max, got {min_whole}-{max_whole}"
                );
            }
        }
        WorksheetType::AlgebraTwoStep {
            a_range,
            b_range,
            x_range,
            variable,
            ..
        } => {
            if a_range.min < 2 || a_range.max > 12 {
                bail!("a-range must be 2-12, got {}-{}", a_range.min, a_range.max);
            }
            if b_range.max > 99 || b_range.min > b_range.max {
                bail!(
                    "b-range must be 0-99 with min ≤ max, got {}-{}",
                    b_range.min,
                    b_range.max
                );
            }
            if x_range.max > 99 || x_range.min > x_range.max {
                bail!(
                    "x-range must be 0-99 with min ≤ max, got {}-{}",
                    x_range.min,
                    x_range.max
                );
            }
            // Variable must be exactly one unicode scalar (a single letter,
            // symbol, or single-codepoint emoji like 🍌). Compound emoji
            // sequences (flags, ZWJ) aren't supported.
            if variable.chars().count() != 1 {
                bail!("variable must be a single character, got {:?}", variable);
            }
        }
    };

    // One centralized path from params → Sheet → Document → .typ:
    // generators return pure data; Document::from_params assembles
    // the Document (including `Document::validate()` for paper-fit)
    // and Document::render emits the source.
    Document::from_params(params)?.render()
}

/// When a generator's unique-problem pool is smaller than `target`
/// (narrow search spaces — `carry=none` + small binary operands, forced
/// ripple in few digits, etc.), fill the remaining slots by randomly
/// sampling from whatever was generated. Students get some repetition
/// instead of the generator bailing with "not enough unique".
pub(crate) fn pad_with_duplicates<T: Clone>(
    problems: &mut Vec<T>,
    target: usize,
    rng: &mut rand::rngs::SmallRng,
) {
    use rand::Rng;
    if problems.is_empty() || problems.len() >= target {
        return;
    }
    let pool_len = problems.len();
    while problems.len() < target {
        let idx = rng.gen_range(0..pool_len);
        problems.push(problems[idx].clone());
    }
}

fn validate_digit_ranges(ranges: &[DigitRange]) -> Result<()> {
    if ranges.is_empty() {
        bail!("digits must have at least one value");
    }
    for r in ranges {
        validate_digit_range(*r, "operand")?;
    }
    if ranges.len() > MAX_OPERANDS {
        bail!("max {MAX_OPERANDS} operands, got {}", ranges.len());
    }
    Ok(())
}

fn validate_digit_range(r: DigitRange, label: &str) -> Result<()> {
    if r.min == 0 || r.max > MAX_DIGITS {
        bail!(
            "{label} digit count must be 1-{MAX_DIGITS}, got {}-{}",
            r.min,
            r.max
        );
    }
    if r.min > r.max {
        bail!("{label} digit range invalid: {} > {}", r.min, r.max);
    }
    Ok(())
}

fn validate_max_quotient(mq: u32) -> Result<()> {
    if mq < 2 || mq > 12 {
        bail!("max-quotient must be 2-12, got {mq}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spot-check that `cell_size_cm()` matches the committed
    /// `cell-sizes.toml` fixtures. When this fails: re-run `cargo run
    /// --example measure_cells` and update the mirror in lib.rs in
    /// the same commit.
    #[test]
    fn paper_basics() {
        use std::str::FromStr;
        assert_eq!(Paper::default(), Paper::A4);
        assert_eq!(Paper::A4.typst_name(), "a4");
        assert_eq!(Paper::Letter.typst_name(), "us-letter");
        assert_eq!(Paper::A4.dimensions_cm(), (21.0, 29.7));
        assert_eq!(Paper::Letter.dimensions_cm(), (21.59, 27.94));
        assert_eq!(Paper::from_str("a4").unwrap(), Paper::A4);
        assert_eq!(Paper::from_str("us-letter").unwrap(), Paper::Letter);
        assert_eq!(Paper::from_str("letter").unwrap(), Paper::Letter);
        assert!(Paper::from_str("legal").is_err());
    }

    #[test]
    fn content_area_cm_matches_hand_math() {
        // A4 body = 21 − 1.5 − 1.5 wide, 29.7 − 3.2 − 2.2 tall.
        let (w, h) = content_area_cm(Paper::A4);
        assert!((w - 18.0).abs() < 0.01);
        assert!((h - 24.3).abs() < 0.01);
        // Letter body = 21.59 − 3.0 wide, 27.94 − 5.4 tall.
        let (w, h) = content_area_cm(Paper::Letter);
        assert!((w - 18.59).abs() < 0.01);
        assert!((h - 22.54).abs() < 0.01);
    }

    #[test]
    fn max_digits_bound_examples() {
        assert_eq!(
            WorksheetType::Add {
                digits: vec![DigitRange::new(2, 4), DigitRange::fixed(3)],
                carry: CarryMode::Any,
                binary: false,
            }
            .max_digits_bound(),
            4
        );
        assert_eq!(
            WorksheetType::LongDivision {
                digits: DigitRange::fixed(3),
                remainder: true,
            }
            .max_digits_bound(),
            3
        );
    }

    #[test]
    fn document_validate_rejects_too_many_cols() {
        // Default A4 content width = 18cm. A 3-digit add cell is 2.6cm,
        // so max cols = floor(18 / 2.6) = 6. Asking for 7 should error.
        let sheet = Sheet {
            worksheet: WorksheetType::Add {
                digits: vec![DigitRange::fixed(3), DigitRange::fixed(3)],
                carry: CarryMode::Any,
                binary: false,
            },
            problems: vec![],
            opts: ComponentOpts {
                operator: "sym.plus".to_string(),
                width_cm: 2.6,
                answer_rows: 1,
                pad_width: 0,
                implicit: false,
                variable: "x".to_string(),
            },
        };
        let chrome = Chrome {
            student_name: None,
            paper: Paper::A4,
            include_answers: false,
            debug: false,
            solve_first: false,
        };
        let doc = Document {
            sheet: sheet.clone(),
            cols: 7,
            chrome: chrome.clone(),
            num_problems: 12,
            pages: 1,
        };
        let err = doc.validate().unwrap_err().to_string();
        assert!(err.contains("too wide"), "unexpected error: {err}");

        // 6 cols should fit.
        let ok_doc = Document {
            cols: 6,
            ..doc
        };
        assert!(ok_doc.validate().is_ok());
    }

    #[test]
    fn cell_size_cm_matches_toml_fixtures() {
        // addition-basic-d3-op2-blank: 3-digit operands → 2.6 × 3.5
        let add = WorksheetType::Add {
            digits: vec![DigitRange::fixed(3), DigitRange::fixed(3)],
            carry: CarryMode::Any,
            binary: false,
        };
        assert_eq!(add.cell_size_cm(3), (2.6, 3.5));

        // multiplication-basic-d3x3-blank: 3×3 → 2.6 × 6.5
        let mul_3x3 = WorksheetType::Multiply {
            digits: vec![DigitRange::fixed(3), DigitRange::fixed(3)],
        };
        assert_eq!(mul_3x3.cell_size_cm(3), (2.6, 6.5));

        // multiplication-basic-d2x2-blank / d3x2-blank: 2-digit multiplier
        // drives 5.5 cm height regardless of multiplicand width.
        let mul_3x2 = WorksheetType::Multiply {
            digits: vec![DigitRange::fixed(3), DigitRange::fixed(2)],
        };
        assert_eq!(mul_3x2.cell_size_cm(3), (2.6, 5.5));

        // division-long-d3-blank: 3-digit dividend → 3.8 × 8.0
        let long_d3 = WorksheetType::LongDivision {
            digits: DigitRange::fixed(3),
            remainder: true,
        };
        assert_eq!(long_d3.cell_size_cm(3), (3.8, 8.0));

        // multiplication-drill-d1x1: 4.5 × 1.0
        let mult_drill = WorksheetType::MultiplicationDrill {
            multiplicand: vec![DigitRange::fixed(7)],
            multiplier: DigitRange::new(1, 10),
        };
        assert_eq!(mult_drill.cell_size_cm(1), (4.5, 1.0));

        // fraction-multiplication-unit-d2: 6.0 × 2.8
        let frac = WorksheetType::FractionMultiply {
            denominators: vec![2, 3, 4],
            min_whole: 1,
            max_whole: 99,
            unit_only: true,
        };
        assert_eq!(frac.cell_size_cm(2), (6.0, 2.8));

        // algebra-two-step-small-form0: 7.3 × 4.1 (all ranges ≤ 30)
        let alg_small = WorksheetType::AlgebraTwoStep {
            a_range: DigitRange::new(2, 12),
            b_range: DigitRange::new(1, 30),
            x_range: DigitRange::new(0, 20),
            variable: "x".into(),
            implicit: false,
            mix_forms: false,
        };
        assert_eq!(alg_small.cell_size_cm(2), (7.3, 4.1));
    }
}

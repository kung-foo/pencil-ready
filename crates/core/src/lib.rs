//! pencil-ready-core: worksheet generation and typst compilation.
//!
//! Each worksheet type lives in its own module.
//! They share a common template renderer and the typst World implementation.

mod add;
mod algebra_one_step;
mod algebra_square_root;
mod algebra_two_step;
mod decimal_add;
mod decimal_mult;
mod decimal_sub;
mod div_drill;
mod divide;
mod fraction_equiv;
mod fraction_mult;
mod fraction_simplify;
mod meta;
mod mult_drill;
mod multiply;
mod subtract;

mod document;
mod world;

pub use fraction_equiv::MissingSlot;
pub use world::Fonts;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

const MAX_DIGITS: u32 = 5;
/// Upper bound on total problems across the whole worksheet (all
/// pages combined). Chosen to cover realistic teacher packets without
/// exposing a trivial DoS vector against the public server — typst
/// compile time scales roughly with problem count.
const MAX_PROBLEMS: u32 = 100;
/// Same cap for drills, which are cheaper per problem (horizontal
/// single-line) so can afford more.
const MAX_PROBLEMS_DRILL: u32 = 200;
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
    FractionSimplify {
        /// Allowed denominators of the printed fraction.
        denominators: Vec<u32>,
        /// Maximum numerator of the printed fraction. Setting it larger
        /// than the largest denominator lets improper fractions appear.
        max_numerator: u32,
        /// If true, allow numerator >= denominator (answers become mixed
        /// numbers). Default true.
        include_improper: bool,
        /// If true, allow problems whose reduced denominator collapses
        /// to 1 (answer is a pure whole number, e.g. 12/4 → 3). Default
        /// false — these feel more like division than simplification.
        include_whole: bool,
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
    AlgebraOneStep {
        /// Coefficient/divisor range used by the multiply/divide forms
        /// (default 2-10).
        a_range: DigitRange,
        /// Constant range used by the add/subtract forms (default 1-30).
        b_range: DigitRange,
        /// Answer (x) range (default 0-20).
        x_range: DigitRange,
        /// Variable glyph; same rules as `AlgebraTwoStep::variable`.
        variable: String,
        /// `x + b = c`.
        add: bool,
        /// `x − b = c` — only triples with `x ≥ b` are emitted so `c ≥ 0`.
        subtract: bool,
        /// `a · x = c`.
        multiply: bool,
        /// `x ÷ a = c` — only triples where `a` divides `x` evenly.
        divide: bool,
    },
    FractionEquiv {
        /// Allowed denominators of the base (reduced) fraction.
        denominators: Vec<u32>,
        /// Scale-factor range — the multiplier applied to produce the
        /// equivalent fraction. Min 2; max reasonable 10.
        scale: DigitRange,
        /// Which of the four slots is left blank. `Any` picks randomly.
        missing: MissingSlot,
        /// Restrict base fraction to proper fractions (num < den).
        proper_only: bool,
    },
    AlgebraSquareRoot {
        /// Constant range (default 1-30). Same shape as two-step's
        /// `b_range` so the two algebra worksheets feel uniform.
        b_range: DigitRange,
        /// Variable glyph; same rules as `AlgebraTwoStep::variable`.
        variable: String,
        /// Include `x² ± b = c` problems.
        squares: bool,
        /// Include `√x ± b = c` problems.
        roots: bool,
    },
    /// Decimal column addition. Operands and answer all share the same
    /// number of decimal places, so the printed decimal points line up
    /// column-wise. Values flow through `Sheet.problems` as scaled
    /// integers (e.g. `1.23` → `123`), and the typst component
    /// re-introduces the decimal point at render time using the
    /// `decimal-places` opt.
    DecimalAdd {
        /// Per-operand integer-part digit count (e.g. `2,2` for two
        /// 2-digit-integer operands). Length determines operand count.
        digits: Vec<DigitRange>,
        /// Decimal places shared by every operand and the answer.
        decimal_places: u32,
    },
    /// Decimal column subtraction. Always 2 operands; first is always ≥
    /// second so the answer is non-negative. Same encoding as
    /// `DecimalAdd`.
    DecimalSubtract {
        digits: Vec<DigitRange>,
        decimal_places: u32,
    },
    /// Decimal multiplication. Top operand has `decimal_places` dp, the
    /// multiplier (bottom) has `bottom_decimal_places` (0 = whole), and
    /// the answer's dp is the sum. Single answer-row layout — partial
    /// products with the dp shift come later.
    DecimalMultiply {
        /// Integer-part digit count of the top (decimal) operand.
        digits: DigitRange,
        /// Decimal places on the top operand.
        decimal_places: u32,
        /// Multiplier integer-part value range (e.g. 2..=9). Inclusive.
        /// When `bottom_decimal_places > 0` these bound the integer
        /// part; the random fractional part is drawn separately.
        multiplier_min: u32,
        multiplier_max: u32,
        /// Decimal places on the multiplier. 0 = whole number.
        bottom_decimal_places: u32,
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
            WorksheetType::FractionSimplify { .. } => "fraction-simplification-problem",
            WorksheetType::AlgebraTwoStep { .. } => "algebra-two-step-problem",
            WorksheetType::AlgebraOneStep { .. } => "algebra-one-step-problem",
            WorksheetType::FractionEquiv { .. } => "fraction-equivalence-problem",
            WorksheetType::AlgebraSquareRoot { .. } => "algebra-square-root-problem",
            WorksheetType::DecimalAdd { .. } => "decimal-add-problem",
            WorksheetType::DecimalSubtract { .. } => "decimal-subtract-problem",
            WorksheetType::DecimalMultiply { .. } => "decimal-multiply-problem",
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
            // When `remainder` is on, the answer is rendered as `Q r N`,
            // which is wider than just the quotient — bump cell width so
            // the bracket overline + suffix fit inside the cell.
            WorksheetType::LongDivision { digits, remainder } => {
                let (w, h) = match digits.max {
                    1..=2 => (3.8, 6.0),
                    3 => (3.8, 8.0),
                    _ => (4.4, 10.0),
                };
                if *remainder { (w + 1.2, h) } else { (w, h) }
            }

            // Horizontal drills — same primitive for multiplication and
            // division. d1x1 fits in 4.5cm; d2x1 needs 5.0cm.
            WorksheetType::MultiplicationDrill { .. } | WorksheetType::DivisionDrill { .. } => {
                let w = if max_digits <= 1 { 4.5 } else { 5.0 };
                (w, 1.0)
            }

            // Fraction × whole — width is fixed (the fraction slot
            // dominates). 2-row vertical layout gives 2.8cm height.
            WorksheetType::FractionMultiply { .. } => (6.0, 2.8),

            // Fraction simplification: LHS fraction + `=` + answer slot
            // on a single row. Width fixed — largest answers fit a 2-digit
            // mixed number (e.g. "2 7/12"). Height is the fraction stack.
            WorksheetType::FractionSimplify { .. } => (5.0, 2.6),

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

            // Algebra one-step — narrower than two-step (no coefficient
            // grouping or const-first reshuffling) and shorter (two rows
            // instead of three). Widths from cell-sizes.toml fixtures:
            // small numbers (≤30) fit in ~6.0cm, up to 99 needs ~6.4cm.
            WorksheetType::AlgebraOneStep {
                a_range,
                b_range,
                x_range,
                ..
            } => {
                let max = a_range.max.max(b_range.max).max(x_range.max);
                let w = if max <= 30 { 6.2 } else { 6.5 };
                (w, 2.6)
            }

            // Equivalent fractions — two stacked fractions + `=` on one row.
            // Width fixed at 5.5cm; height matches fraction-simplify.
            WorksheetType::FractionEquiv { .. } => (5.5, 2.6),

            // Squares and square-roots — same 3-row shape as two-step
            // (given equation, intermediate, `x = ___`). Inner is pinned
            // 0..10 and answer at most 100, so `b_range.max` (≤99) is
            // the only knob that can push width up.
            WorksheetType::AlgebraSquareRoot { b_range, .. } => {
                let w = if b_range.max <= 30 { 7.3 } else { 8.0 };
                (w, 4.1)
            }

            // Decimal worksheets: `max_digits` is the encoded digit
            // count (integer + dp), so visible-glyph count is
            // max_digits + 1 (the decimal point). Single answer-row.
            WorksheetType::DecimalAdd { .. }
            | WorksheetType::DecimalSubtract { .. }
            | WorksheetType::DecimalMultiply { .. } => {
                (decimal_stack_width(max_digits + 1), 3.5)
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
            | WorksheetType::Multiply { digits } => digits.iter().map(|r| r.max).max().unwrap_or(2),
            WorksheetType::LongDivision { digits, .. } => digits.max,
            WorksheetType::MultiplicationDrill {
                multiplicand,
                multiplier,
            } => {
                let a = multiplicand.iter().map(|r| r.max).max().unwrap_or(1);
                a.max(multiplier.max)
            }
            WorksheetType::DivisionDrill {
                divisor,
                max_quotient,
            } => {
                let a = divisor.iter().map(|r| r.max).max().unwrap_or(1);
                a.max(max_quotient.max)
            }
            // A simple-division problem is dividend÷divisor = quotient
            // where quotient ≤ max_quotient and divisor is 1-digit.
            // The widest operand is the product (dividend), up to
            // ~9 * max_quotient — at most 3 digits for sane inputs.
            WorksheetType::SimpleDivision { max_quotient } => {
                document::digit_count(9 * *max_quotient)
            }
            // Fraction-mult's width is driven by the whole-number LHS.
            WorksheetType::FractionMultiply { max_whole, .. } => document::digit_count(*max_whole),
            // Simplify's width is capped by max_numerator (the widest
            // operand printed on the LHS) and the largest denominator.
            WorksheetType::FractionSimplify {
                max_numerator,
                denominators,
                ..
            } => {
                let max_den = denominators.iter().copied().max().unwrap_or(2);
                document::digit_count((*max_numerator).max(max_den))
            }
            // Algebra's width bound is the largest numeric literal in
            // the LHS / intermediate / solution lines — worst case the
            // a*x+b product or `c` itself.
            WorksheetType::AlgebraTwoStep {
                a_range,
                b_range,
                x_range,
                ..
            } => document::digit_count(
                (a_range.max * x_range.max)
                    .max(b_range.max)
                    .max(a_range.max),
            ),
            // One-step's widest literal: `c` for add (x_max + b_max) or
            // mul (a_max * x_max). Sub and div produce smaller `c`s.
            WorksheetType::AlgebraOneStep {
                a_range,
                b_range,
                x_range,
                ..
            } => document::digit_count(
                (a_range.max * x_range.max).max(x_range.max + b_range.max),
            ),
            // Widest number: scaled denominator = max_den * scale_max.
            WorksheetType::FractionEquiv {
                denominators,
                scale,
                ..
            } => {
                let max_den = denominators.iter().copied().max().unwrap_or(2);
                document::digit_count(max_den * scale.max)
            }
            // Square-roots: inner ≤ 10, so x² ≤ 100 and the root-form
            // answer (r²) is also ≤ 100. The widest emitted literal is
            // therefore max(100, b_max).
            WorksheetType::AlgebraSquareRoot { b_range, .. } => {
                document::digit_count(100u32.max(b_range.max))
            }
            // Decimal worksheets — encoded digit count (integer part +
            // decimal places). The widest sum's integer part can be 1
            // wider than its operands' (carry); decimal_places is on
            // top of that.
            WorksheetType::DecimalAdd {
                digits,
                decimal_places,
            } => digits.iter().map(|r| r.max).max().unwrap_or(2) + 1 + *decimal_places,
            WorksheetType::DecimalSubtract {
                digits,
                decimal_places,
            } => digits.iter().map(|r| r.max).max().unwrap_or(2) + *decimal_places,
            // Decimal multiply: encoded answer up to top_max * bot_max
            // = 10^(top_int+top_dp+bot_int+bot_dp). The bot integer
            // digit count comes from `multiplier_max`.
            WorksheetType::DecimalMultiply {
                digits,
                decimal_places,
                multiplier_max,
                bottom_decimal_places,
                ..
            } => {
                let bot_int = document::digit_count((*multiplier_max).max(1));
                digits.max + decimal_places + bot_int + bottom_decimal_places
            }
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

/// Width for decimal-stack cells. `total_glyphs` counts integer digits +
/// decimal point + decimal places. Roughly 0.55cm per glyph plus a small
/// padding allowance.
fn decimal_stack_width(total_glyphs: u32) -> f32 {
    match total_glyphs {
        0..=3 => 2.8,
        4 => 3.1,
        5 => 3.6,
        6 => 4.1,
        7 => 4.6,
        _ => 5.2,
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
    // `top` is overridden per-page by `margin_top_for_chrome` — chrome
    // height varies with which sections are present (Name/Date, title,
    // instructions). The constant here is just the worst-case (all
    // three sections) used by callers that don't compute it themselves.
    // bottom/left/right are constant across worksheets.
    top: 5.2,
    bottom: 2.2,
    left: 1.5,
    right: 1.5,
};

// Section heights for the variable-chrome layout in `lib/header.typ`.
// Mirror the typst constants of the same name; the two must stay in
// sync.
pub const NAME_DATE_SECTION_CM: f32 = 0.7;
pub const TITLE_SECTION_CM: f32 = 0.85;
pub const INSTRUCTIONS_SECTION_CM: f32 = 1.65;
/// Legacy COMPACT chrome height — Name/Date + plain rule + bottom
/// padding, used when neither title nor instructions is set.
pub const COMPACT_CHROME_CM: f32 = 1.5;

/// Worst-case chrome height (Name/Date + title + instructions). Used
/// where a single conservative figure is needed (e.g. PDF metadata
/// sizing); per-worksheet pagination uses [`chrome_height_cm`] instead.
pub const HEADER_HEIGHT_CM: f32 =
    NAME_DATE_SECTION_CM + TITLE_SECTION_CM + INSTRUCTIONS_SECTION_CM;

/// Compute the chrome's box height in cm for the given combination of
/// sections. Mirrors the typst layout in `lib/header.typ`.
///
/// The legacy compact case (Name/Date only, no title or instructions)
/// returns 1.5cm to preserve existing story-baseline pixel positions.
pub fn chrome_height_cm(show_name_date: bool, has_title: bool, has_instructions: bool) -> f32 {
    if show_name_date && !has_title && !has_instructions {
        return COMPACT_CHROME_CM;
    }
    let mut h = 0.0;
    if show_name_date {
        h += NAME_DATE_SECTION_CM;
    }
    if has_title {
        h += TITLE_SECTION_CM;
    } else if has_instructions && show_name_date {
        // Plain divider rule between Name/Date and instructions when
        // there's no title to provide one — small slot taken from the
        // instructions section's budget.
        h += 0.35;
    }
    if has_instructions {
        h += INSTRUCTIONS_SECTION_CM;
    }
    h
}

/// Compute `margin.top` (in cm) needed to fit the given chrome above
/// the body grid. Adds [`HEADER_ASCENT_CM`] (gap above header) +
/// [`HEADER_PAD_TOP_CM`] (the `pad(top: ...)` wrapper around the
/// header callback) to the chrome's box height.
pub fn margin_top_for_chrome(chrome_h_cm: f32) -> f32 {
    HEADER_ASCENT_CM + HEADER_PAD_TOP_CM + chrome_h_cm
}
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

/// Body content area (grid region) on the given paper, in cm.
///
/// `chrome_h_cm` is the worksheet's chrome height (see
/// [`chrome_height_cm`]); that determines the page's top margin via
/// [`margin_top_for_chrome`]. Footer/left/right margins are constant
/// across worksheets, so only the top margin varies.
pub fn content_area_cm(paper: Paper, chrome_h_cm: f32) -> (f32, f32) {
    let (pw, ph) = paper.dimensions_cm();
    let margin_top = margin_top_for_chrome(chrome_h_cm);
    (
        pw - MARGINS_CM.left - MARGINS_CM.right,
        ph - margin_top - MARGINS_CM.bottom,
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
    /// Override for the worksheet's default instruction text. `None`
    /// means use `WorksheetType::instructions()` for the type. `Some`
    /// means render the provided string verbatim — used by callers
    /// (typically the server) that want to ship custom copy without
    /// touching the per-type defaults.
    pub instructions: Option<String>,
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
            instructions: p.instructions.clone(),
        }
    }

    /// Body content area (grid region) in cm on the configured paper,
    /// for the given chrome height. Use [`Chrome::problem_chrome_h_cm`]
    /// or compute directly via [`chrome_height_cm`] for the worksheet
    /// at hand.
    pub fn content_area_cm(&self, chrome_h_cm: f32) -> (f32, f32) {
        content_area_cm(self.paper, chrome_h_cm)
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
#[derive(Debug, Clone, Default)]
pub struct ComponentOpts {
    /// Typst expression yielding the operator content (e.g.
    /// `"sym.plus"`). Empty string for long-division.
    pub operator: String,
    /// Secondary operator for components that render two operations
    /// in the same problem (algebra one-step renders both `·` for
    /// multiply and `÷`/`:` for divide). Empty string when unused.
    pub divide_operator: String,
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
    /// Per-slot decimal places, one entry per number in each problem
    /// tuple (operands + answer). Empty for non-decimal worksheets.
    pub decimal_places: Vec<u32>,
    /// Long-division only: when true, every cell reserves overline
    /// width for the worst-case `Q r N` answer, so brackets render
    /// uniformly across cells even when individual problems have no
    /// remainder.
    pub reserve_remainder: bool,
}

/// Pure data a generator produces. Zero awareness of chrome or paging
/// — just the problems, the worksheet type, and the component options.
#[derive(Debug, Clone)]
pub struct Sheet {
    pub worksheet: WorksheetType,
    pub problems: Vec<Vec<u32>>,
    pub opts: ComponentOpts,
}

/// A worksheet ready to validate and render. Owns its `Sheet`.
/// Pagination is derived — `cells_per_page` and `pages` come from
/// `cols × rows_per_page` where `rows_per_page = floor(content_area_cm.h
/// / cell_size_cm.h)`. Users no longer supply `pages` directly.
#[derive(Debug, Clone)]
pub struct Document {
    pub sheet: Sheet,
    pub cols: u32,
    pub chrome: Chrome,
    /// `cols × rows_per_page`. Drives the per-page chunking in
    /// `Document::render`.
    pub cells_per_page: u32,
    /// Number of problem pages. `ceil(problems.len() / cells_per_page)`.
    /// Does not include answer-key pages (they're appended in render).
    pub pages: u32,
}

impl Document {
    /// Build a Document from legacy `WorksheetParams` — runs the
    /// appropriate per-worksheet generator to produce the `Sheet`,
    /// derives pagination from `cell_size_cm` + `content_area_cm`,
    /// and runs `validate()`.
    pub fn from_params(params: &WorksheetParams) -> Result<Self> {
        validate_worksheet_params(params)?;
        let sheet = match &params.worksheet {
            WorksheetType::Add { .. } => add::generate(params)?,
            WorksheetType::Subtract { .. } => subtract::generate(params)?,
            WorksheetType::Multiply { .. } => multiply::generate(params)?,
            WorksheetType::SimpleDivision { .. } => divide::generate_simple(params)?,
            WorksheetType::LongDivision { .. } => divide::generate_long(params)?,
            WorksheetType::MultiplicationDrill { .. } => mult_drill::generate(params)?,
            WorksheetType::DivisionDrill { .. } => div_drill::generate(params)?,
            WorksheetType::FractionMultiply { .. } => fraction_mult::generate(params)?,
            WorksheetType::FractionSimplify { .. } => fraction_simplify::generate(params)?,
            WorksheetType::AlgebraTwoStep { .. } => algebra_two_step::generate(params)?,
            WorksheetType::AlgebraOneStep { .. } => algebra_one_step::generate(params)?,
            WorksheetType::FractionEquiv { .. } => fraction_equiv::generate(params)?,
            WorksheetType::AlgebraSquareRoot { .. } => algebra_square_root::generate(params)?,
            WorksheetType::DecimalAdd { .. } => decimal_add::generate(params)?,
            WorksheetType::DecimalSubtract { .. } => decimal_sub::generate(params)?,
            WorksheetType::DecimalMultiply { .. } => decimal_mult::generate(params)?,
        };
        let chrome = Chrome::from_params(params);
        let max_digits = sheet.worksheet.max_digits_bound();
        let (cell_w, cell_h) = sheet.worksheet.cell_size_cm(max_digits);
        let chrome_h = chrome_height_cm(
            true,
            true,
            chrome
                .instructions
                .as_deref()
                .map(|s| !s.is_empty())
                .unwrap_or_else(|| sheet.worksheet.instructions().is_some()),
        );
        let (area_w, area_h) = chrome.content_area_cm(chrome_h);

        // Paper-fit check: cols must fit in the content-area width.
        let max_cols = (area_w / cell_w).floor() as u32;
        if params.cols > max_cols {
            bail!(
                "{} cols is too wide for {} on {}: at {max_digits} digits \
                 each cell is {cell_w:.2}cm but content area is only \
                 {area_w:.2}cm wide (max {max_cols} cols)",
                params.cols,
                sheet.worksheet.component_typst_name(),
                chrome.paper,
            );
        }
        let rows_per_page = (area_h / cell_h).floor() as u32;
        if rows_per_page == 0 {
            bail!(
                "content area is {area_h:.2}cm tall but a single {} cell \
                 needs {cell_h:.2}cm — nothing fits.",
                sheet.worksheet.component_typst_name(),
            );
        }
        let cells_per_page = params.cols * rows_per_page;
        let problem_count = sheet.problems.len() as u32;
        let pages = if problem_count == 0 {
            1 // degenerate but render-able: one blank grid
        } else {
            problem_count.div_ceil(cells_per_page)
        };

        let doc = Document {
            sheet,
            cols: params.cols,
            chrome,
            cells_per_page,
            pages,
        };
        doc.validate()?;
        Ok(doc)
    }

    /// Check the user-supplied `cols` fits on the configured paper for
    /// the worst-case operand digit count. Kept as a pub method for
    /// callers that construct Document directly.
    pub fn validate(&self) -> Result<()> {
        let max_digits = self.sheet.worksheet.max_digits_bound();
        let (cell_w, _) = self.sheet.worksheet.cell_size_cm(max_digits);
        // Width is independent of chrome height; use any value here.
        let (area_w, _) = self.chrome.content_area_cm(0.0);
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
    /// Override for the worksheet's default instruction text. `None`
    /// uses `WorksheetType::instructions()`; `Some` is rendered verbatim.
    pub instructions: Option<String>,
}

impl WorksheetParams {
    /// Total number of unique problems on the worksheet. Page count
    /// is derived from `cell_size_cm` + `content_area_cm` by
    /// `Document::from_params`.
    pub fn total_problems(&self) -> u32 {
        self.num_problems
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
    // Typst memoizes compilation via the global `comemo` cache (shared
    // across all `World`s in the process). Without periodic eviction the
    // cache grows unbounded under load — on a 256 MB Fly machine that
    // OOMs within minutes. Scoped here rather than inside `compile_typst`
    // so callers measuring raw cache behavior (see examples/cache_growth)
    // can render without side-effects.
    typst::comemo::evict(10);
    // `Document::from_params` runs all config + paper-fit validation
    // and derives the page count; the format check below keys off the
    // derived value rather than a user-supplied hint.
    let doc = Document::from_params(params)?;
    if (doc.pages > 1 || doc.chrome.include_answers) && !matches!(format, OutputFormat::Pdf) {
        bail!(
            "this worksheet spans {} pages (or has --include-answers set); \
             PNG/SVG are single-image formats — use --format pdf",
            doc.pages,
        );
    }
    let typ_source = doc.render()?;
    let bytes = world::compile_and_export(&typ_source, format, root, fonts)?;
    Ok(Worksheet { bytes, format })
}

/// Build the typst source for a worksheet without compiling it. Exposed
/// so the CLI can concatenate several worksheets into a single multi-page
/// PDF (the `all` subcommand).
pub fn generate_typst_source(params: &WorksheetParams) -> Result<String> {
    Document::from_params(params)?.render()
}

/// Per-worksheet range / shape validation. Cheap checks on the config
/// itself, separate from the paper-fit check in `Document::validate()`
/// and from generator-internal validation. Called by
/// `Document::from_params` (so direct callers get it too, not just
/// the `generate` / `generate_typst_source` entry points).
fn validate_worksheet_params(params: &WorksheetParams) -> Result<()> {
    if params.cols == 0 {
        bail!("cols must be at least 1, got 0");
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
                bail!("division drill supports max 3 columns, got {}", params.cols);
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
                bail!("whole range must be 2-99 with min ≤ max, got {min_whole}-{max_whole}");
            }
        }
        WorksheetType::FractionSimplify {
            denominators,
            max_numerator,
            ..
        } => {
            if denominators.is_empty() {
                bail!("denominators must have at least one value");
            }
            for &d in denominators {
                if d < 2 || d > 20 {
                    bail!("denominator must be 2-20, got {d}");
                }
            }
            if *max_numerator < 2 || *max_numerator > 99 {
                bail!("max-numerator must be 2-99, got {max_numerator}");
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
        WorksheetType::AlgebraOneStep {
            a_range,
            b_range,
            x_range,
            variable,
            add,
            subtract,
            multiply,
            divide,
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
            if variable.chars().count() != 1 {
                bail!("variable must be a single character, got {:?}", variable);
            }
            if !(*add || *subtract || *multiply || *divide) {
                bail!(
                    "at least one of add/subtract/multiply/divide must be enabled"
                );
            }
        }
        WorksheetType::FractionEquiv {
            denominators,
            scale,
            ..
        } => {
            if denominators.is_empty() {
                bail!("denominators must have at least one value");
            }
            for &d in denominators {
                if d < 2 || d > 20 {
                    bail!("denominator must be 2-20, got {d}");
                }
            }
            if scale.min < 2 || scale.max > 10 || scale.min > scale.max {
                bail!(
                    "scale must be 2-10 with min ≤ max, got {}-{}",
                    scale.min,
                    scale.max
                );
            }
        }
        WorksheetType::DecimalAdd {
            digits,
            decimal_places,
        } => {
            validate_digit_ranges(digits)?;
            validate_decimal_places(*decimal_places)?;
        }
        WorksheetType::DecimalSubtract {
            digits,
            decimal_places,
        } => {
            validate_digit_ranges(digits)?;
            if digits.len() > 2 {
                bail!(
                    "decimal subtract supports max 2 operands, got {}",
                    digits.len()
                );
            }
            validate_decimal_places(*decimal_places)?;
        }
        WorksheetType::DecimalMultiply {
            digits,
            decimal_places,
            multiplier_min,
            multiplier_max,
            bottom_decimal_places,
        } => {
            validate_digit_range(*digits, "decimal multiply top operand")?;
            validate_decimal_places(*decimal_places)?;
            // Multiplier integer part 0-99. With bottom_decimal_places=0
            // and min=0 the multiplier could be 0 — students get the
            // trivial × 0 = 0 case but it isn't a generation failure.
            if *multiplier_max > 99 || multiplier_min > multiplier_max {
                bail!(
                    "multiplier integer part must be 0-99 with min ≤ max, got {multiplier_min}-{multiplier_max}"
                );
            }
            if *bottom_decimal_places > 4 {
                bail!(
                    "bottom-decimal-places must be 0-4, got {bottom_decimal_places}"
                );
            }
        }
        WorksheetType::AlgebraSquareRoot {
            b_range,
            variable,
            squares,
            roots,
        } => {
            if b_range.max > 99 || b_range.min > b_range.max {
                bail!(
                    "b-range must be 0-99 with min ≤ max, got {}-{}",
                    b_range.min,
                    b_range.max
                );
            }
            if variable.chars().count() != 1 {
                bail!("variable must be a single character, got {:?}", variable);
            }
            if !(*squares || *roots) {
                bail!("at least one of squares/roots must be enabled");
            }
        }
    };
    Ok(())
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

fn validate_decimal_places(dp: u32) -> Result<()> {
    if dp == 0 || dp > 4 {
        bail!("decimal-places must be 1-4, got {dp}");
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
        // A4 body = 21 − 1.5 − 1.5 wide. Height depends on chrome size.
        // Worst-case chrome (3.2cm) → margin.top = 0.8 + 0.7 + 3.2 = 4.7cm.
        let chrome_h = HEADER_HEIGHT_CM;
        let (w, h) = content_area_cm(Paper::A4, chrome_h);
        assert!((w - 18.0).abs() < 0.01);
        assert!((h - (29.7 - 4.7 - 2.2)).abs() < 0.01);
        let (w, h) = content_area_cm(Paper::Letter, chrome_h);
        assert!((w - 18.59).abs() < 0.01);
        assert!((h - (27.94 - 4.7 - 2.2)).abs() < 0.01);
    }

    #[test]
    fn chrome_height_legacy_compact_preserved() {
        // Name/Date only (no banner) keeps the legacy 1.5cm box for
        // existing visual-regression baselines.
        assert!((chrome_height_cm(true, false, false) - 1.5).abs() < 0.001);
    }

    #[test]
    fn chrome_height_drill_problem_page() {
        // Drill problem page: name + title (no instructions). Should
        // be ~1.55cm.
        let h = chrome_height_cm(true, true, false);
        assert!((h - (NAME_DATE_SECTION_CM + TITLE_SECTION_CM)).abs() < 0.001);
    }

    #[test]
    fn chrome_height_full_problem_page() {
        // Standard problem page: all three sections.
        let h = chrome_height_cm(true, true, true);
        assert!(
            (h - (NAME_DATE_SECTION_CM + TITLE_SECTION_CM + INSTRUCTIONS_SECTION_CM)).abs() < 0.001
        );
    }

    #[test]
    fn chrome_height_answer_key_title_only() {
        // Answer-key page: title only (no name/date, no instructions).
        let h = chrome_height_cm(false, true, false);
        assert!((h - TITLE_SECTION_CM).abs() < 0.001);
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
    fn document_from_params_rejects_zero_cols() {
        let params = WorksheetParams {
            worksheet: WorksheetType::Add {
                digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
                carry: CarryMode::Any,
                binary: false,
            },
            num_problems: 12,
            cols: 0,
            paper: Paper::A4,
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Locale::Us,
            solve_first: false,
            include_answers: false,
            student_name: None,
            instructions: None,
        };
        let err = Document::from_params(&params).unwrap_err().to_string();
        assert!(
            err.contains("cols must be at least 1"),
            "unexpected error: {err}"
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
                divide_operator: String::new(),
                width_cm: 2.6,
                answer_rows: 1,
                pad_width: 0,
                implicit: false,
                variable: "x".to_string(),
                decimal_places: Vec::new(),
                reserve_remainder: false,
            },
        };
        let chrome = Chrome {
            student_name: None,
            paper: Paper::A4,
            include_answers: false,
            debug: false,
            solve_first: false,
            instructions: None,
        };
        let doc = Document {
            sheet: sheet.clone(),
            cols: 7,
            chrome: chrome.clone(),
            cells_per_page: 7 * 6,
            pages: 1,
        };
        let err = doc.validate().unwrap_err().to_string();
        assert!(err.contains("too wide"), "unexpected error: {err}");

        // 6 cols should fit.
        let ok_doc = Document { cols: 6, ..doc };
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

        // division-long-d3-no-remainder: 3-digit dividend → 3.8 × 8.0.
        // With remainder=true the cell widens by 1.2cm to fit the
        // `Q r N` answer key suffix.
        let long_d3 = WorksheetType::LongDivision {
            digits: DigitRange::fixed(3),
            remainder: false,
        };
        assert_eq!(long_d3.cell_size_cm(3), (3.8, 8.0));
        let long_d3_rem = WorksheetType::LongDivision {
            digits: DigitRange::fixed(3),
            remainder: true,
        };
        assert_eq!(long_d3_rem.cell_size_cm(3), (5.0, 8.0));

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

        // algebra-one-step-add (small): 6.2 × 2.6 (all ranges ≤ 30).
        let one_step_small = WorksheetType::AlgebraOneStep {
            a_range: DigitRange::new(2, 10),
            b_range: DigitRange::new(1, 30),
            x_range: DigitRange::new(0, 20),
            variable: "x".into(),
            add: true,
            subtract: true,
            multiply: false,
            divide: false,
        };
        assert_eq!(one_step_small.cell_size_cm(2), (6.2, 2.6));

        // algebra-one-step-large (b up to 99): 6.5 × 2.6.
        let one_step_large = WorksheetType::AlgebraOneStep {
            a_range: DigitRange::new(2, 10),
            b_range: DigitRange::new(1, 99),
            x_range: DigitRange::new(0, 20),
            variable: "x".into(),
            add: true,
            subtract: true,
            multiply: false,
            divide: false,
        };
        assert_eq!(one_step_large.cell_size_cm(3), (6.5, 2.6));
    }
}

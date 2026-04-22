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

mod template;
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

#[derive(Debug, Clone)]
pub struct WorksheetParams {
    pub worksheet: WorksheetType,
    pub num_problems: u32,
    pub cols: u32,
    pub paper: String,
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

    let typ_source = match &params.worksheet {
        WorksheetType::Add { digits, .. } => {
            validate_digit_ranges(digits)?;
            add::generate_typ(params)?
        }
        WorksheetType::Subtract { digits, .. } => {
            validate_digit_ranges(digits)?;
            if digits.len() > 2 {
                bail!("subtract supports max 2 operands, got {}", digits.len());
            }
            subtract::generate_typ(params)?
        }
        WorksheetType::Multiply { digits } => {
            validate_digit_ranges(digits)?;
            if digits.len() > 2 {
                bail!("multiply supports max 2 operands, got {}", digits.len());
            }
            multiply::generate_typ(params)?
        }
        WorksheetType::SimpleDivision { max_quotient } => {
            validate_max_quotient(*max_quotient)?;
            divide::generate_simple(params)?
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
            divide::generate_long(params)?
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
            mult_drill::generate_typ(params)?
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
            div_drill::generate_typ(params)?
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
            fraction_mult::generate_typ(params)?
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
                bail!("b-range must be 0-99 with min ≤ max, got {}-{}", b_range.min, b_range.max);
            }
            if x_range.max > 99 || x_range.min > x_range.max {
                bail!("x-range must be 0-99 with min ≤ max, got {}-{}", x_range.min, x_range.max);
            }
            // Variable must be exactly one unicode scalar (a single letter,
            // symbol, or single-codepoint emoji like 🍌). Compound emoji
            // sequences (flags, ZWJ) aren't supported.
            if variable.chars().count() != 1 {
                bail!("variable must be a single character, got {:?}", variable);
            }
            algebra_two_step::generate_typ(params)?
        }
    };
    Ok(typ_source)
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

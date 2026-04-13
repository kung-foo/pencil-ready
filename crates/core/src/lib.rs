//! mathsheet-core: worksheet generation and typst compilation.
//!
//! Each worksheet type lives in its own module.
//! They share a common template renderer and the typst World implementation.

mod add;
mod div_drill;
mod divide;
mod fraction_mult;
mod mult_drill;
mod multiply;
mod subtract;

mod template;
mod world;

use anyhow::{Result, bail};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigitRange {
    pub min: u32,
    pub max: u32,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CarryMode {
    None,
    #[default]
    Any,
    Force,
    Ripple,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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
        /// If true, render the first problem as a worked example (shows
        /// the multiply-across intermediate and simplified integer).
        solve_first: bool,
    },
}

/// Regional defaults for operator symbols in horizontal layouts.
///
/// Vertical/bracket layouts use universal notation regardless of locale.
/// Locale only affects horizontal layouts (drills).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct WorksheetParams {
    pub worksheet: WorksheetType,
    pub num_problems: u32,
    pub cols: u32,
    pub font: String,
    pub paper: String,
    pub debug: bool,
    pub seed: Option<u64>,
    /// Explicit symbol override. Takes precedence over locale.
    pub symbol: Option<String>,
    pub locale: Locale,
    /// Number of pages. Each page has `num_problems` unique problems.
    /// pages > 1 requires PDF output.
    pub pages: u32,
}

impl WorksheetParams {
    /// Total number of unique problems across all pages.
    pub fn total_problems(&self) -> u32 {
        self.num_problems * self.pages
    }
}

#[derive(Debug, Clone, Copy)]
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
) -> Result<Vec<u8>> {
    world::compile_and_export(typ_source, format, root)
}

pub fn generate(
    params: &WorksheetParams,
    format: OutputFormat,
    root: &std::path::Path,
) -> Result<Worksheet> {
    if params.pages == 0 {
        bail!("pages must be at least 1");
    }
    if params.pages > 1 && !matches!(format, OutputFormat::Pdf) {
        bail!("pages > 1 requires PDF output (PNG/SVG are single-image formats)");
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
    };

    let bytes = world::compile_and_export(&typ_source, format, root)?;
    Ok(Worksheet { bytes, format })
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

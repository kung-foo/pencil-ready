//! CLI for generating math worksheets.
//!
//! Usage:
//!   mathsheet add --digits 2,2,2 --carry ripple
//!   mathsheet subtract --digits 3,2 --borrow none
//!   mathsheet multiply --digits 2,2
//!   mathsheet simple-divide --max-quotient 12
//!   mathsheet long-divide --digits 2-4 --remainder
//!   mathsheet mult-drill --multiplicand 2,3
//!   mathsheet algebra-two-step --solve-first

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use mathsheet_core::{
    BorrowMode, CarryMode, DigitRange, Locale, OutputFormat, WorksheetParams, WorksheetType,
    generate,
};

#[derive(Clone, Copy, Default, ValueEnum)]
enum CliCarryMode {
    None,
    #[default]
    Any,
    Force,
    Ripple,
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum CliBorrowMode {
    None,
    NoAcrossZero,
    #[default]
    Any,
    Force,
    Ripple,
}

impl From<CliCarryMode> for CarryMode {
    fn from(m: CliCarryMode) -> Self {
        match m {
            CliCarryMode::None => CarryMode::None,
            CliCarryMode::Any => CarryMode::Any,
            CliCarryMode::Force => CarryMode::Force,
            CliCarryMode::Ripple => CarryMode::Ripple,
        }
    }
}

impl From<CliBorrowMode> for BorrowMode {
    fn from(m: CliBorrowMode) -> Self {
        match m {
            CliBorrowMode::None => BorrowMode::None,
            CliBorrowMode::NoAcrossZero => BorrowMode::NoAcrossZero,
            CliBorrowMode::Any => BorrowMode::Any,
            CliBorrowMode::Force => BorrowMode::Force,
            CliBorrowMode::Ripple => BorrowMode::Ripple,
        }
    }
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum CliLocale {
    /// US defaults (× for multiply, ÷ for divide)
    #[default]
    Us,
    /// Norwegian defaults (· for multiply, : for divide)
    No,
}

impl From<CliLocale> for Locale {
    fn from(l: CliLocale) -> Self {
        match l {
            CliLocale::Us => Locale::Us,
            CliLocale::No => Locale::No,
        }
    }
}

/// Flags that apply to every subcommand. Problem count and grid columns
/// are *not* here — each subcommand opts in to those individually, since
/// some have fixed layouts (algebra-two-step, fraction-mult) or drive
/// problem count from their own args (drills use --count).
#[derive(Parser)]
struct GlobalArgs {
    /// Number of pages (PDF only). Each page has the same number of problems.
    #[arg(long, default_value = "1")]
    pages: u32,

    #[arg(long, default_value = "a4")]
    paper: String,

    #[arg(long)]
    seed: Option<u64>,

    #[arg(short, long, default_value = "output/worksheet")]
    output: String,

    #[arg(long, default_value = "all")]
    format: String,

    /// Override the operator symbol (typst expression, e.g. "sym.colon")
    #[arg(long)]
    symbol: Option<String>,

    /// Locale for regional symbol defaults: us, no
    #[arg(long, value_enum, default_value = "us")]
    locale: CliLocale,

    #[arg(long)]
    debug: bool,

    #[arg(long, default_value = ".")]
    root: PathBuf,
}

// --- Subcommands ---

#[derive(Subcommand)]
enum Command {
    /// Addition worksheet
    Add {
        #[command(flatten)]
        global: GlobalArgs,

        #[arg(long, default_value = "12")]
        problems: u32,

        #[arg(long, default_value = "4")]
        cols: u32,

        /// Digits per operand, comma-separated. Use N for fixed or N-M for range.
        /// e.g. "2,2" or "2-4,2-4" or "2,2,2"
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::fixed(2), DigitRange::fixed(2)])]
        digits: Vec<DigitRange>,

        /// Carry constraint: none, any, force, ripple
        #[arg(long, value_enum, default_value = "any")]
        carry: CliCarryMode,
    },

    /// Subtraction worksheet
    Subtract {
        #[command(flatten)]
        global: GlobalArgs,

        #[arg(long, default_value = "12")]
        problems: u32,

        #[arg(long, default_value = "4")]
        cols: u32,

        /// Digits per operand, comma-separated. Use N for fixed or N-M for range.
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::fixed(2), DigitRange::fixed(2)])]
        digits: Vec<DigitRange>,

        /// Borrow constraint: none, no-across-zero, any, force, ripple
        #[arg(long, value_enum, default_value = "any")]
        borrow: CliBorrowMode,
    },

    /// Multiplication worksheet
    Multiply {
        #[command(flatten)]
        global: GlobalArgs,

        #[arg(long, default_value = "12")]
        problems: u32,

        #[arg(long, default_value = "4")]
        cols: u32,

        /// Digits per operand, comma-separated. Use N for fixed or N-M for range.
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::fixed(2), DigitRange::fixed(2)])]
        digits: Vec<DigitRange>,
    },

    /// Simple division (vertical layout with ÷)
    SimpleDivide {
        #[command(flatten)]
        global: GlobalArgs,

        #[arg(long, default_value = "12")]
        problems: u32,

        #[arg(long, default_value = "4")]
        cols: u32,

        /// Max quotient (answer) for generated problems (2-12)
        #[arg(long, default_value = "10")]
        max_quotient: u32,
    },

    /// Long division (bracket notation)
    LongDivide {
        #[command(flatten)]
        global: GlobalArgs,

        #[arg(long, default_value = "12")]
        problems: u32,

        #[arg(long, default_value = "4")]
        cols: u32,

        /// Dividend digits: N for fixed or N-M for range. e.g. "3" or "2-4"
        #[arg(long, default_value = "3")]
        digits: DigitRange,

        /// Allow problems with remainders
        #[arg(long)]
        remainder: bool,
    },

    /// Multiplication drill (horizontal times-table recall)
    MultDrill {
        #[command(flatten)]
        global: GlobalArgs,

        /// Columns in the problem grid (drills are wide, so 3 is the max).
        #[arg(long, default_value = "3")]
        cols: u32,

        /// Which tables to drill, comma-separated. e.g. "2,3" or "1-10"
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::new(1, 10)])]
        multiplicand: Vec<DigitRange>,

        /// Range of the other factor. e.g. "1-10" or "1-12"
        #[arg(long, default_value = "1-10")]
        multiplier: DigitRange,

        /// Number of problems (0 = all problems from the enumerated table).
        #[arg(long, default_value = "0")]
        count: u32,
    },

    /// Division drill (horizontal times-table recall, reversed)
    DivDrill {
        #[command(flatten)]
        global: GlobalArgs,

        /// Columns in the problem grid (drills are wide, so 3 is the max).
        #[arg(long, default_value = "3")]
        cols: u32,

        /// Which divisors to drill, comma-separated. e.g. "2,3" or "1-10"
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::new(1, 10)])]
        divisor: Vec<DigitRange>,

        /// Range of the quotient. e.g. "1-10" or "1-12"
        #[arg(long, default_value = "1-10")]
        max_quotient: DigitRange,

        /// Number of problems (0 = all problems from the enumerated table).
        #[arg(long, default_value = "0")]
        count: u32,
    },

    /// Two-step linear equation (ax + b = c, solve for x)
    ///
    /// Always 6 problems × 2 columns — the equations are wide and the
    /// three-row layout needs vertical breathing room.
    AlgebraTwoStep {
        #[command(flatten)]
        global: GlobalArgs,

        /// Coefficient range, e.g. "2-10"
        #[arg(long, default_value = "2-10")]
        a_range: DigitRange,

        /// Constant range, e.g. "1-30"
        #[arg(long, default_value = "1-30")]
        b_range: DigitRange,

        /// Answer (x) range, e.g. "0-20"
        #[arg(long, default_value = "0-20")]
        x_range: DigitRange,

        /// Render coefficient-variable as `4x` (implicit). Default: explicit
        /// operator (`4 · x`).
        #[arg(long)]
        implicit: bool,

        /// Variable glyph to solve for. Default: x. Any single character
        /// (letter, symbol, or emoji) works as long as a loaded font renders it.
        #[arg(long, default_value = "x")]
        variable: String,

        /// Randomly mix canonical (ax + b = c), const-first (b + ax = c),
        /// and subtraction (ax − b = c) equation forms within a worksheet.
        #[arg(long, default_value = "true")]
        mix_forms: bool,

        /// Render the first problem as a worked example
        #[arg(long)]
        solve_first: bool,
    },

    /// Fraction multiplication (whole × num/den = ___, integer answers)
    ///
    /// Always 3 columns — the problem renders as a horizontal fraction
    /// expression that needs a narrower cell.
    FractionMult {
        #[command(flatten)]
        global: GlobalArgs,

        #[arg(long, default_value = "12")]
        problems: u32,

        /// Allowed denominators, comma-separated. e.g. "2,3,4,5,10"
        #[arg(long, value_delimiter = ',', default_values_t = [2, 3, 4, 5, 10])]
        denominators: Vec<u32>,

        /// Minimum whole-number value
        #[arg(long, default_value = "2")]
        min_whole: u32,

        /// Maximum whole-number value
        #[arg(long, default_value = "20")]
        max_whole: u32,

        /// Use only unit fractions (numerator always 1)
        #[arg(long)]
        unit_only: bool,

        /// Render the first problem as a worked example (shows intermediate
        /// fraction and simplified integer)
        #[arg(long)]
        solve_first: bool,
    },
}

#[derive(Parser)]
#[command(name = "mathsheet", about = "Generate printable math worksheets")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// A normalized "resolved" configuration for a subcommand — the CLI enum
/// is destructured into these pieces once, then we build WorksheetParams
/// and drive the generator. Each subcommand decides what problem count
/// and column count to use.
struct Resolved {
    global: GlobalArgs,
    num_problems: u32,
    cols: u32,
    worksheet: WorksheetType,
}

fn resolve(command: Command) -> Resolved {
    match command {
        Command::Add { global, problems, cols, digits, carry } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::Add { digits, carry: carry.into() },
        },
        Command::Subtract { global, problems, cols, digits, borrow } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::Subtract { digits, borrow: borrow.into() },
        },
        Command::Multiply { global, problems, cols, digits } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::Multiply { digits },
        },
        Command::SimpleDivide { global, problems, cols, max_quotient } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::SimpleDivision { max_quotient },
        },
        Command::LongDivide { global, problems, cols, digits, remainder } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::LongDivision { digits, remainder },
        },
        Command::MultDrill { global, cols, multiplicand, multiplier, count } => Resolved {
            global,
            num_problems: count,
            cols,
            worksheet: WorksheetType::MultiplicationDrill { multiplicand, multiplier },
        },
        Command::DivDrill { global, cols, divisor, max_quotient, count } => Resolved {
            global,
            num_problems: count,
            cols,
            worksheet: WorksheetType::DivisionDrill { divisor, max_quotient },
        },
        Command::AlgebraTwoStep {
            global, a_range, b_range, x_range, implicit, variable, mix_forms, solve_first,
        } => Resolved {
            global,
            num_problems: 6,
            cols: 2,
            worksheet: WorksheetType::AlgebraTwoStep {
                a_range, b_range, x_range, variable, implicit, mix_forms, solve_first,
            },
        },
        Command::FractionMult {
            global, problems, denominators, min_whole, max_whole, unit_only, solve_first,
        } => Resolved {
            global,
            num_problems: problems,
            cols: 3,
            worksheet: WorksheetType::FractionMultiply {
                denominators, min_whole, max_whole, unit_only, solve_first,
            },
        },
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let Resolved { global, num_problems, cols, worksheet } = resolve(cli.command);

    let params = WorksheetParams {
        worksheet,
        num_problems,
        cols,
        paper: global.paper,
        debug: global.debug,
        seed: global.seed,
        symbol: global.symbol,
        locale: global.locale.into(),
        pages: global.pages,
    };

    let root = global
        .root
        .canonicalize()
        .context("could not resolve project root")?;

    let formats = match global.format.as_str() {
        "all" => vec![OutputFormat::Pdf, OutputFormat::Png, OutputFormat::Svg],
        "pdf" => vec![OutputFormat::Pdf],
        "png" => vec![OutputFormat::Png],
        "svg" => vec![OutputFormat::Svg],
        other => bail!("unknown format: {other} (choose: pdf, png, svg, all)"),
    };

    for format in &formats {
        let ext = match format {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Png => "png",
            OutputFormat::Svg => "svg",
        };
        let out_path = format!("{}.{}", global.output, ext);

        let worksheet = generate(&params, *format, &root)
            .with_context(|| format!("generating {ext}"))?;

        if let Some(parent) = std::path::Path::new(&out_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&out_path, &worksheet.bytes)
            .with_context(|| format!("writing {out_path}"))?;

        println!("wrote {out_path}");
    }

    Ok(())
}

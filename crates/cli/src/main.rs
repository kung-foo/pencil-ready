//! CLI for generating math worksheets.
//!
//! Usage:
//!   mathsheet add --digits 2,2,2 --carry ripple
//!   mathsheet add --digits 2-4,2-4 --carry force
//!   mathsheet subtract --digits 3,2 --borrow none
//!   mathsheet multiply --digits 2,2
//!   mathsheet simple-divide --max-quotient 12
//!   mathsheet long-divide --digits 2-4 --remainder

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

// --- Shared flags ---

#[derive(Parser)]
struct SharedArgs {
    #[arg(long, default_value = "12")]
    problems: u32,

    #[arg(long, default_value = "4")]
    cols: u32,

    #[arg(long, default_value = "B612")]
    font: String,

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
        shared: SharedArgs,

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
        shared: SharedArgs,

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
        shared: SharedArgs,

        /// Digits per operand, comma-separated. Use N for fixed or N-M for range.
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::fixed(2), DigitRange::fixed(2)])]
        digits: Vec<DigitRange>,
    },

    /// Simple division (vertical layout with ÷)
    SimpleDivide {
        #[command(flatten)]
        shared: SharedArgs,

        /// Max quotient (answer) for generated problems (2-12)
        #[arg(long, default_value = "10")]
        max_quotient: u32,
    },

    /// Long division (bracket notation)
    LongDivide {
        #[command(flatten)]
        shared: SharedArgs,

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
        shared: SharedArgs,

        /// Which tables to drill, comma-separated. e.g. "2,3" or "1-10"
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::new(1, 10)])]
        multiplicand: Vec<DigitRange>,

        /// Range of the other factor. e.g. "1-10" or "1-12"
        #[arg(long, default_value = "1-10")]
        multiplier: DigitRange,

        /// Number of problems (0 = all problems from the table). Overrides --problems.
        #[arg(long, default_value = "0")]
        count: u32,
    },

    /// Division drill (horizontal times-table recall, reversed)
    DivDrill {
        #[command(flatten)]
        shared: SharedArgs,

        /// Which divisors to drill, comma-separated. e.g. "2,3" or "1-10"
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::new(1, 10)])]
        divisor: Vec<DigitRange>,

        /// Range of the quotient. e.g. "1-10" or "1-12"
        #[arg(long, default_value = "1-10")]
        max_quotient: DigitRange,

        /// Number of problems (0 = all problems from the table). Overrides --problems.
        #[arg(long, default_value = "0")]
        count: u32,
    },
}

#[derive(Parser)]
#[command(name = "mathsheet", about = "Generate printable math worksheets")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (shared, worksheet) = match cli.command {
        Command::Add { shared, digits, carry } => (
            shared,
            WorksheetType::Add { digits, carry: carry.into() },
        ),
        Command::Subtract { shared, digits, borrow } => (
            shared,
            WorksheetType::Subtract { digits, borrow: borrow.into() },
        ),
        Command::Multiply { shared, digits } => (
            shared,
            WorksheetType::Multiply { digits },
        ),
        Command::SimpleDivide { shared, max_quotient } => (
            shared,
            WorksheetType::SimpleDivision { max_quotient },
        ),
        Command::LongDivide { shared, digits, remainder } => (
            shared,
            WorksheetType::LongDivision { digits, remainder },
        ),
        Command::MultDrill { mut shared, multiplicand, multiplier, count } => {
            if shared.cols == 4 {
                shared.cols = 3;
            }
            shared.problems = count;
            (shared, WorksheetType::MultiplicationDrill { multiplicand, multiplier })
        }
        Command::DivDrill { mut shared, divisor, max_quotient, count } => {
            if shared.cols == 4 {
                shared.cols = 3;
            }
            shared.problems = count;
            (shared, WorksheetType::DivisionDrill { divisor, max_quotient })
        }
    };

    let params = WorksheetParams {
        worksheet,
        num_problems: shared.problems,
        cols: shared.cols,
        font: shared.font,
        paper: shared.paper,
        debug: shared.debug,
        seed: shared.seed,
        symbol: shared.symbol,
        locale: shared.locale.into(),
    };

    let root = shared
        .root
        .canonicalize()
        .context("could not resolve project root")?;

    let formats = match shared.format.as_str() {
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
        let out_path = format!("{}.{}", shared.output, ext);

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

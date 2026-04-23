//! CLI for generating math worksheets.
//!
//! Usage:
//!   pencil-ready add --digits 2,2,2 --carry ripple
//!   pencil-ready subtract --digits 3,2 --borrow none
//!   pencil-ready multiply --digits 2,2
//!   pencil-ready simple-divide --max-quotient 12
//!   pencil-ready long-divide --digits 2-4 --remainder
//!   pencil-ready mult-drill --multiplicand 2,3
//!   pencil-ready algebra-two-step --solve-first

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

use pencil_ready_core::{
    BorrowMode, CarryMode, DigitRange, Fonts, Locale, OutputFormat, Paper, WorksheetParams,
    WorksheetType, compile_typst, generate, generate_typst_source,
};

fn parse_paper(s: &str) -> Result<Paper, String> {
    s.parse()
}

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
    /// Paper size. Accepts "a4" (default) or "us-letter" (alias: "letter").
    #[arg(long, default_value = "a4", value_parser = parse_paper)]
    paper: Paper,

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

    /// Render the first problem as a worked example. Each worksheet type
    /// decides what "solved" means (filled-in answer, partial products,
    /// step-by-step work, etc.).
    #[arg(long)]
    solve_first: bool,

    /// Append an answer-key page (or pages, one per problem page) showing
    /// just the correct answer for each problem. PDF only.
    #[arg(long)]
    include_answers: bool,

    /// Pre-fill the student name on the header in a handwriting font.
    /// When set, the Name signature line is replaced with this text.
    #[arg(long)]
    student_name: Option<String>,

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
        /// e.g. "2,2" or "2-4,2-4" or "2,2,2". In --binary mode this is
        /// interpreted as the *bit* count per operand (4 is a good default).
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::fixed(2), DigitRange::fixed(2)])]
        digits: Vec<DigitRange>,

        /// Carry constraint: none, any, force, ripple
        #[arg(long, value_enum, default_value = "any")]
        carry: CliCarryMode,

        /// Base-2 mode. Operands take values in [0, 2^d − 1] and render
        /// in binary with leading-zero padding. Pair with --digits 4,4
        /// (or similar) since small bit-counts are trivial.
        #[arg(long)]
        binary: bool,
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
        #[arg(long, value_delimiter = ',', default_values_t = [DigitRange::new(2, 9)])]
        divisor: Vec<DigitRange>,

        /// Range of the quotient. e.g. "1-10" or "1-12"
        #[arg(long, default_value = "2-9")]
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
    },

    /// Fraction simplification (num/den = ___, simplest form)
    ///
    /// Answer is one of: reduced proper fraction, mixed number, the
    /// same fraction (already in lowest terms), or a whole number
    /// (only with --include-whole).
    FractionSimplify {
        #[command(flatten)]
        global: GlobalArgs,

        #[arg(long, default_value = "12")]
        problems: u32,

        #[arg(long, default_value = "3")]
        cols: u32,

        /// Allowed denominators of the printed fraction, comma-separated.
        #[arg(long, value_delimiter = ',', default_values_t = [2, 3, 4, 5, 6, 8, 10, 12])]
        denominators: Vec<u32>,

        /// Maximum numerator of the printed fraction. Larger than the
        /// largest denominator lets improper fractions appear.
        #[arg(long, default_value = "20")]
        max_numerator: u32,

        /// Exclude improper fractions — only proper (num < den) problems.
        /// Default off: a good worksheet mixes proper and improper.
        #[arg(long)]
        proper_only: bool,

        /// Include problems whose answer is a pure whole number
        /// (e.g. 12/4 → 3). Default off.
        #[arg(long)]
        include_whole: bool,
    },

    /// Generate a multi-page PDF with one of each worksheet type.
    ///
    /// All worksheets use their defaults plus --solve-first and --seed 42
    /// so the output is reproducible and the first problem on each page is
    /// worked as an example.
    All {
        #[command(flatten)]
        global: GlobalArgs,
    },
}

#[derive(Parser)]
#[command(name = "pencil-ready", about = "Generate printable math worksheets")]
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

/// Convert a parsed CLI `Command` into a normalized `Resolved` configuration.
///
/// This maps each `Command` variant into a `Resolved` struct containing:
/// - `global`: the shared `GlobalArgs`,
/// - `num_problems`: the effective number of problems (CLI-provided or command-fixed),
/// - `cols`: the effective column count (CLI-provided or command-fixed),
/// - `worksheet`: the corresponding `WorksheetType` populated with the command's parameters.
///
/// The `Command::All` variant is not expected here and is marked unreachable.
fn resolve(command: Command) -> Resolved {
    match command {
        Command::Add {
            global,
            problems,
            cols,
            digits,
            carry,
            binary,
        } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::Add {
                digits,
                carry: carry.into(),
                binary,
            },
        },
        Command::Subtract {
            global,
            problems,
            cols,
            digits,
            borrow,
        } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::Subtract {
                digits,
                borrow: borrow.into(),
            },
        },
        Command::Multiply {
            global,
            problems,
            cols,
            digits,
        } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::Multiply { digits },
        },
        Command::SimpleDivide {
            global,
            problems,
            cols,
            max_quotient,
        } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::SimpleDivision { max_quotient },
        },
        Command::LongDivide {
            global,
            problems,
            cols,
            digits,
            remainder,
        } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::LongDivision { digits, remainder },
        },
        Command::MultDrill {
            global,
            cols,
            multiplicand,
            multiplier,
            count,
        } => Resolved {
            global,
            num_problems: count,
            cols,
            worksheet: WorksheetType::MultiplicationDrill {
                multiplicand,
                multiplier,
            },
        },
        Command::DivDrill {
            global,
            cols,
            divisor,
            max_quotient,
            count,
        } => Resolved {
            global,
            num_problems: count,
            cols,
            worksheet: WorksheetType::DivisionDrill {
                divisor,
                max_quotient,
            },
        },
        Command::AlgebraTwoStep {
            global,
            a_range,
            b_range,
            x_range,
            implicit,
            variable,
            mix_forms,
        } => Resolved {
            global,
            num_problems: 6,
            cols: 2,
            worksheet: WorksheetType::AlgebraTwoStep {
                a_range,
                b_range,
                x_range,
                variable,
                implicit,
                mix_forms,
            },
        },
        Command::FractionMult {
            global,
            problems,
            denominators,
            min_whole,
            max_whole,
            unit_only,
        } => Resolved {
            global,
            num_problems: problems,
            cols: 3,
            worksheet: WorksheetType::FractionMultiply {
                denominators,
                min_whole,
                max_whole,
                unit_only,
            },
        },
        Command::FractionSimplify {
            global,
            problems,
            cols,
            denominators,
            max_numerator,
            proper_only,
            include_whole,
        } => Resolved {
            global,
            num_problems: problems,
            cols,
            worksheet: WorksheetType::FractionSimplify {
                denominators,
                max_numerator,
                include_improper: !proper_only,
                include_whole,
            },
        },
        Command::All { .. } => unreachable!("Command::All is handled before resolve()"),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // `all` is a meta-command: it bundles one of each worksheet type into
    // a single multi-page PDF. Handled before the normal dispatch since it
    // doesn't map to a single WorksheetType.
    if let Command::All { global } = cli.command {
        return run_all(global);
    }

    let Resolved {
        global,
        num_problems,
        cols,
        worksheet,
    } = resolve(cli.command);

    let params = WorksheetParams {
        worksheet,
        num_problems,
        cols,
        paper: global.paper,
        debug: global.debug,
        seed: global.seed,
        symbol: global.symbol,
        locale: global.locale.into(),
        solve_first: global.solve_first,
        include_answers: global.include_answers,
        student_name: global.student_name,
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

    let fonts = Fonts::load(&root).context("loading fonts from <root>/fonts")?;

    for format in &formats {
        let ext = match format {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Png => "png",
            OutputFormat::Svg => "svg",
        };
        let out_path = format!("{}.{}", global.output, ext);

        let worksheet = generate(&params, *format, &root, &fonts)
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

/// Build a default `WorksheetParams` for a given worksheet type. Mirrors
/// what each subcommand's dispatch would produce, but without going
/// through clap — used only by the `all` meta-command.
fn default_params_for(
    worksheet: WorksheetType,
    num_problems: u32,
    cols: u32,
    global: &GlobalArgs,
) -> WorksheetParams {
    WorksheetParams {
        worksheet,
        num_problems,
        cols,
        paper: global.paper,
        debug: global.debug,
        // Ignore any --seed the caller passed: `all` must be reproducible.
        seed: Some(42),
        symbol: global.symbol.clone(),
        locale: global.locale.into(),
        // Force worked-example rendering for every type.
        solve_first: true,
        // `all` is a single-PDF sampler — don't duplicate pages with an
        // answer key.
        include_answers: false,
        student_name: global.student_name.clone(),
    }
}

/// Extract the body portion of a generated typst source — everything
/// after the imports and `#set page` / `#set text` preamble. Used so
/// the `all` command can concatenate multiple worksheets' bodies into
/// one document without duplicating the setup.
fn typst_body(source: &str) -> &str {
    let marker = "#set text(font: body-font, size: 10pt)\n\n";
    source
        .split_once(marker)
        .map(|(_, body)| body)
        .unwrap_or(source)
}

/// Generates a single multi-page PDF that contains one page-set of each worksheet type and writes it to `<global.output>.pdf`.
///
/// The document is constructed by generating each worksheet's Typst body, concatenating them with page breaks, compiling the combined Typst to PDF, and saving the result at the path derived from `global.output`.
///
/// # Examples
///
/// ```
/// // Construct `GlobalArgs` with desired output path and options, then generate the combined PDF.
/// let global = GlobalArgs { output: "output/all_worksheets".into(), ..Default::default() };
/// run_all(global).unwrap();
/// assert!(std::path::Path::new("output/all_worksheets.pdf").exists());
/// ```
fn run_all(global: GlobalArgs) -> Result<()> {
    // One entry per worksheet type. Column and problem counts mirror each
    // subcommand's dispatch. Keep in sync with `resolve()`.
    let sheets: Vec<(&str, WorksheetType, u32, u32)> = vec![
        (
            "add",
            WorksheetType::Add {
                digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
                carry: CarryMode::Any,
                binary: false,
            },
            12,
            4,
        ),
        (
            "subtract",
            WorksheetType::Subtract {
                digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
                borrow: BorrowMode::Any,
            },
            12,
            4,
        ),
        (
            "multiply",
            WorksheetType::Multiply {
                digits: vec![DigitRange::fixed(2), DigitRange::fixed(2)],
            },
            12,
            4,
        ),
        (
            "simple-divide",
            WorksheetType::SimpleDivision { max_quotient: 10 },
            12,
            4,
        ),
        (
            "long-divide",
            WorksheetType::LongDivision {
                digits: DigitRange::fixed(3),
                remainder: false,
            },
            12,
            4,
        ),
        (
            "mult-drill",
            WorksheetType::MultiplicationDrill {
                multiplicand: vec![DigitRange::new(1, 10)],
                multiplier: DigitRange::new(1, 10),
            },
            // Drills: 0 = all problems in the enumerated table.
            0,
            3,
        ),
        (
            "div-drill",
            WorksheetType::DivisionDrill {
                divisor: vec![DigitRange::new(2, 9)],
                max_quotient: DigitRange::new(2, 9),
            },
            0,
            3,
        ),
        (
            "fraction-mult",
            WorksheetType::FractionMultiply {
                denominators: vec![2, 3, 4, 5, 10],
                min_whole: 2,
                max_whole: 20,
                unit_only: false,
            },
            12,
            3,
        ),
        (
            "fraction-simplify",
            WorksheetType::FractionSimplify {
                denominators: vec![2, 3, 4, 5, 6, 8, 10, 12],
                max_numerator: 20,
                include_improper: true,
                include_whole: false,
            },
            12,
            3,
        ),
        (
            "algebra-two-step",
            WorksheetType::AlgebraTwoStep {
                a_range: DigitRange::new(2, 10),
                b_range: DigitRange::new(1, 30),
                x_range: DigitRange::new(0, 20),
                variable: "x".into(),
                implicit: false,
                mix_forms: true,
            },
            6,
            2,
        ),
    ];

    let mut bodies = Vec::with_capacity(sheets.len());
    for (name, worksheet, num_problems, cols) in sheets {
        let params = default_params_for(worksheet, num_problems, cols, &global);
        let source = generate_typst_source(&params)
            .with_context(|| format!("generating source for {name}"))?;
        bodies.push(typst_body(&source).to_string());
    }

    // Wrap with the same preamble used by each individual worksheet and
    // join bodies with pagebreaks.
    let combined = format!(
        r#"#import "/lib/header.typ": worksheet-header
#import "/lib/page.typ": worksheet-page
#import "/lib/footer.typ": worksheet-footer, pencil-ready-content
#import "/lib/problems/shared.typ": body-font
// Problem components passed to worksheet-grid by reference must be
// in scope at the call site. (Mirrors document.rs's preamble.)
#import "/lib/problems/addition/basic.typ": addition-basic-problem
#import "/lib/problems/subtraction/basic.typ": subtraction-basic-problem
#import "/lib/problems/multiplication/basic.typ": multiplication-basic-problem
#import "/lib/problems/multiplication/drill.typ": multiplication-drill-problem
#import "/lib/problems/division/simple.typ": division-simple-problem
#import "/lib/problems/division/long.typ": division-long-problem
#import "/lib/problems/division/drill.typ": division-drill-problem
#import "/lib/problems/fraction/multiplication.typ": fraction-multiplication-problem
#import "/lib/problems/fraction/simplification.typ": fraction-simplification-problem
#import "/lib/problems/algebra/two-step.typ": algebra-two-step-problem

#set page(paper: "{paper}", margin: (top: 1.5cm, bottom: 1.0cm, left: 1.5cm, right: 1.5cm))
#set text(font: body-font, size: 10pt)

{body}"#,
        paper = global.paper,
        body = bodies.join("\n#pagebreak()\n\n"),
    );

    let root = global
        .root
        .canonicalize()
        .context("could not resolve project root")?;
    let fonts = Fonts::load(&root).context("loading fonts from <root>/fonts")?;

    let bytes = compile_typst(&combined, OutputFormat::Pdf, &root, &fonts)
        .context("compiling combined worksheet PDF")?;

    let out_path = format!("{}.pdf", global.output);
    if let Some(parent) = std::path::Path::new(&out_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, &bytes).with_context(|| format!("writing {out_path}"))?;
    println!("wrote {out_path}");
    Ok(())
}

//! HTTP server for worksheet generation.
//!
//! GET /api/worksheets/{type}?format=pdf&seed=42&...
//!
//! Each endpoint uses two typed query extractors — `SharedParams` for the
//! cross-cutting knobs (format, seed, paper, etc.) and an endpoint-specific
//! struct for the worksheet-type params. Each derives `utoipa::IntoParams`,
//! so the OpenAPI spec at /openapi.json stays in sync with the code.
//! Swagger UI at /docs.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use axum::{
    body::{Body, Bytes},
    extract::{Query, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use clap::Parser;
use pencil_ready_core::{
    BorrowMode, CarryMode, ConversionGiven, DenominatorRelation, DigitRange, DivisionStyle, Fonts, Locale, MissingSlot,
    OutputFormat,
    Paper,
    WorksheetParams, WorksheetType, factor_pairs_default_problems, generate, long_division_default_cols,
    long_division_default_problems,
};
use serde::Deserialize;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use utoipa::{IntoParams, OpenApi};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    root: PathBuf,
    /// Short region slug for the serving machine (e.g. `arn`, `sjc`).
    /// Populated from `FLY_REGION` at boot; `None` for local dev. Both
    /// the structured log and the PDF footer include it.
    region: Option<String>,
    /// Fonts parsed once at startup. Shared (via `Arc`) with every
    /// compile — avoids re-reading `fonts/` from disk per request.
    fonts: Fonts,
    /// Force `debug: true` on every worksheet render regardless of
    /// query params — turns the red/blue layout-debug borders on for
    /// every browser-facing render. Set via `--debug` / `PENCIL_READY_DEBUG`.
    force_debug: bool,
    /// Shared HTTP client for outbound proxy requests (Umami).
    umami_client: reqwest::Client,
    /// Lazily-fetched Umami tracker bundle. Fetched once from
    /// cloud.umami.is on first `/umami/script.js` request, then held
    /// for the lifetime of the process.
    umami_script: Arc<tokio::sync::Mutex<Option<Bytes>>>,
}

// ---------------------------------------------------------------------------
// Shared params
// ---------------------------------------------------------------------------

/// Cross-cutting query params common to every worksheet endpoint. Extracted
/// via its own `Query<SharedParams>` on each handler; unknown fields are
/// silently ignored so the type-specific extractor can pick them up.
#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SharedParams {
    /// Output format. Default `pdf`.
    #[serde(default)]
    format: Option<OutputFormat>,
    /// Seed for reproducible output. Omit for a random worksheet.
    #[serde(default)]
    seed: Option<u64>,
    /// Number of problems. Default depends on the worksheet type.
    #[serde(default)]
    problems: Option<u32>,
    /// Columns in the problem grid. Default depends on the worksheet type.
    #[serde(default)]
    cols: Option<u32>,
    /// Paper size. Default `a4`; `letter` is accepted as an alias for
    /// `us-letter`.
    #[serde(default)]
    paper: Option<Paper>,
    /// Printed language and number notation. Default `us`.
    #[serde(default)]
    locale: Option<Locale>,
    /// Division layout for long-divide, simple-divide and mean. Default
    /// follows `locale`: `bracket` for `us`, `equation` for `no`.
    #[serde(default)]
    division: Option<DivisionStyle>,
    /// Override the operator symbol, as a typst symbol expression
    /// (e.g. `sym.colon`). Default follows `locale`.
    #[serde(default)]
    symbol: Option<String>,
    /// Render the first problem as a worked example.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    solve_first: Option<bool>,
    /// Append an answer-key page showing just the final answers (PDF only).
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    include_answers: Option<bool>,
    /// Draw debug borders around problem boxes and grid cells.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    debug: Option<bool>,
    /// Pre-fill the student name on the header in a handwriting font.
    #[serde(default)]
    student_name: Option<String>,
    /// Print a QR code encoding `share_url` in the bottom-right of every
    /// page. Default false; has no effect without a valid `share_url`.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    qr: Option<bool>,
    /// URL encoded by the QR code when `qr=true`. Must start with
    /// `https://pencilready.com/`; other URLs are silently ignored.
    // Allow-list rationale: a printed worksheet lives forever, so an
    // arbitrary `share_url` would let any caller mint a scannable
    // redirect to anywhere. See `SHARE_URL_ALLOWED_PREFIXES`.
    #[serde(default)]
    share_url: Option<String>,
}

const SHARE_URL_ALLOWED_PREFIXES: &[&str] = &[
    "https://pencilready.com/",
    "http://localhost:",
    "http://127.0.0.1:",
];

fn validated_share_url(s: Option<String>) -> Option<String> {
    let s = s.filter(|v| !v.is_empty())?;
    if SHARE_URL_ALLOWED_PREFIXES.iter().any(|p| s.starts_with(p)) {
        Some(s)
    } else {
        None
    }
}

/// Loose boolean deserializer for query-string params. Accepts the
/// canonical `true`/`false` plus the common shortcuts `1`/`0`,
/// `yes`/`no`, `on`/`off` (case-insensitive). Lets `?qr=1` or
/// `?debug=on` work in addition to the strict serde default of
/// `?qr=true`. An empty string deserializes as `None` so URLs like
/// `?qr=` (param present, no value) don't break.
fn deserialize_loose_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Error, Visitor};
    use std::fmt;

    struct LooseBool;
    impl<'de> Visitor<'de> for LooseBool {
        type Value = Option<bool>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a boolean (true/false/1/0/yes/no/on/off)")
        }
        fn visit_bool<E: Error>(self, v: bool) -> Result<Self::Value, E> {
            Ok(Some(v))
        }
        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            match v.trim().to_ascii_lowercase().as_str() {
                "" => Ok(None),
                "true" | "1" | "yes" | "on" => Ok(Some(true)),
                "false" | "0" | "no" | "off" => Ok(Some(false)),
                other => Err(E::custom(format!(
                    "expected boolean (true/false/1/0/yes/no/on/off), got {other:?}"
                ))),
            }
        }
        fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
            self.visit_str(&v)
        }
        fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: serde::Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            deserializer.deserialize_any(LooseBool)
        }
    }
    deserializer.deserialize_any(LooseBool)
}

impl SharedParams {
    /// Explicit `division=` wins; otherwise the locale's default.
    fn division_style(&self) -> DivisionStyle {
        self.division
            .unwrap_or_else(|| self.locale.unwrap_or_default().division_style())
    }

    fn fold(
        self,
        worksheet: WorksheetType,
        default_problems: u32,
        default_cols: u32,
    ) -> (OutputFormat, WorksheetParams) {
        let format = self.format.unwrap_or(OutputFormat::Pdf);
        let params = WorksheetParams {
            worksheet,
            num_problems: self.problems.unwrap_or(default_problems),
            cols: self.cols.unwrap_or(default_cols),
            paper: self.paper.unwrap_or_default(),
            debug: self.debug.unwrap_or(false),
            seed: self.seed,
            symbol: self.symbol,
            locale: self.locale.unwrap_or_default(),
            solve_first: self.solve_first.unwrap_or(false),
            include_answers: self.include_answers.unwrap_or(false),
            student_name: self.student_name.filter(|s| !s.is_empty()),
            instructions: None,
            // `qr=true` gates rendering; without it, `share_url` is
            // ignored. Lets the same client send a share URL on every
            // request and toggle the QR independently.
            share_url: if self.qr.unwrap_or(false) {
                validated_share_url(self.share_url)
            } else {
                None
            },
        };
        (format, params)
    }
}

// ---------------------------------------------------------------------------
// CSV parsing helpers (query strings use `a,b,c` rather than repeated keys)
// ---------------------------------------------------------------------------

fn parse_digits_csv(s: &str, field: &str) -> Result<Vec<DigitRange>> {
    s.split(',')
        .map(|p| DigitRange::from_str(p.trim()).map_err(|e| anyhow!("{field}: {e}")))
        .collect()
}

fn parse_u32_csv(s: &str, field: &str) -> Result<Vec<u32>> {
    s.split(',')
        .map(|p| p.trim().parse::<u32>().map_err(|e| anyhow!("{field}: {e}")))
        .collect()
}

fn parse_digit_range(s: &str, field: &str) -> Result<DigitRange> {
    DigitRange::from_str(s).map_err(|e| anyhow!("{field}: {e}"))
}

fn digits_or(opt: Option<String>, default: &[u32]) -> Result<Vec<DigitRange>> {
    match opt {
        Some(s) => parse_digits_csv(&s, "digits"),
        None => Ok(default.iter().copied().map(DigitRange::fixed).collect()),
    }
}

// ---------------------------------------------------------------------------
// Per-endpoint type-specific params + builders
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AddSpecific {
    /// Digit count (range) per operand, comma-separated. Default `2,2`.
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
    /// Carrying constraint. Default `any`.
    #[serde(default)]
    carry: Option<CarryMode>,
    /// Render in base 2; `digits` become bit counts. Default false.
    #[serde(default)]
    binary: Option<bool>,
}

impl AddSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::Add {
                digits: digits_or(self.digits, &[2, 2])?,
                carry: self.carry.unwrap_or(CarryMode::Any),
                binary: self.binary.unwrap_or(false),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SubtractSpecific {
    /// Digit count (range) per operand, comma-separated. Default `2,2`.
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
    /// Borrowing constraint. Default `any`.
    #[serde(default)]
    borrow: Option<BorrowMode>,
}

impl SubtractSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::Subtract {
                digits: digits_or(self.digits, &[2, 2])?,
                borrow: self.borrow.unwrap_or(BorrowMode::Any),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct MultiplySpecific {
    /// Digit count (range) per operand, comma-separated. Default `2,2`.
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
}

impl MultiplySpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::Multiply {
                digits: digits_or(self.digits, &[2, 2])?,
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SimpleDivideSpecific {
    /// Largest quotient (the answer), 2-12. Default 10.
    #[serde(default)]
    max_quotient: Option<u32>,
}

impl SimpleDivideSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let style = shared.division_style();
        // One-line equation problems are short and wider than a stack:
        // more of them, three to a row.
        let (problems, cols) = match style {
            DivisionStyle::Bracket => (12, 4),
            DivisionStyle::Equation => (30, 3),
        };
        Ok(shared.fold(
            WorksheetType::SimpleDivision {
                max_quotient: self.max_quotient.unwrap_or(10),
                style,
            },
            problems,
            cols,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct LongDivideSpecific {
    /// Dividend digit count (range). Default `3`.
    #[serde(default)]
    #[param(value_type = String, example = "3")]
    digits: Option<String>,
    /// Allow problems with a remainder. Default false.
    #[serde(default)]
    remainder: Option<bool>,
}

impl LongDivideSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let digits = match self.digits {
            Some(s) => parse_digit_range(&s, "digits")?,
            None => DigitRange::fixed(3),
        };
        let style = shared.division_style();
        let remainder = self.remainder.unwrap_or(false);
        let cols = shared
            .cols
            .unwrap_or_else(|| long_division_default_cols(style, digits, remainder));
        Ok(shared.fold(
            WorksheetType::LongDivision {
                digits,
                remainder,
                style,
            },
            long_division_default_problems(style, digits, remainder, cols, 12),
            cols,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct MultDrillSpecific {
    /// Tables to drill: ranges, comma-separated (e.g. `2,3` or `1-10`).
    /// Default `1-10`.
    #[serde(default)]
    #[param(value_type = String, example = "1-10")]
    multiplicand: Option<String>,
    /// Range of the other factor. Default `1-10`.
    #[serde(default)]
    #[param(value_type = String, example = "1-10")]
    multiplier: Option<String>,
    /// Problem count; 0 = every fact in the chosen tables. Default 0.
    #[serde(default)]
    count: Option<u32>,
}

impl MultDrillSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let multiplicand = match self.multiplicand {
            Some(s) => parse_digits_csv(&s, "multiplicand")?,
            None => vec![DigitRange::new(1, 10)],
        };
        let multiplier = match self.multiplier {
            Some(s) => parse_digit_range(&s, "multiplier")?,
            None => DigitRange::new(1, 10),
        };
        let (fmt, mut params) = shared.fold(
            WorksheetType::MultiplicationDrill {
                multiplicand,
                multiplier,
            },
            0,
            3,
        );
        if let Some(c) = self.count {
            params.num_problems = c;
        }
        Ok((fmt, params))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DivDrillSpecific {
    /// Divisors to drill: ranges, comma-separated. Default `2-9`.
    #[serde(default)]
    #[param(value_type = String, example = "2-9")]
    divisor: Option<String>,
    /// Quotient range. Default `2-9`.
    #[serde(default)]
    #[param(value_type = String, example = "2-9")]
    max_quotient: Option<String>,
    /// Problem count; 0 = every fact in range. Default 0.
    #[serde(default)]
    count: Option<u32>,
}

impl DivDrillSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let divisor = match self.divisor {
            Some(s) => parse_digits_csv(&s, "divisor")?,
            None => vec![DigitRange::new(2, 9)],
        };
        let max_quotient = match self.max_quotient {
            Some(s) => parse_digit_range(&s, "max_quotient")?,
            None => DigitRange::new(2, 9),
        };
        let (fmt, mut params) = shared.fold(
            WorksheetType::DivisionDrill {
                divisor,
                max_quotient,
            },
            0,
            3,
        );
        if let Some(c) = self.count {
            params.num_problems = c;
        }
        Ok((fmt, params))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FractionMultSpecific {
    /// Allowed denominators, comma-separated. Default `2,3,4,5,10`.
    #[serde(default)]
    #[param(value_type = String, example = "2,3,4,5,10")]
    denominators: Option<String>,
    /// Smallest whole number. Default 2.
    #[serde(default)]
    min_whole: Option<u32>,
    /// Largest whole number. Default 20.
    #[serde(default)]
    max_whole: Option<u32>,
    /// Numerator is always 1. Default false.
    #[serde(default)]
    unit_only: Option<bool>,
}

impl FractionMultSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let denominators = match self.denominators {
            Some(s) => parse_u32_csv(&s, "denominators")?,
            None => vec![2, 3, 4, 5, 10],
        };
        Ok(shared.fold(
            WorksheetType::FractionMultiply {
                denominators,
                min_whole: self.min_whole.unwrap_or(2),
                max_whole: self.max_whole.unwrap_or(20),
                unit_only: self.unit_only.unwrap_or(false),
            },
            12,
            3,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FractionSimplifySpecific {
    /// Allowed denominators of the printed fraction, comma-separated.
    /// Default `2,3,4,5,6,8,10,12`.
    #[serde(default)]
    #[param(value_type = String, example = "2,3,4,5,6,8,10,12")]
    denominators: Option<String>,
    /// Largest numerator of the printed fraction. Default 20.
    #[serde(default)]
    max_numerator: Option<u32>,
    /// Only proper fractions (numerator < denominator), so no mixed-number
    /// answers. Default false.
    // Polarity is flipped so the default (mixed proper + improper) is
    // the false case and round-trips cleanly through the URL encoder.
    #[serde(default)]
    proper_only: Option<bool>,
    /// Include problems whose answer is a whole number. Default false.
    #[serde(default)]
    include_whole: Option<bool>,
}

impl FractionSimplifySpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let denominators = match self.denominators {
            Some(s) => parse_u32_csv(&s, "denominators")?,
            None => vec![2, 3, 4, 5, 6, 8, 10, 12],
        };
        Ok(shared.fold(
            WorksheetType::FractionSimplify {
                denominators,
                max_numerator: self.max_numerator.unwrap_or(20),
                include_improper: !self.proper_only.unwrap_or(false),
                include_whole: self.include_whole.unwrap_or(false),
            },
            12,
            3,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AlgebraSquareRootSpecific {
    /// Constant range. Default `1-50`.
    #[serde(default)]
    #[param(value_type = String, example = "1-30")]
    b_range: Option<String>,
    /// Variable letter. Default `x`.
    #[serde(default)]
    variable: Option<String>,
    /// Include `x² ± b = c` problems. Default false.
    #[serde(default)]
    squares: Option<bool>,
    /// Include `√x ± b = c` problems. Default false.
    #[serde(default)]
    roots: Option<bool>,
}

impl AlgebraSquareRootSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let b_range = match self.b_range {
            Some(s) => parse_digit_range(&s, "b_range")?,
            None => DigitRange::new(1, 50),
        };
        Ok(shared.fold(
            WorksheetType::AlgebraSquareRoot {
                b_range,
                variable: self.variable.unwrap_or_else(|| "x".into()),
                // Same "absence == off" convention as one-step. Validation
                // rejects "neither" with a readable error, matching the
                // CLI's at-least-one rule.
                squares: self.squares.unwrap_or(false),
                roots: self.roots.unwrap_or(false),
            },
            8,
            2,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AlgebraTwoStepSpecific {
    /// Coefficient range. Default `2-10`.
    #[serde(default)]
    #[param(value_type = String, example = "2-10")]
    a_range: Option<String>,
    /// Constant range. Default `1-30`.
    #[serde(default)]
    #[param(value_type = String, example = "1-30")]
    b_range: Option<String>,
    /// Answer (`x`) range. Default `0-20`.
    #[serde(default)]
    #[param(value_type = String, example = "0-20")]
    x_range: Option<String>,
    /// Variable letter. Default `x`.
    #[serde(default)]
    variable: Option<String>,
    /// Write the coefficient as `4x` instead of `4 · x`. Default false.
    #[serde(default)]
    implicit: Option<bool>,
    /// Vary the equation form (e.g. `b + a·x = c`). Default true.
    #[serde(default)]
    mix_forms: Option<bool>,
}

impl AlgebraTwoStepSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let a_range = match self.a_range {
            Some(s) => parse_digit_range(&s, "a_range")?,
            None => DigitRange::new(2, 10),
        };
        let b_range = match self.b_range {
            Some(s) => parse_digit_range(&s, "b_range")?,
            None => DigitRange::new(1, 30),
        };
        let x_range = match self.x_range {
            Some(s) => parse_digit_range(&s, "x_range")?,
            None => DigitRange::new(0, 20),
        };
        Ok(shared.fold(
            WorksheetType::AlgebraTwoStep {
                a_range,
                b_range,
                x_range,
                variable: self.variable.unwrap_or_else(|| "x".into()),
                implicit: self.implicit.unwrap_or(false),
                mix_forms: self.mix_forms.unwrap_or(true),
            },
            8,
            2,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AlgebraOneStepSpecific {
    /// Coefficient range. Default `2-10`.
    #[serde(default)]
    #[param(value_type = String, example = "2-10")]
    a_range: Option<String>,
    /// Constant range. Default `1-30`.
    #[serde(default)]
    #[param(value_type = String, example = "1-30")]
    b_range: Option<String>,
    /// Answer (`x`) range. Default `0-20`.
    #[serde(default)]
    #[param(value_type = String, example = "0-20")]
    x_range: Option<String>,
    /// Variable letter. Default `x`.
    #[serde(default)]
    variable: Option<String>,
    /// Include `x + b = c` problems. Default false.
    #[serde(default)]
    add: Option<bool>,
    /// Include `x − b = c` problems. Default false.
    #[serde(default)]
    subtract: Option<bool>,
    /// Include `a · x = c` problems. Default false.
    #[serde(default)]
    multiply: Option<bool>,
    /// Include `x ÷ a = c` problems. Default false.
    #[serde(default)]
    divide: Option<bool>,
}

impl AlgebraOneStepSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let a_range = match self.a_range {
            Some(s) => parse_digit_range(&s, "a_range")?,
            None => DigitRange::new(2, 10),
        };
        let b_range = match self.b_range {
            Some(s) => parse_digit_range(&s, "b_range")?,
            None => DigitRange::new(1, 30),
        };
        let x_range = match self.x_range {
            Some(s) => parse_digit_range(&s, "x_range")?,
            None => DigitRange::new(0, 20),
        };
        Ok(shared.fold(
            WorksheetType::AlgebraOneStep {
                a_range,
                b_range,
                x_range,
                variable: self.variable.unwrap_or_else(|| "x".into()),
                // Absence == off. With no toggle in the URL the existing
                // "at least one op enabled" validation rejects the request
                // with a readable error rather than silently defaulting.
                add: self.add.unwrap_or(false),
                subtract: self.subtract.unwrap_or(false),
                multiply: self.multiply.unwrap_or(false),
                divide: self.divide.unwrap_or(false),
            },
            10,
            2,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DecimalAddSpecific {
    /// Integer-part digit count (range) per operand, comma-separated.
    /// Default `2,2`.
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
    /// Decimal places on every operand and the answer, 1-4. Default 2.
    #[serde(default)]
    decimal_places: Option<u32>,
}

impl DecimalAddSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::DecimalAdd {
                digits: digits_or(self.digits, &[2, 2])?,
                decimal_places: self.decimal_places.unwrap_or(2),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DecimalSubtractSpecific {
    /// Integer-part digit count (range) per operand, comma-separated.
    /// Default `2,2`.
    #[serde(default)]
    #[param(value_type = String, example = "2,2")]
    digits: Option<String>,
    /// Decimal places on every operand and the answer, 1-4. Default 2.
    #[serde(default)]
    decimal_places: Option<u32>,
}

impl DecimalSubtractSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::DecimalSubtract {
                digits: digits_or(self.digits, &[2, 2])?,
                decimal_places: self.decimal_places.unwrap_or(2),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DecimalMultiplySpecific {
    /// Integer-part digit count (range) of the top operand. Default `2`.
    #[serde(default)]
    #[param(value_type = String, example = "2")]
    digits: Option<String>,
    /// Decimal places on the top operand, 1-4. Default 1.
    #[serde(default)]
    decimal_places: Option<u32>,
    /// Smallest multiplier integer part, 0-99. Default 2.
    #[serde(default)]
    multiplier_min: Option<u32>,
    /// Largest multiplier integer part, 0-99. Default 9.
    #[serde(default)]
    multiplier_max: Option<u32>,
    /// Decimal places on the multiplier; 0 = whole number. Default 0.
    #[serde(default)]
    bottom_decimal_places: Option<u32>,
}

impl DecimalMultiplySpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let digits = match self.digits {
            Some(s) => parse_digit_range(&s, "digits")?,
            None => DigitRange::fixed(2),
        };
        Ok(shared.fold(
            WorksheetType::DecimalMultiply {
                digits,
                decimal_places: self.decimal_places.unwrap_or(1),
                multiplier_min: self.multiplier_min.unwrap_or(2),
                multiplier_max: self.multiplier_max.unwrap_or(9),
                bottom_decimal_places: self.bottom_decimal_places.unwrap_or(0),
            },
            12,
            4,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct OrderOfOpsSpecific {
    /// Operators per problem, 2 or 3. Default 2.
    #[serde(default)]
    operations: Option<u32>,
    /// Wrap the additive part in parentheses so it evaluates first
    /// (e.g. `(a + b) × c`). Default false.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    parens: Option<bool>,
}

impl OrderOfOpsSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::OrderOfOperations {
                operations: self.operations.unwrap_or(2),
                use_parens: self.parens.unwrap_or(false),
            },
            12,
            2,
        ))
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct MeanSpecific {
    /// Values per data set (range). A range varies the count, and so
    /// the divisor, per problem. Default `3-4`.
    #[serde(default)]
    #[param(value_type = String, example = "3-4")]
    count: Option<String>,
    /// Digit count (range) of each value: `1-2` gives 1-99, `3` gives
    /// 100-999. Default `1-2`.
    #[serde(default)]
    #[param(value_type = String, example = "1-2")]
    digits: Option<String>,
    /// Print a pre-filled column-addition stack under the data. Changes
    /// the scaffolding, not the layout. Default false.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    scaffold: Option<bool>,
    /// Give every value one decimal place (e.g. `12.5`). Decimal or wide
    /// sets get an open work area instead of the addition stack. Default
    /// false.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    decimals: Option<bool>,
}

impl MeanSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let count = match self.count {
            Some(s) => parse_digit_range(&s, "count")?,
            None => DigitRange::new(3, 4),
        };
        let digits = match self.digits {
            Some(s) => parse_digit_range(&s, "digits")?,
            None => DigitRange::new(1, 2),
        };
        let style = shared.division_style();
        Ok(shared.fold(
            WorksheetType::Mean {
                count,
                digits,
                scaffold: self.scaffold.unwrap_or(false),
                decimals: self.decimals.unwrap_or(false),
                style,
            },
            // Each problem is a full column addition plus a long
            // division, so a page holds exactly four.
            4,
            2,
        ))
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(
    state: &AppState,
    endpoint: &'static str,
    built: Result<(OutputFormat, WorksheetParams)>,
    headers: &HeaderMap,
) -> Response {
    let ip = client_ip(headers);
    let ua = user_agent(headers);

    let (format, mut params) = match built {
        Ok(p) => p,
        Err(e) => {
            warn!(
                endpoint,
                region = %region_display(&state.region),
                ip = %ip,
                ua = %ua,
                error = %e,
                "worksheet request rejected"
            );
            return (StatusCode::BAD_REQUEST, format!("{e:#}\n")).into_response();
        }
    };
    if state.force_debug {
        params.debug = true;
    }

    let start = Instant::now();
    let result = generate(&params, format, &state.root, &state.fonts);
    let typst_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(ws) => {
            info!(
                kind = params.kind_slug(),
                format = format_slug(format),
                region = %region_display(&state.region),
                num_problems = params.num_problems,
                cols = params.cols,
                paper = %params.paper,
                seed = params.seed,
                solve_first = params.solve_first,
                include_answers = params.include_answers,
                bytes = ws.bytes.len(),
                typst_ms,
                ip = %ip,
                ua = %ua,
                "worksheet rendered"
            );

            let (ct, ext) = match format {
                OutputFormat::Pdf => ("application/pdf", "pdf"),
                OutputFormat::Png => ("image/png", "png"),
                OutputFormat::Svg => ("image/svg+xml", "svg"),
            };
            // `inline` (not `attachment`) so browsers show a preview; the
            // filename hint is still used when the user chooses Save As.
            let filename = format!("pencil-ready-{}.{ext}", params.slug());
            let disposition = format!("inline; filename=\"{filename}\"");
            (
                [
                    (header::CONTENT_TYPE, ct.to_string()),
                    (header::CONTENT_DISPOSITION, disposition),
                ],
                ws.bytes,
            )
                .into_response()
        }
        Err(e) => {
            warn!(
                kind = params.kind_slug(),
                format = format_slug(format),
                region = %region_display(&state.region),
                typst_ms,
                ip = %ip,
                ua = %ua,
                error = %e,
                "worksheet generation failed"
            );
            (StatusCode::BAD_REQUEST, format!("{e:#}\n")).into_response()
        }
    }
}

/// Resolve the real client IP. Behind Fly's edge proxy, `Fly-Client-IP`
/// is the canonical source; `X-Forwarded-For` is a fallback for generic
/// reverse proxies; otherwise we don't know.
fn client_ip(headers: &HeaderMap) -> String {
    if let Some(v) = headers.get("fly-client-ip").and_then(|v| v.to_str().ok()) {
        return v.to_string();
    }
    if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = xff.split(',').next() {
            return first.trim().to_string();
        }
    }
    "-".to_string()
}

fn user_agent(headers: &HeaderMap) -> String {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string()
}

fn format_slug(f: OutputFormat) -> &'static str {
    match f {
        OutputFormat::Pdf => "pdf",
        OutputFormat::Png => "png",
        OutputFormat::Svg => "svg",
    }
}

fn region_display(region: &Option<String>) -> &str {
    region.as_deref().unwrap_or("local")
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Addition
///
/// Multi-digit addition in columns, with control over carrying. `binary=true` renders in base 2.
#[utoipa::path(
    get,
    path = "/api/worksheets/add",
    operation_id = "add",
    params(SharedParams, AddSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_add(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<AddSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "add", p.build(shared), &headers)
}

/// Subtraction
///
/// Multi-digit subtraction in columns, with control over borrowing.
#[utoipa::path(
    get,
    path = "/api/worksheets/subtract",
    operation_id = "subtract",
    params(SharedParams, SubtractSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_subtract(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<SubtractSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "subtract", p.build(shared), &headers)
}

/// Multiplication
///
/// Multi-digit long multiplication in columns.
#[utoipa::path(
    get,
    path = "/api/worksheets/multiply",
    operation_id = "multiply",
    params(SharedParams, MultiplySpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_multiply(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<MultiplySpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "multiply", p.build(shared), &headers)
}

/// Simple division
///
/// Single-digit divisor, whole-number quotient. Times-table recall in reverse.
#[utoipa::path(
    get,
    path = "/api/worksheets/simple-divide",
    operation_id = "simple_divide",
    params(SharedParams, SimpleDivideSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_simple_divide(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<SimpleDivideSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "simple-divide", p.build(shared), &headers)
}

/// Long division
///
/// Single-digit divisor, multi-digit dividend, optionally with remainders.
#[utoipa::path(
    get,
    path = "/api/worksheets/long-divide",
    operation_id = "long_divide",
    params(SharedParams, LongDivideSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_long_divide(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<LongDivideSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "long-divide", p.build(shared), &headers)
}

/// Multiplication drill
///
/// One-line times-table facts from the chosen tables.
#[utoipa::path(
    get,
    path = "/api/worksheets/mult-drill",
    operation_id = "mult_drill",
    params(SharedParams, MultDrillSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_mult_drill(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<MultDrillSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "mult-drill", p.build(shared), &headers)
}

/// Division drill
///
/// One-line division facts built from the chosen divisors.
#[utoipa::path(
    get,
    path = "/api/worksheets/div-drill",
    operation_id = "div_drill",
    params(SharedParams, DivDrillSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_div_drill(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<DivDrillSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "div-drill", p.build(shared), &headers)
}

/// Fraction multiplication
///
/// Whole number times a proper fraction, with a whole-number answer.
#[utoipa::path(
    get,
    path = "/api/worksheets/fraction-mult",
    operation_id = "fraction_mult",
    params(SharedParams, FractionMultSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_fraction_mult(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FractionMultSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "fraction-mult", p.build(shared), &headers)
}

/// Fraction simplification
///
/// Reduce a fraction to lowest terms (or to a mixed/whole number).
#[utoipa::path(
    get,
    path = "/api/worksheets/fraction-simplify",
    operation_id = "fraction_simplify",
    params(SharedParams, FractionSimplifySpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_fraction_simplify(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FractionSimplifySpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "fraction-simplify", p.build(shared), &headers)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FractionAddSpecific {
    /// Allowed denominators of the two fractions, comma-separated.
    /// Default `2,3,4,5,6,8,10,12`.
    #[serde(default)]
    #[param(value_type = String, example = "2,3,4,5,6,8,10,12")]
    denominators: Option<String>,
    /// How the two denominators relate, which sets the difficulty.
    /// Default `any`.
    #[serde(default)]
    relation: Option<DenominatorRelation>,
}

impl FractionAddSpecific {
    fn build(self, shared: SharedParams, subtract: bool) -> Result<(OutputFormat, WorksheetParams)> {
        let denominators = match self.denominators {
            Some(s) => parse_u32_csv(&s, "denominators")?,
            None => vec![2, 3, 4, 5, 6, 8, 10, 12],
        };
        let relation = self.relation.unwrap_or_default();
        // A full page: 3 columns of 5 rows, or 7 when every problem
        // shares its denominator (shorter cells, no rewrite step).
        let problems = if relation == DenominatorRelation::Same { 21 } else { 15 };
        let worksheet = if subtract {
            WorksheetType::FractionSubtract { denominators, relation }
        } else {
            WorksheetType::FractionAdd { denominators, relation }
        };
        Ok(shared.fold(worksheet, problems, 3))
    }
}

/// Fraction addition
///
/// Add two proper fractions; answers in simplest form. `relation` sets the difficulty: same denominator, one a multiple of the other, or unrelated.
#[utoipa::path(
    get,
    path = "/api/worksheets/fraction-add",
    operation_id = "fraction_add",
    params(SharedParams, FractionAddSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_fraction_add(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FractionAddSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "fraction-add", p.build(shared, false), &headers)
}

/// Fraction subtraction
///
/// Subtract two proper fractions (the first is always larger); answers in simplest form. Same parameters as fraction addition.
#[utoipa::path(
    get,
    path = "/api/worksheets/fraction-subtract",
    operation_id = "fraction_subtract",
    params(SharedParams, FractionAddSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_fraction_subtract(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FractionAddSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "fraction-subtract", p.build(shared, true), &headers)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FractionDecimalPercentSpecific {
    /// Allowed denominators, comma-separated; each must divide 1000
    /// (2, 4, 5, 8, 10, 20, 25, 40, 50, 100). Default `2,4,5,10,20,25,50`.
    #[serde(default)]
    #[param(value_type = String, example = "2,4,5,10,20,25,50")]
    denominators: Option<String>,
    /// Include values above 1 (below 3), as mixed numbers. Default false.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    improper: Option<bool>,
    /// Which form is printed. Default `any`.
    #[serde(default)]
    given: Option<ConversionGiven>,
}

impl FractionDecimalPercentSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let denominators = match self.denominators {
            Some(s) => parse_u32_csv(&s, "denominators")?,
            None => vec![2, 4, 5, 10, 20, 25, 50],
        };
        Ok(shared.fold(
            WorksheetType::FractionDecimalPercent {
                denominators,
                improper: self.improper.unwrap_or(false),
                given: self.given.unwrap_or_default(),
            },
            // 2 columns of 5: the rows get extra space for working out
            // the conversions (the page would hold 9).
            10,
            2,
        ))
    }
}

/// Fractions, decimals and percents
///
/// One number written three ways (`3/4 = 0.75 = 75%`): one form is printed, the student writes the other two.
#[utoipa::path(
    get,
    path = "/api/worksheets/fraction-decimal-percent",
    operation_id = "fraction_decimal_percent",
    params(SharedParams, FractionDecimalPercentSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_fraction_decimal_percent(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FractionDecimalPercentSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "fraction-decimal-percent", p.build(shared), &headers)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct PercentOfSpecific {
    /// Percents to use (1-100): values or ranges, comma-separated
    /// (e.g. `10,50` or `1-99`). Default `10,20,25,50,75`.
    #[serde(default)]
    #[param(value_type = String, example = "10,20,25,50,75")]
    percents: Option<String>,
    /// Largest base number, 10-1000. Default 400.
    #[serde(default)]
    max_base: Option<u32>,
    /// Allow answers with up to two decimal places (`37% of 48 =
    /// 17.76`), so any percent pairs with any base. Default false
    /// (whole-number answers only).
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    decimals: Option<bool>,
}

impl PercentOfSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let percents = match self.percents {
            Some(s) => parse_digits_csv(&s, "percents")?
                .into_iter()
                .flat_map(|r| r.min..=r.max)
                .collect(),
            None => vec![10, 20, 25, 50, 75],
        };
        Ok(shared.fold(
            WorksheetType::PercentOf {
                percents,
                max_base: self.max_base.unwrap_or(400),
                decimals: self.decimals.unwrap_or(false),
            },
            PERCENT_OF_PROBLEMS,
            2,
        ))
    }
}

/// Percent-of problems that fill one page: 2 columns of 3, each with
/// room under it for the working (a decimal column multiplication).
const PERCENT_OF_PROBLEMS: u32 = 6;

/// Percent of a number
///
/// `20% of 350 = ___`: whole-number answers, or up to two decimals with `decimals=true`.
#[utoipa::path(
    get,
    path = "/api/worksheets/percent-of",
    operation_id = "percent_of",
    params(SharedParams, PercentOfSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_percent_of(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<PercentOfSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "percent-of", p.build(shared), &headers)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct GcfSpecific {
    /// Largest number in a pair, 10-100. Default 30.
    #[serde(default)]
    max_number: Option<u32>,
    /// Mix in pairs whose GCF is 1 (a fraction already in simplest
    /// form). Default false.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    coprime: Option<bool>,
    /// A blank per factor (`6: __ __ __ __`) instead of one line per
    /// number, so the student knows how many factors to find. Default
    /// false.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    slots: Option<bool>,
}

impl GcfSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::Gcf {
                max_number: self.max_number.unwrap_or(30),
                coprime: self.coprime.unwrap_or(false),
                slots: self.slots.unwrap_or(false),
            },
            GCF_PROBLEMS,
            1,
        ))
    }
}

/// GCF problems that fill one page: one column of 5, each with lines
/// to list both numbers' factors.
const GCF_PROBLEMS: u32 = 5;

/// Greatest common factor
///
/// `GCF(12, 18) = ___`, with a line under it per number to list its
/// factors (`SFF` in Norwegian). Practice for simplifying fractions.
#[utoipa::path(
    get,
    path = "/api/worksheets/gcf",
    operation_id = "gcf",
    params(SharedParams, GcfSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_gcf(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<GcfSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "gcf", p.build(shared), &headers)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FactorPairsSpecific {
    /// Largest number, 10-100. Numbers come from its top two thirds
    /// (10-30 for 30). Default 30.
    #[serde(default)]
    max_number: Option<u32>,
    /// Include primes, which have one pair (`1 × p`). Default false.
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    primes: Option<bool>,
    /// Show only as many pair rows as each number has, as a hint for
    /// how many to find. Default false (every cell shows the same rows).
    #[serde(default, deserialize_with = "deserialize_loose_bool")]
    exact_rows: Option<bool>,
}

impl FactorPairsSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let max_number = self.max_number.unwrap_or(30);
        let primes = self.primes.unwrap_or(false);
        Ok(shared.fold(
            WorksheetType::FactorPairs {
                max_number,
                primes,
                exact_rows: self.exact_rows.unwrap_or(false),
            },
            // Four columns, as many rows as fit: the cell grows with the
            // most pairs a number in the range has.
            factor_pairs_default_problems(max_number, primes),
            4,
        ))
    }
}

/// Factor pairs
///
/// Every factor of a number, as factor pairs (`1 × 24`, `2 × 12`, …),
/// with a row per pair under the number.
#[utoipa::path(
    get,
    path = "/api/worksheets/factor-pairs",
    operation_id = "factor_pairs",
    params(SharedParams, FactorPairsSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_factor_pairs(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FactorPairsSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "factor-pairs", p.build(shared), &headers)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct PrimeFactorsSpecific {
    /// Largest number, 10-100. Composite numbers come from its top two
    /// thirds (10-30 for 30). Default 30.
    #[serde(default)]
    max_number: Option<u32>,
}

impl PrimeFactorsSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        Ok(shared.fold(
            WorksheetType::PrimeFactors {
                max_number: self.max_number.unwrap_or(30),
            },
            PRIME_FACTORS_PROBLEMS,
            2,
        ))
    }
}

/// Prime-factorization problems that fill one page: 2 columns of 3, each
/// with room for a factor tree.
const PRIME_FACTORS_PROBLEMS: u32 = 6;

/// Prime factorization
///
/// `24 = ___`: write the number as a product of primes, with room under
/// it for a factor tree.
#[utoipa::path(
    get,
    path = "/api/worksheets/prime-factors",
    operation_id = "prime_factors",
    params(SharedParams, PrimeFactorsSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_prime_factors(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<PrimeFactorsSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "prime-factors", p.build(shared), &headers)
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FractionEquivSpecific {
    /// Allowed denominators of the base fraction, comma-separated.
    /// Default `2,3,4,5,6,8,10`.
    #[serde(default)]
    #[param(value_type = String, example = "2,3,4,5,6,8,10")]
    denominators: Option<String>,
    /// Scale-factor range (the multiplier), within 2-10. Default `2-5`.
    #[serde(default)]
    #[param(value_type = String, example = "2-5")]
    scale: Option<String>,
    /// Which slot is blank. Default `any` (varies per problem).
    #[serde(default)]
    missing: Option<MissingSlot>,
    /// Base fraction is proper (numerator < denominator). Default true.
    #[serde(default)]
    proper_only: Option<bool>,
}

impl FractionEquivSpecific {
    fn build(self, shared: SharedParams) -> Result<(OutputFormat, WorksheetParams)> {
        let denominators = match self.denominators {
            Some(s) => parse_u32_csv(&s, "denominators")?,
            None => vec![2, 3, 4, 5, 6, 8, 10],
        };
        let scale = match self.scale {
            Some(s) => s.parse::<DigitRange>().map_err(|e| anyhow::anyhow!(e))?,
            None => DigitRange::new(2, 5),
        };
        Ok(shared.fold(
            WorksheetType::FractionEquiv {
                denominators,
                scale,
                missing: self.missing.unwrap_or(MissingSlot::Any),
                proper_only: self.proper_only.unwrap_or(true),
            },
            12,
            3,
        ))
    }
}

/// Equivalent fractions
///
/// Fill in the missing numerator or denominator of `a/b = c/d`.
#[utoipa::path(
    get,
    path = "/api/worksheets/fraction-equiv",
    operation_id = "fraction_equiv",
    params(SharedParams, FractionEquivSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_fraction_equiv(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<FractionEquivSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "fraction-equiv", p.build(shared), &headers)
}

/// Algebra: two-step equations
///
/// Solve `a·x + b = c` and related forms for a whole-number `x`.
#[utoipa::path(
    get,
    path = "/api/worksheets/algebra-two-step",
    operation_id = "algebra_two_step",
    params(SharedParams, AlgebraTwoStepSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_algebra_two_step(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<AlgebraTwoStepSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "algebra-two-step", p.build(shared), &headers)
}

/// Algebra: one-step equations
///
/// Solve `x + b = c`, `x − b = c`, `a·x = c` or `x ÷ a = c`. Enable at least one form.
#[utoipa::path(
    get,
    path = "/api/worksheets/algebra-one-step",
    operation_id = "algebra_one_step",
    params(SharedParams, AlgebraOneStepSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_algebra_one_step(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<AlgebraOneStepSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "algebra-one-step", p.build(shared), &headers)
}

/// Algebra: squares and square roots
///
/// Solve `x² ± b = c` and `√x ± b = c`. Enable at least one of `squares` / `roots`.
#[utoipa::path(
    get,
    path = "/api/worksheets/algebra-square-root",
    operation_id = "algebra_square_root",
    params(SharedParams, AlgebraSquareRootSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_algebra_square_root(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<AlgebraSquareRootSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "algebra-square-root", p.build(shared), &headers)
}

/// Decimal addition
///
/// Column addition with a fixed number of decimal places.
#[utoipa::path(
    get,
    path = "/api/worksheets/decimal-add",
    operation_id = "decimal_add",
    params(SharedParams, DecimalAddSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_decimal_add(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<DecimalAddSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "decimal-add", p.build(shared), &headers)
}

/// Decimal subtraction
///
/// Column subtraction with a fixed number of decimal places.
#[utoipa::path(
    get,
    path = "/api/worksheets/decimal-subtract",
    operation_id = "decimal_subtract",
    params(SharedParams, DecimalSubtractSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_decimal_subtract(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<DecimalSubtractSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "decimal-subtract", p.build(shared), &headers)
}

/// Decimal multiplication
///
/// A decimal times a whole number (or a second decimal).
#[utoipa::path(
    get,
    path = "/api/worksheets/decimal-multiply",
    operation_id = "decimal_multiply",
    params(SharedParams, DecimalMultiplySpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_decimal_multiply(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<DecimalMultiplySpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "decimal-multiply", p.build(shared), &headers)
}

/// Order of operations
///
/// Evaluate expressions with 2-3 operators, optionally with parentheses.
#[utoipa::path(
    get,
    path = "/api/worksheets/order-of-ops",
    operation_id = "order_of_ops",
    params(SharedParams, OrderOfOpsSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_order_of_ops(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<OrderOfOpsSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "order-of-ops", p.build(shared), &headers)
}

/// Mean (average)
///
/// Find the average of a small data set, optionally with a column-addition scaffold.
#[utoipa::path(
    get,
    path = "/api/worksheets/mean",
    operation_id = "mean",
    params(SharedParams, MeanSpecific),
    responses(
        (status = 200, response = WorksheetFile),
        (status = 400, response = InvalidParams),
    ),
    tag = "worksheets",
)]
async fn handle_mean(
    State(s): State<Arc<AppState>>,
    Query(shared): Query<SharedParams>,
    Query(p): Query<MeanSpecific>,
    headers: HeaderMap,
) -> Response {
    render(&s, "mean", p.build(shared), &headers)
}

// ---------------------------------------------------------------------------
// Umami analytics proxy
// ---------------------------------------------------------------------------

/// Serve the Umami tracker bundle from our own origin.
///
/// Fetches `script.js` from cloud.umami.is once and caches it in memory for
/// the lifetime of the process. The frontend points `data-host-url="/umami"`
/// so the tracker posts events to `/umami/api/send` (below) rather than
/// directly to umami.is.
async fn handle_umami_script(State(s): State<Arc<AppState>>) -> Response {
    let mut cache = s.umami_script.lock().await;
    if cache.is_none() {
        match s
            .umami_client
            .get("https://cloud.umami.is/script.js")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(b) => *cache = Some(Bytes::copy_from_slice(&b)),
                Err(e) => {
                    warn!("umami script body error: {e}");
                    return (StatusCode::BAD_GATEWAY, "failed to read umami script")
                        .into_response();
                }
            },
            Ok(resp) => {
                warn!("umami script upstream status: {}", resp.status());
                return (StatusCode::BAD_GATEWAY, "umami upstream error").into_response();
            }
            Err(e) => {
                warn!("umami script fetch error: {e}");
                return (StatusCode::BAD_GATEWAY, "failed to fetch umami script")
                    .into_response();
            }
        }
    }
    let bytes = cache.as_ref().unwrap().clone();
    (
        [
            (
                header::CONTENT_TYPE,
                "application/javascript; charset=utf-8",
            ),
            (header::CACHE_CONTROL, "public, max-age=86400"),
        ],
        bytes,
    )
        .into_response()
}

/// Forward Umami tracking payloads to cloud.umami.is, preserving the
/// client's User-Agent and IP so Umami can geo-locate and detect browsers.
///
/// The visitor IP travels as `payload.ip` in the JSON body, not just
/// `X-Forwarded-For`: Umami Cloud sits behind Cloudflare, which stamps
/// every request with geo headers (`cf-ipcity`, `cf-ipcountry`) derived
/// from the *connecting* IP — this machine, not the visitor — and Umami
/// consults those headers before any forwarded-IP header. `payload.ip`
/// is the only field that makes Umami skip them and geo-locate the
/// visitor. It also feeds Umami's visitor-identity hash, so distinct
/// visitors sharing a User-Agent don't collapse into one session.
async fn handle_umami_send(State(s): State<Arc<AppState>>, req: Request) -> Response {
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 64 * 1024).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "request body too large").into_response(),
    };

    let ip = client_ip(&parts.headers);
    let body_bytes = match inject_payload_ip(&body_bytes, &ip) {
        Some(b) => b,
        None => body_bytes.to_vec(),
    };

    let mut proxy = s
        .umami_client
        .post("https://cloud.umami.is/api/send")
        .body(body_bytes);

    if let Some(ct) = parts.headers.get(header::CONTENT_TYPE) {
        proxy = proxy.header("Content-Type", ct);
    }
    if let Some(ua) = parts.headers.get(header::USER_AGENT) {
        proxy = proxy.header("User-Agent", ua);
    }
    // Fallback for bodies `inject_payload_ip` couldn't rewrite; ignored
    // by Umami Cloud (Cloudflare's headers outrank it) but honored by
    // self-hosted instances.
    if ip != "-" {
        proxy = proxy.header("X-Forwarded-For", ip);
    }

    match proxy.send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.bytes().await.unwrap_or_default();
            (status, body).into_response()
        }
        Err(e) => {
            warn!("umami send proxy error: {e}");
            (StatusCode::BAD_GATEWAY, "proxy error").into_response()
        }
    }
}

/// Set `payload.ip` in an Umami event body to the visitor's IP,
/// overwriting any client-supplied value (a visitor must not be able to
/// spoof their location through our proxy). Returns `None` — forward
/// the body untouched — when the IP is unknown or the body isn't the
/// expected `{"payload": {...}}` shape; a mislocated event beats a
/// dropped one.
fn inject_payload_ip(body: &[u8], ip: &str) -> Option<Vec<u8>> {
    if ip == "-" {
        return None;
    }
    let mut value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let payload = value.get_mut("payload")?.as_object_mut()?;
    payload.insert("ip".into(), serde_json::Value::String(ip.to_string()));
    serde_json::to_vec(&value).ok()
}

/// Text substitution middleware: replaces `SERVER_REGION_PLACEHOLDER`
/// inside `text/html` response bodies with a `(server:<region>)` tag (or
/// an empty string for local dev). Layered *inside* compression so the
/// body is still plaintext when we rewrite it.
async fn region_rewrite(
    State(s): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let response = next.run(req).await;

    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("text/html"));
    if !is_html {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    let replacement = match s.region.as_deref() {
        Some(r) => format!("(server:{r})"),
        None => String::new(),
    };

    // Bail fast if the marker isn't present — most non-page HTML won't
    // have it, and we save an allocation.
    let haystack = match std::str::from_utf8(&bytes) {
        Ok(s) if s.contains("SERVER_REGION_PLACEHOLDER") => s,
        _ => return Response::from_parts(parts, Body::from(bytes)),
    };
    let rewritten = haystack.replace("SERVER_REGION_PLACEHOLDER", &replacement);
    // Content-Length no longer matches; drop it so the transport layer
    // re-computes or switches to chunked.
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(rewritten))
}

/// 301 `www.<host>` to the bare host so search engines see one
/// canonical origin instead of two duplicate sites. Hostnames without
/// the `www.` prefix (apex, `*.fly.dev`, localhost) pass through.
async fn www_redirect(req: Request, next: Next) -> Response {
    let host = req.headers().get(header::HOST).and_then(|v| v.to_str().ok());
    let path_and_query = req.uri().path_and_query().map_or("/", |pq| pq.as_str());
    match host.and_then(|h| www_redirect_target(h, path_and_query)) {
        Some(location) => (
            StatusCode::MOVED_PERMANENTLY,
            [(header::LOCATION, location)],
        )
            .into_response(),
        None => next.run(req).await,
    }
}

fn www_redirect_target(host: &str, path_and_query: &str) -> Option<String> {
    let host = host.to_ascii_lowercase();
    let apex = host.strip_prefix("www.")?;
    Some(format!("https://{apex}{path_and_query}"))
}

/// Non-default site locales, by URL path prefix and `lang` cookie value
/// — the ids in `LOCALES` (frontend/astro/src/i18n/locales.ts) other than
/// the root locale. Keep in step with that list.
const LOCALE_PREFIXES: &[&str] = &["no"];

/// Send a visitor *entering* the site at `/` to the locale they chose in
/// the language selector. Deliberately narrow (see SPEC.md → Language):
///
/// - only `GET`/`HEAD` of exactly `/` — every other URL serves what was
///   asked for, so links and bookmarks into either locale are stable;
/// - only with a `lang=<locale>` cookie, which the selector sets on an
///   explicit click (never from `Accept-Language`, so crawlers — which
///   send no cookie — always get the English root);
/// - only on entry: `Sec-Fetch-Site` absent, `none` (typed URL,
///   bookmark) or `cross-site` (external link). A `same-origin` /
///   `same-site` request — the logo on an English page, a ClientRouter
///   fetch, a prefetch — is never redirected, so a reader who is on an
///   English page stays there.
///
/// Caching for `/` (private, varies on cookie + fetch site) is set by
/// `cache_control`, which wraps this layer and so sees both the 302 and
/// the 200.
async fn locale_root_redirect(req: Request, next: Next) -> Response {
    let headers = req.headers();
    let cookie = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");
    let fetch_site = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok());
    match locale_root_redirect_target(
        req.method(),
        req.uri().path(),
        req.uri().query(),
        &cookie,
        fetch_site,
    ) {
        Some(location) => (StatusCode::FOUND, [(header::LOCATION, location)]).into_response(),
        None => next.run(req).await,
    }
}

fn locale_root_redirect_target(
    method: &Method,
    path: &str,
    query: Option<&str>,
    cookie: &str,
    sec_fetch_site: Option<&str>,
) -> Option<String> {
    if !(method == Method::GET || method == Method::HEAD) || path != "/" {
        return None;
    }
    if !matches!(sec_fetch_site, None | Some("none") | Some("cross-site")) {
        return None;
    }
    // Hand-parsed: `name=value` pairs split on `;`. The first `lang`
    // wins; an unknown value (including the root locale's `en`) is
    // ignored.
    let lang = cookie
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == "lang")
        .map(|(_, value)| value.trim())?;
    let prefix = LOCALE_PREFIXES.iter().find(|p| **p == lang)?;
    Some(match query {
        Some(q) if !q.is_empty() => format!("/{prefix}/?{q}"),
        _ => format!("/{prefix}/"),
    })
}

/// Cache-Control by URI prefix. Astro hashes its bundle filenames
/// (`/_astro/*.js`, `/_astro/*.css`) so they're safe to mark immutable;
/// fonts ship with stable filenames but never change in practice
/// (we'd version-bump the filename if they did) so they get the same
/// year-long lifetime. Other static assets (svg/png/ico) get a month.
/// HTML is held back to 5 minutes so region-rewrite tags and routine
/// content updates propagate without forcing a hard reload.
///
/// The root `/` is the exception: `locale_root_redirect` answers it per
/// visitor (302 to `/no/` or the English page, by cookie and
/// `Sec-Fetch-Site`), so it's `private, no-cache` and varies on both
/// inputs. A shared 5-minute cache there would keep serving English for
/// minutes after a visitor switched to Norwegian. Both the redirect and
/// the page pass through here, so this is the one place that sets it.
///
/// Anything else (including `/api/*` worksheet generation and the
/// Umami proxy) is left untouched — handlers set their own caching.
async fn cache_control(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let mut response = next.run(req).await;

    if path == "/" {
        let headers = response.headers_mut();
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-cache"),
        );
        headers.append(
            header::VARY,
            HeaderValue::from_static("Cookie, Sec-Fetch-Site"),
        );
        return response;
    }

    if response.headers().contains_key(header::CACHE_CONTROL) {
        return response;
    }

    let value = if path.starts_with("/_astro/") {
        Some("public, max-age=31536000, immutable")
    } else if path.ends_with(".svg") || path.ends_with(".png") || path.ends_with(".ico") {
        Some("public, max-age=2592000")
    } else if path.ends_with(".html") || path.ends_with('/') {
        Some("public, max-age=300, must-revalidate")
    } else {
        None
    };

    if let Some(v) = value {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(v));
    }
    response
}

// ---------------------------------------------------------------------------
// OpenAPI + entry
// ---------------------------------------------------------------------------

/// Raw file bytes.
// Spec-only: handlers return the bytes directly, never this type.
#[derive(utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
#[allow(dead_code)]
struct WorksheetBytes(Vec<u8>);

/// The rendered worksheet. Media type follows `format`.
/// `Content-Disposition` is `inline` with a suggested filename.
#[derive(utoipa::ToResponse)]
#[response(headers(
    ("Content-Disposition" = String, description = "inline; filename=\"pencil-ready-<slug>.<ext>\""),
))]
#[allow(dead_code)]
enum WorksheetFile {
    Pdf(#[content("application/pdf")] WorksheetBytes),
    Png(#[content("image/png")] WorksheetBytes),
    Svg(#[content("image/svg+xml")] WorksheetBytes),
}

/// Parameters failed to parse or validate, or the combination can't
/// produce a worksheet. Body is a one-line, human-readable error.
#[derive(utoipa::ToResponse)]
#[response(content_type = "text/plain", example = json!("digits: bad digit count: invalid digit found in string\n"))]
#[allow(dead_code)]
struct InvalidParams(String);

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Pencil Ready",
        description = include_str!("openapi-description.md"),
        license(name = "MIT", identifier = "MIT"),
    ),
    servers(
        (url = "/", description = "This server"),
        (url = "https://pencilready.com", description = "Production"),
    ),
    tags((name = "worksheets", description = "Worksheet generation endpoints")),
    components(
        schemas(
            CarryMode, BorrowMode, Locale, DivisionStyle, OutputFormat, Paper, MissingSlot,
            DenominatorRelation, ConversionGiven,
            WorksheetBytes,
        ),
        responses(WorksheetFile, InvalidParams),
    ),
)]
struct ApiDoc;

#[derive(Parser)]
#[command(name = "pencil-ready-server", about = "Pencil Ready worksheet server")]
struct Cli {
    /// Listen port.
    #[arg(long, default_value_t = 8080, env = "PORT")]
    port: u16,
    /// Project root for typst imports (lib/, fonts/, assets/).
    #[arg(long, default_value = ".", env = "PENCIL_READY_ROOT")]
    root: PathBuf,
    /// Static-content directory. If omitted, defaults to
    /// `<root>/frontend/astro/dist`; the server runs API-only if that
    /// path doesn't exist and `--static-dir` isn't overridden.
    #[arg(long, env = "PENCIL_READY_STATIC_DIR")]
    static_dir: Option<PathBuf>,
    /// Run API-only, without serving any static bundle.
    #[arg(long)]
    api_only: bool,
    /// Force `debug: true` on every worksheet render — useful for visual
    /// layout debugging via the browser. Off by default; never set in
    /// production.
    #[arg(long, env = "PENCIL_READY_DEBUG")]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    init_tracing();

    let root = cli
        .root
        .canonicalize()
        .expect("canonicalize project root (set --root or cd to the repo)");
    // FLY_REGION is set on every Fly machine; absent locally.
    let region = std::env::var("FLY_REGION").ok().filter(|s| !s.is_empty());
    // Parse all bundled fonts once; clone the Arcs per request.
    let fonts = Fonts::load(&root).expect("load fonts from <root>/fonts");
    info!(region = %region_display(&region), "pencil-ready-server starting");
    if cli.debug {
        info!("--debug active: forcing debug borders on every worksheet render");
    }
    let state = Arc::new(AppState {
        root: root.clone(),
        region,
        fonts,
        force_debug: cli.debug,
        umami_client: reqwest::Client::new(),
        umami_script: Arc::new(tokio::sync::Mutex::new(None)),
    });

    // Resolve the static directory. Explicit flag wins; otherwise
    // default to the Astro build alongside the repo. --api-only bypasses
    // static serving entirely.
    let static_dir = if cli.api_only {
        None
    } else {
        cli.static_dir
            .or_else(|| Some(root.join("frontend/astro/dist")))
            .filter(|p| p.join("index.html").is_file())
    };

    let compression = CompressionLayer::new().gzip(true).br(true);

    // Permissive CORS. Routes are all GET-only reads with no credentials,
    // so the blast radius is the generated PDFs/PNGs/SVGs anyway. Enables
    // the Astro dev server on :4321 (or any other frontend on a different
    // origin) to fetch() worksheet bytes cleanly.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (api_router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(handle_add))
        .routes(routes!(handle_subtract))
        .routes(routes!(handle_multiply))
        .routes(routes!(handle_simple_divide))
        .routes(routes!(handle_long_divide))
        .routes(routes!(handle_mult_drill))
        .routes(routes!(handle_div_drill))
        .routes(routes!(handle_fraction_mult))
        .routes(routes!(handle_fraction_simplify))
        .routes(routes!(handle_fraction_equiv))
        .routes(routes!(handle_fraction_add))
        .routes(routes!(handle_fraction_subtract))
        .routes(routes!(handle_fraction_decimal_percent))
        .routes(routes!(handle_percent_of))
        .routes(routes!(handle_gcf))
        .routes(routes!(handle_factor_pairs))
        .routes(routes!(handle_prime_factors))
        .routes(routes!(handle_algebra_two_step))
        .routes(routes!(handle_algebra_one_step))
        .routes(routes!(handle_algebra_square_root))
        .routes(routes!(handle_decimal_add))
        .routes(routes!(handle_decimal_subtract))
        .routes(routes!(handle_decimal_multiply))
        .routes(routes!(handle_order_of_ops))
        .routes(routes!(handle_mean))
        .with_state(state.clone())
        .split_for_parts();

    // SwaggerUi::url("/openapi.json", api) both serves the spec at that
    // path and points the UI at it — no separate route needed.
    let mut app = axum::Router::new()
        .merge(SwaggerUi::new("/docs").url("/openapi.json", api))
        .merge(api_router)
        .route(
            "/umami/script.js",
            axum::routing::get(handle_umami_script),
        )
        .route(
            "/umami/api/send",
            axum::routing::post(handle_umami_send),
        )
        .with_state(state.clone());

    // One frontend per process. Astro's static build emits a real file
    // per route (`/worksheets/add/index.html`, etc.), so ServeDir
    // resolves deep links directly — no SPA-style rewrite-to-index
    // fallback needed. For anything Astro didn't generate (typos, scan
    // traffic hitting `/wp-login.php`, etc.) we return the pre-rendered
    // `404.html` with a real 404 status.
    app = match &static_dir {
        Some(dir) => {
            println!("serving frontend from {}", dir.display());
            let not_found_html = std::fs::read_to_string(dir.join("404.html")).ok();
            let serve_dir = match not_found_html {
                Some(html) => {
                    let html: Arc<str> = Arc::from(html);
                    ServeDir::new(dir).fallback(axum::routing::any(move || {
                        let html = html.clone();
                        async move {
                            (
                                StatusCode::NOT_FOUND,
                                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                                html.as_ref().to_owned(),
                            )
                        }
                    }))
                }
                None => ServeDir::new(dir).fallback(axum::routing::any(|| async {
                    (StatusCode::NOT_FOUND, "not found\n")
                })),
            };
            app.fallback_service(serve_dir)
        }
        None => {
            // No SPA bundle available — expose a cheap root for liveness.
            app.route("/", axum::routing::get(|| async { "hello world\n" }))
        }
    };

    // Layer order (innermost → outermost):
    //   1. region_rewrite — mutates text/html bodies before compression
    //      sees them. Must be innermost so we operate on plaintext.
    //   2. locale_root_redirect — 302s an entering `/` request to the
    //      visitor's chosen locale. Outside ServeDir/region_rewrite (no
    //      need to render the page it replaces), inside www_redirect
    //      (the apex hop happens first).
    //   3. cache_control — wraps the redirect so `/`'s private,
    //      cookie-varying policy covers the 302 and the 200 alike.
    //   4. compression — gzips/brs the (possibly rewritten) body.
    //   5. www_redirect — short-circuits `www.` hosts before any of the
    //      above run.
    //   6. cors — adds headers, including to www redirects so
    //      cross-origin fetches can follow them.
    //   7. TraceLayer — request spans wrap everything above. 2xx/3xx/5xx
    //      log at INFO; 4xx (mostly scan traffic hitting our 404 page)
    //      drops to DEBUG so it doesn't flood the access log.
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|req: &Request<Body>| {
            info_span!(
                "http",
                method = %req.method(),
                uri = %req.uri(),
                ip = %client_ip(req.headers()),
                ua = %user_agent(req.headers()),
            )
        })
        .on_response(
            |response: &Response<Body>, latency: Duration, _span: &tracing::Span| {
                let status = response.status();
                if status.is_client_error() {
                    tracing::debug!(
                        status = status.as_u16(),
                        latency_ms = latency.as_millis() as u64,
                        "request"
                    );
                } else {
                    tracing::info!(
                        status = status.as_u16(),
                        latency_ms = latency.as_millis() as u64,
                        "request"
                    );
                }
            },
        );

    let app = app
        .layer(middleware::from_fn_with_state(state, region_rewrite))
        .layer(middleware::from_fn(locale_root_redirect))
        .layer(middleware::from_fn(cache_control))
        .layer(compression)
        .layer(middleware::from_fn(www_redirect))
        .layer(cors)
        .layer(trace_layer);

    let addr = format!("0.0.0.0:{}", cli.port);
    info!(port = cli.port, "pencil-ready-server listening");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Pretty output to a terminal, JSON when stderr is piped (Fly, systemd,
/// containers). Override with `LOG_FORMAT=json|pretty`. Filter via
/// `RUST_LOG=…` (defaults to info for this crate + warn for everything else).
fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("warn,pencil_ready_server=info,tower_http=info")
    });

    let force = std::env::var("LOG_FORMAT").ok();
    let json = match force.as_deref() {
        Some("json") => true,
        Some("pretty") => false,
        _ => !std::io::stderr().is_terminal(),
    };

    if json {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json().flatten_event(true))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().with_target(false).compact())
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::{Method, inject_payload_ip, locale_root_redirect_target, www_redirect_target};

    mod paper_fit {
        use super::super::*;
        use pencil_ready_core::Document;

        const KINDS: &[&str] = &[
            "add",
            "subtract",
            "multiply",
            "simple-divide",
            "long-divide",
            "mult-drill",
            "div-drill",
            "fraction-mult",
            "fraction-simplify",
            "fraction-equiv",
            "fraction-add",
            "fraction-subtract",
            "fraction-decimal-percent",
            "percent-of",
            "gcf",
            "factor-pairs",
            "prime-factors",
            "algebra-two-step",
            "algebra-one-step",
            "algebra-square-root",
            "decimal-add",
            "decimal-subtract",
            "decimal-multiply",
            "order-of-ops",
            "mean",
        ];
        const DIVISION_KINDS: &[&str] = &["long-divide", "simple-divide", "mean"];

        /// Parse `qs` the way the handler for `kind` does.
        fn params_for(kind: &str, qs: &str) -> WorksheetParams {
            let uri: axum::http::Uri = format!("/x?{qs}").parse().unwrap();
            let shared = Query::<SharedParams>::try_from_uri(&uri).unwrap().0;
            macro_rules! build {
                ($t:ty) => {
                    Query::<$t>::try_from_uri(&uri)
                        .unwrap()
                        .0
                        .build(shared)
                        .unwrap_or_else(|e| panic!("{kind}?{qs}: {e}"))
                        .1
                };
            }
            match kind {
                "add" => build!(AddSpecific),
                "subtract" => build!(SubtractSpecific),
                "multiply" => build!(MultiplySpecific),
                "simple-divide" => build!(SimpleDivideSpecific),
                "long-divide" => build!(LongDivideSpecific),
                "mult-drill" => build!(MultDrillSpecific),
                "div-drill" => build!(DivDrillSpecific),
                "fraction-mult" => build!(FractionMultSpecific),
                "fraction-simplify" => build!(FractionSimplifySpecific),
                "fraction-equiv" => build!(FractionEquivSpecific),
                "fraction-decimal-percent" => build!(FractionDecimalPercentSpecific),
                "percent-of" => build!(PercentOfSpecific),
                "gcf" => build!(GcfSpecific),
                "factor-pairs" => build!(FactorPairsSpecific),
                "prime-factors" => build!(PrimeFactorsSpecific),
                "fraction-add" => Query::<FractionAddSpecific>::try_from_uri(&uri)
                    .unwrap()
                    .0
                    .build(shared, false)
                    .unwrap_or_else(|e| panic!("{kind}?{qs}: {e}"))
                    .1,
                "fraction-subtract" => Query::<FractionAddSpecific>::try_from_uri(&uri)
                    .unwrap()
                    .0
                    .build(shared, true)
                    .unwrap_or_else(|e| panic!("{kind}?{qs}: {e}"))
                    .1,
                "algebra-two-step" => build!(AlgebraTwoStepSpecific),
                "algebra-one-step" => build!(AlgebraOneStepSpecific),
                "algebra-square-root" => build!(AlgebraSquareRootSpecific),
                "decimal-add" => build!(DecimalAddSpecific),
                "decimal-subtract" => build!(DecimalSubtractSpecific),
                "decimal-multiply" => build!(DecimalMultiplySpecific),
                "order-of-ops" => build!(OrderOfOpsSpecific),
                "mean" => build!(MeanSpecific),
                other => panic!("unknown kind {other}"),
            }
        }

        /// The guarantee behind `common_content_area_cm`: on every paper
        /// and in every locale and division layout, `qs` renders on one
        /// page with the same problems per page — so the printed sheet
        /// differs only in whitespace.
        fn check_one_identical_page(kind: &str, qs: &str, failures: &mut Vec<String>) {
            let styles: &[&str] = if DIVISION_KINDS.contains(&kind) {
                &["bracket", "equation"]
            } else {
                &[""]
            };
            for style in styles {
                for locale in ["us", "no"] {
                    let mut per_page = Vec::new();
                    for paper in ["a4", "us-letter"] {
                        let mut q = format!("{qs}&paper={paper}&locale={locale}&seed=1");
                        if !style.is_empty() && !qs.contains("division=") {
                            q.push_str(&format!("&division={style}"));
                        }
                        let params = params_for(kind, q.trim_start_matches('&'));
                        match Document::from_params(&params) {
                            Err(e) => failures.push(format!("{kind}?{q}: {e}")),
                            Ok(doc) => {
                                if doc.pages != 1 {
                                    failures.push(format!(
                                        "{kind}?{q}: {} pages ({} problems, {} per page)",
                                        doc.pages, params.num_problems, doc.cells_per_page
                                    ));
                                }
                                per_page.push(doc.cells_per_page);
                            }
                        }
                    }
                    if per_page.len() == 2 && per_page[0] != per_page[1] {
                        failures.push(format!(
                            "{kind}?{qs} style={style} locale={locale}: A4 and Letter paginate differently"
                        ));
                    }
                }
            }
        }

        /// Every kind with no params — what the API serves by default.
        /// Two kinds refuse an empty query (no operation enabled), so
        /// they get the one switch they require.
        #[test]
        fn defaults_fit_one_page_on_every_paper() {
            let mut failures = Vec::new();
            for kind in KINDS {
                let qs = match *kind {
                    "algebra-one-step" => "add=true",
                    "algebra-square-root" => "squares=true",
                    _ => "",
                };
                check_one_identical_page(kind, qs, &mut failures);
            }
            failures.dedup();
            assert!(failures.is_empty(), "\n{}", failures.join("\n"));
        }

        /// Every configurator preset in `levels.json`, expanded the way
        /// `worksheetUrl` does (params, then the layout's overrides).
        #[test]
        fn levels_fit_one_page_on_every_paper() {
            let levels: serde_json::Value = serde_json::from_str(include_str!(
                "../../../frontend/astro/src/lib/levels.json"
            ))
            .unwrap();
            let to_qs = |m: &serde_json::Map<String, serde_json::Value>| {
                m.iter()
                    .map(|(k, v)| match v {
                        serde_json::Value::String(s) => format!("{k}={s}"),
                        other => format!("{k}={other}"),
                    })
                    .collect::<Vec<_>>()
                    .join("&")
            };
            let mut failures = Vec::new();
            for (kind, list) in levels.as_object().unwrap() {
                if kind.starts_with('_') {
                    continue;
                }
                for (i, level) in list.as_array().unwrap().iter().enumerate() {
                    let base = level["params"].as_object().unwrap();
                    let styles: &[&str] = if DIVISION_KINDS.contains(&kind.as_str()) {
                        &["bracket", "equation"]
                    } else {
                        &[""]
                    };
                    for style in styles {
                        let mut merged = base.clone();
                        if let Some(o) = level["styleParams"][*style].as_object() {
                            merged.extend(o.clone());
                        }
                        let mut qs = to_qs(&merged);
                        if !style.is_empty() {
                            qs.push_str(&format!("&division={style}"));
                        }
                        let before = failures.len();
                        check_one_identical_page(kind, &qs, &mut failures);
                        for f in &mut failures[before..] {
                            *f = format!("level {}: {f}", i + 1);
                        }
                    }
                }
            }
            failures.dedup();
            assert!(failures.is_empty(), "\n{}", failures.join("\n"));
        }
    }

    fn root_redirect(
        method: Method,
        path: &str,
        query: Option<&str>,
        cookie: &str,
        site: Option<&str>,
    ) -> Option<String> {
        locale_root_redirect_target(&method, path, query, cookie, site)
    }

    #[test]
    fn locale_root_redirect_on_entry_with_cookie() {
        for site in [None, Some("none"), Some("cross-site")] {
            assert_eq!(
                root_redirect(Method::GET, "/", None, "lang=no", site).as_deref(),
                Some("/no/"),
                "sec-fetch-site {site:?}",
            );
        }
        assert_eq!(
            root_redirect(Method::HEAD, "/", None, "lang=no", None).as_deref(),
            Some("/no/")
        );
        // Query string carried over; other cookies around it ignored.
        assert_eq!(
            root_redirect(Method::GET, "/", Some("utm_source=x"), "a=1; lang=no ; b=2", None)
                .as_deref(),
            Some("/no/?utm_source=x")
        );
    }

    #[test]
    fn locale_root_redirect_skips_internal_navigation() {
        for site in ["same-origin", "same-site"] {
            assert_eq!(root_redirect(Method::GET, "/", None, "lang=no", Some(site)), None);
        }
    }

    #[test]
    fn locale_root_redirect_needs_known_non_default_cookie() {
        assert_eq!(root_redirect(Method::GET, "/", None, "", None), None);
        assert_eq!(root_redirect(Method::GET, "/", None, "lang=en", None), None);
        assert_eq!(root_redirect(Method::GET, "/", None, "lang=xx", None), None);
        assert_eq!(root_redirect(Method::GET, "/", None, "notlang=no", None), None);
        assert_eq!(root_redirect(Method::GET, "/", None, "lang=", None), None);
    }

    #[test]
    fn locale_root_redirect_only_for_root_get_head() {
        for path in ["/about/", "/worksheets/add/", "/no/", "/index.html", ""] {
            assert_eq!(root_redirect(Method::GET, path, None, "lang=no", None), None, "{path}");
        }
        assert_eq!(root_redirect(Method::POST, "/", None, "lang=no", None), None);
        assert_eq!(root_redirect(Method::OPTIONS, "/", None, "lang=no", None), None);
    }

    #[test]
    fn www_redirect_target_strips_www() {
        assert_eq!(
            www_redirect_target("www.pencilready.com", "/worksheets/add/?seed=1").as_deref(),
            Some("https://pencilready.com/worksheets/add/?seed=1")
        );
        assert_eq!(
            www_redirect_target("WWW.PencilReady.com", "/").as_deref(),
            Some("https://pencilready.com/")
        );
        assert_eq!(www_redirect_target("pencilready.com", "/"), None);
        assert_eq!(www_redirect_target("localhost:8080", "/"), None);
    }

    #[test]
    fn inject_payload_ip_sets_ip() {
        let body = br#"{"type":"event","payload":{"website":"abc","url":"/"}}"#;
        let out = inject_payload_ip(body, "203.0.113.7").unwrap();
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["payload"]["ip"], "203.0.113.7");
        assert_eq!(v["payload"]["url"], "/");
        assert_eq!(v["type"], "event");
    }

    #[test]
    fn inject_payload_ip_overwrites_spoofed_ip() {
        let body = br#"{"type":"event","payload":{"website":"abc","ip":"1.2.3.4"}}"#;
        let out = inject_payload_ip(body, "203.0.113.7").unwrap();
        let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(v["payload"]["ip"], "203.0.113.7");
    }

    #[test]
    fn inject_payload_ip_passes_through_unexpected_shapes() {
        assert!(inject_payload_ip(b"not json", "203.0.113.7").is_none());
        assert!(inject_payload_ip(br#"{"no_payload":1}"#, "203.0.113.7").is_none());
        assert!(inject_payload_ip(br#"{"payload":"string"}"#, "203.0.113.7").is_none());
        assert!(inject_payload_ip(br#"{"payload":{}}"#, "-").is_none());
    }
}

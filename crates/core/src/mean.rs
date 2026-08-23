//! Mean (average) — the two-step procedure: add the values, then divide
//! by how many there are.
//!
//! Each problem is a small data set rendered as a comma-separated line,
//! with an optional pre-filled work scaffold underneath (a column
//! addition stack beside a long-division bracket). The scaffold is a
//! render-time concern; the generated data is the same either way.
//!
//! Problem encoding is `[v1, …, vn, sum, mean]`. The value count `n` is
//! derived by the typst component as `len - 2`, so a page can mix 3- and
//! 4-value problems — which is deliberate: the student has to notice how
//! many numbers there are before dividing. Dividing by the wrong count
//! is *the* classic mean error, so the worksheet exercises it rather
//! than hiding it behind a fixed divisor.
//!
//! ## Whole-number answers
//!
//! Every problem divides evenly. Rather than sampling values and
//! rejecting the ~(n-1)/n of draws whose sum isn't divisible, we pick
//! the answer first: choose a target mean `m`, then `n` deviations that
//! sum to zero. The values are `m + devᵢ`, so the sum is exactly `n·m`
//! and the mean is exactly `m` by construction. Same trick the fraction
//! and algebra generators use, and it has a second benefit — the values
//! cluster around a center, so a data set reads like a real
//! measurement series (five classmates' heights) rather than five
//! unrelated numbers.

use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType, pad_with_duplicates};

/// The sum is the long-division dividend, and the cell reserves work
/// space per dividend digit — so an unbounded sum means an unbounded
/// cell height. Capping the sum at three digits keeps a page at four
/// problems and keeps the divide step a 3-digit-by-1-digit long
/// division, which is the shape the curriculum actually teaches. A
/// 4-digit dividend costs 2cm of extra work space and drops the page to
/// two problems.
pub(crate) const MAX_SUM: u32 = 999;

/// Inclusive value range for the given digit-count range and values-per-set.
///
/// `digits` of `1-2` means values in `[1, 99]`; `3` starts at `[100, 999]`.
/// The upper bound is then pulled down so `count_max` values can't sum
/// past [`MAX_SUM`] — with four 3-digit values that lands the range at
/// 100-249, which reads like a set of heights in cm and keeps the
/// dividend at three digits.
pub(crate) fn value_range(
    digits_min: u32,
    digits_max: u32,
    count_max: u32,
    decimals: bool,
) -> (u32, u32) {
    // With `decimals`, everything is in tenths: a 2-digit value spans
    // 10.0-99.9, i.e. 100-999 scaled. Working in scaled integers is what
    // keeps the mean exact — the deviations-sum-to-zero trick operates on
    // tenths, so `sum = n * mean` holds to the last decimal place.
    let scale = if decimals { 10 } else { 1 };
    let lo = if digits_min <= 1 {
        1
    } else {
        10u32.pow(digits_min - 1)
    } * scale;
    let natural_hi = 10u32.pow(digits_max) * scale - 1;
    let sum_capped_hi = (MAX_SUM * scale) / count_max.max(1);
    // `lo + 1` floor: the range must stay non-empty even for a
    // pathological count/digits combination.
    let hi = natural_hi.min(sum_capped_hi).max(lo + 1);
    (lo, hi)
}

/// How far individual values may stray from the target mean. A quarter
/// of the span keeps every value inside `[lo, hi]` while still giving
/// visible spread — a data set of four identical numbers would make the
/// division trivial and the averaging pointless.
fn spread_for(lo: u32, hi: u32) -> u32 {
    ((hi - lo) / 4).max(1)
}

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (count, digits, decimals) = match &params.worksheet {
        WorksheetType::Mean {
            count,
            digits,
            decimals,
            ..
        } => (*count, *digits, *decimals),
        _ => unreachable!(),
    };

    let problems = generate_problems(
        params,
        count.min,
        count.max,
        digits.min,
        digits.max,
        decimals,
    );

    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            // The stack renders as column addition; the divide step uses
            // the long-division bracket, which carries no operator glyph.
            operator: "sym.plus".to_string(),
            divide_operator: String::new(),
            width_cm: 0.0,
            answer_rows: 1,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            // One entry per number in each problem row would vary with
            // the row length (3-8 values); the component reads a single
            // dp for every slot instead, so a one-element vec carries it.
            decimal_places: if decimals { vec![1] } else { Vec::new() },
            reserve_remainder: false,
        },
    })
}

fn generate_problems(
    params: &WorksheetParams,
    count_min: u32,
    count_max: u32,
    digits_min: u32,
    digits_max: u32,
    decimals: bool,
) -> Vec<Vec<u32>> {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use std::collections::HashSet;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let target = params.total_problems() as usize;
    if target == 0 {
        return Vec::new();
    }

    let (lo, hi) = value_range(digits_min, digits_max, count_max, decimals);
    let mut seen: HashSet<Vec<u32>> = HashSet::new();
    let mut problems: Vec<Vec<u32>> = Vec::new();
    let max_attempts = target.saturating_mul(400).max(2_000);
    for _ in 0..max_attempts {
        if problems.len() >= target {
            break;
        }
        if let Some(p) = sample_one(&mut rng, count_min, count_max, lo, hi) {
            if seen.insert(p.clone()) {
                problems.push(p);
            }
        }
    }
    pad_with_duplicates(&mut problems, target, &mut rng);
    problems
}

fn sample_one(
    rng: &mut rand::rngs::SmallRng,
    count_min: u32,
    count_max: u32,
    lo: u32,
    hi: u32,
) -> Option<Vec<u32>> {
    use rand::Rng;
    use rand::seq::SliceRandom;

    let n = rng.gen_range(count_min..=count_max) as usize;
    let spread = spread_for(lo, hi) as i64;
    // Keep the target mean far enough inside [lo, hi] that `m ± spread`
    // can't fall outside the range.
    let m = rng.gen_range((lo as i64 + spread)..=(hi as i64 - spread));

    // n-1 free deviations; the last one closes the sum to zero.
    let mut devs: Vec<i64> = Vec::with_capacity(n);
    let mut acc = 0i64;
    for _ in 0..n - 1 {
        let d = rng.gen_range(-spread..=spread);
        devs.push(d);
        acc += d;
    }
    let closing = -acc;
    // The closing deviation has to obey the same spread bound, otherwise
    // one value sticks out as an obvious outlier (and may leave [lo, hi]).
    if closing.abs() > spread {
        return None;
    }
    devs.push(closing);
    // Shuffle so the closing value isn't always in the last row — a
    // student would otherwise learn that the bottom number is the odd
    // one out.
    devs.shuffle(rng);

    // A data set of all-identical values makes the mean readable without
    // any work; reject it.
    if devs.iter().all(|d| *d == 0) {
        return None;
    }

    let mut values: Vec<u32> = Vec::with_capacity(n);
    for d in &devs {
        let v = m + d;
        if v < lo as i64 || v > hi as i64 {
            return None;
        }
        values.push(v as u32);
    }

    let sum: u32 = values.iter().sum();
    debug_assert_eq!(sum as i64, n as i64 * m, "sum must be exactly n·mean");

    let mut out = values;
    out.push(sum);
    out.push(m as u32);
    Some(out)
}

// ---------------------------------------------------------------------------
// Cell geometry
// ---------------------------------------------------------------------------
//
// A mean cell has two possible shapes and both `cell_size_cm` (which
// drives pagination) and `document::opts_body` (which tells the typst
// component what to draw) have to agree on which one is in play — so
// `uses_scaffold_layout` is the single source of truth for that choice.
// Everything here is mean-specific and stays in this module; the shared
// primitive-geometry helpers (`vertical_stack_width`,
// `long_division_cell`) live in `lib.rs` because several kinds read them.

use crate::{
    DigitRange, Paper, SEPARATOR_INSET_CM, VERTICAL_STACK_WORKED_DELTA_CM, chrome_height_cm,
    content_area_cm, long_division_cell, vertical_stack_height, vertical_stack_width,
};

/// Above this many values the addition stack outgrows a two-per-page
/// cell, so the layout switches to open work space.
const MAX_SCAFFOLD_COUNT: u32 = 6;
/// Gap between the addition and division halves of a scaffold cell.
const HALF_GAP_CM: f32 = 0.6;
/// Width of the "mean = ___" slot under the addition stack. Wider than
/// the stack itself, so it — not the stack — sets the left column's
/// width. Measured: "mean =" is 6 glyphs of Fira Code at 18pt plus a
/// 2em answer box.
const ANSWER_SLOT_CM: f32 = 3.6;
/// Height one data line contributes. Measured, not derived — a composite
/// cell came out 1.34cm taller than its own division half. Rounded up so
/// the reservation is never short.
const DATA_LINE_HEIGHT_CM: f32 = 1.4;
/// Per-character advance of the data line, per point of font size.
/// Measured by rendering a 46-character set: 17.98cm at 18pt, 15.99cm at
/// 16pt — Fira Code with no tracking. Deriving it from the nominal 0.6em
/// advance under-predicted by 2.6%, which is the wrong direction (a
/// too-narrow reservation overflows the cell).
const DATA_LINE_CHAR_CM_PER_PT: f32 = 0.021716;
/// Data-line size in the scaffold layout — matches
/// `problem-text-size-horizontal`.
pub(crate) const SCAFFOLD_DATA_LINE_PT: f32 = 18.0;
/// Data-line size in the open-workspace layout. Smaller because that
/// layout exists for long sets: eight 2-digit values with tenths is
/// 17.98cm at 18pt, which wraps, and 15.99cm at 16pt, which doesn't —
/// and a set reads far better on one line than broken across two.
pub(crate) const WORKSPACE_DATA_LINE_PT: f32 = 16.0;
/// Problems per page the open-workspace layout is sized for.
const WORKSPACE_ROWS_PER_PAGE: f32 = 2.0;

/// Whether a mean cell reserves the two-step scaffold (column-addition
/// stack beside a long-division bracket) or just open work space.
///
/// The scaffold can only express whole-number column addition followed
/// by single-digit long division. Past that it stops being usable: an
/// 8-row stack makes the cell too tall to fit two per page, and the
/// long-division component has no decimal support at all. So wide or
/// decimal data sets get a data line, open space, and an answer slot —
/// the student lays out the work herself.
pub(crate) fn uses_scaffold_layout(count_max: u32, decimals: bool) -> bool {
    !decimals && count_max <= MAX_SCAFFOLD_COUNT
}

/// Body content height available to one worksheet cell on the *shortest*
/// supported paper. Folding over [`Paper::ALL`] is what keeps pagination
/// paper-independent: sizing a cell against A4's taller page turns a
/// one-page worksheet into a two-page one on US Letter, which is 1.8cm
/// shorter.
fn shortest_content_height_cm() -> f32 {
    Paper::ALL
        .iter()
        .map(|p| content_area_cm(*p, chrome_height_cm(true, true, true)).1)
        .fold(f32::INFINITY, f32::min)
}

/// Narrowest content width across supported papers — where the
/// open-workspace data line has to wrap.
pub(crate) fn data_line_wrap_width_cm() -> f32 {
    let narrowest = Paper::ALL
        .iter()
        .map(|p| content_area_cm(*p, chrome_height_cm(true, true, true)).0)
        .fold(f32::INFINITY, f32::min);
    narrowest - 2.0 * SEPARATOR_INSET_CM
}

/// Writing space reserved by the open-workspace layout: whatever is left
/// of a cell once the data lines and the grid inset are accounted for,
/// with the cell itself sized to fit [`WORKSPACE_ROWS_PER_PAGE`] to a
/// page on the shortest paper. Floored to 0.1cm so float wobble can't
/// push `floor(area / cell)` down to one row.
fn workspace_height_cm(data_rows: f32) -> f32 {
    let per_cell = shortest_content_height_cm() / WORKSPACE_ROWS_PER_PAGE;
    let leftover = per_cell - DATA_LINE_HEIGHT_CM * data_rows - 2.0 * SEPARATOR_INSET_CM;
    (leftover * 10.0).floor() / 10.0
}

/// How many lines the data line wraps onto. Rust and the typst component
/// have to agree on this: Rust reserves the height, typst draws into it,
/// and a line more than was reserved would push the work area (and the
/// answer slot pinned under it) past the bottom of the cell. The width
/// estimate deliberately runs ~0.3cm long so the count errs high.
pub(crate) fn data_line_rows(count_max: u32, digits_max: u32, decimals: bool) -> f32 {
    let size = data_line_size_pt(uses_scaffold_layout(count_max, decimals));
    let natural = data_line_width_cm(count_max, digits_max, decimals, size);
    (natural / data_line_wrap_width_cm()).ceil().max(1.0)
}

/// Data-line font size for each layout.
pub(crate) fn data_line_size_pt(scaffold: bool) -> f32 {
    if scaffold {
        SCAFFOLD_DATA_LINE_PT
    } else {
        WORKSPACE_DATA_LINE_PT
    }
}

/// Height the data line reserves at the top of a cell.
pub(crate) fn data_height_cm(count_max: u32, digits_max: u32, decimals: bool) -> f32 {
    DATA_LINE_HEIGHT_CM * data_line_rows(count_max, digits_max, decimals)
}

/// Height of a scaffold cell's working area — the taller of its two
/// halves. Shared with `document.rs`, which passes it to the typst
/// component so the "mean = ___" slot can be bottom-anchored inside the
/// same box (costing no extra height).
pub(crate) fn body_height_cm(count_max: u32, sum_digits: u32) -> f32 {
    let (_, div_h) = long_division_cell(sum_digits);
    let stack_h = vertical_stack_height(count_max) + VERTICAL_STACK_WORKED_DELTA_CM;
    stack_h.max(div_h)
}

/// Natural (unwrapped) width of the data line: `count` values of
/// `digits` digits — plus a point and a tenth when `decimals` — joined
/// by ", ".
fn data_line_width_cm(count: u32, digits: u32, decimals: bool, size_pt: f32) -> f32 {
    let per_value = digits + if decimals { 2 } else { 0 };
    let chars = count * per_value + count.saturating_sub(1) * 2;
    chars as f32 * DATA_LINE_CHAR_CM_PER_PT * size_pt + 0.3
}

/// Height the component should fill below its data line, given the row
/// the grid actually laid out. `cell_size_cm` reserves the *minimum* a
/// cell needs; the grid then splits the content area into equal `1fr`
/// rows, so the real row is that tall or taller. Filling it is what
/// keeps the gap under the answer slot equal to the gap above the data
/// line — otherwise the slack all lands at the bottom.
pub(crate) fn fill_height_cm(
    row_height_cm: f32,
    count_max: u32,
    digits_max: u32,
    decimals: bool,
) -> f32 {
    let inner = row_height_cm
        - 2.0 * SEPARATOR_INSET_CM
        - data_height_cm(count_max, digits_max, decimals);
    inner.max(1.0)
}

/// Natural cell rectangle for a mean worksheet. `max_digits` is the
/// sum's digit count — the widest number printed, and the long-division
/// dividend.
pub(crate) fn cell_size_cm(
    count: DigitRange,
    digits: DigitRange,
    decimals: bool,
    max_digits: u32,
) -> (f32, f32) {
    // Separator hairlines are on for this kind, so the grid insets every
    // cell — that has to be reserved here too.
    let pad = 2.0 * SEPARATOR_INSET_CM;
    let scaffold = uses_scaffold_layout(count.max, decimals);
    let data_w = data_line_width_cm(count.max, digits.max, decimals, data_line_size_pt(scaffold));
    let data_rows = data_line_rows(count.max, digits.max, decimals);
    if scaffold {
        let (div_w, _) = long_division_cell(max_digits);
        // The stack's answer row holds the sum, so it must be as wide as
        // the sum, not just as the operands — and the answer slot below
        // it is wider still, so that sets the left column's width.
        let stack_w = vertical_stack_width(max_digits).max(ANSWER_SLOT_CM);
        (
            f32::max(data_w, stack_w + HALF_GAP_CM + div_w) + pad,
            DATA_LINE_HEIGHT_CM * data_rows + body_height_cm(count.max, max_digits) + pad,
        )
    } else {
        (
            data_w.min(data_line_wrap_width_cm()) + pad,
            DATA_LINE_HEIGHT_CM * data_rows + workspace_height_cm(data_rows) + pad,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DigitRange, Locale, Paper};

    fn params(count: DigitRange, digits: DigitRange) -> WorksheetParams {
        params_dp(count, digits, false)
    }

    fn params_dp(count: DigitRange, digits: DigitRange, decimals: bool) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::Mean {
                count,
                digits,
                scaffold: true,
                decimals,
            },
            num_problems: 12,
            cols: 2,
            paper: Paper::A4,
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Locale::Us,
            solve_first: false,
            include_answers: false,
            student_name: None,
            instructions: None,
            share_url: None,
        }
    }

    /// Every problem must encode `[v1..vn, sum, mean]` consistently:
    /// the sum is the sum of the values, and the mean divides evenly.
    fn assert_well_formed(problems: &[Vec<u32>], lo: u32, hi: u32, n_min: usize, n_max: usize) {
        for nums in problems {
            assert!(nums.len() >= 4, "need at least 2 values + sum + mean: {nums:?}");
            let n = nums.len() - 2;
            assert!(
                (n_min..=n_max).contains(&n),
                "value count {n} outside {n_min}-{n_max}: {nums:?}"
            );
            let values = &nums[..n];
            let sum = nums[n];
            let mean = nums[n + 1];
            assert_eq!(
                values.iter().sum::<u32>(),
                sum,
                "encoded sum doesn't match values: {nums:?}"
            );
            assert_eq!(
                sum,
                mean * n as u32,
                "mean must be exact (sum = mean × count): {nums:?}"
            );
            for v in values {
                assert!(
                    (lo..=hi).contains(v),
                    "value {v} outside {lo}-{hi}: {nums:?}"
                );
            }
        }
    }

    #[test]
    fn level_one_small_values() {
        let p = params(DigitRange::new(3, 4), DigitRange::new(1, 2));
        let problems = generate_problems(&p, 3, 4, 1, 2, false);
        assert_eq!(problems.len(), 12);
        assert_well_formed(&problems, 1, 99, 3, 4);
    }

    #[test]
    fn level_two_three_digit_values() {
        let p = params(DigitRange::new(3, 4), DigitRange::fixed(3));
        let problems = generate_problems(&p, 3, 4, 3, 3, false);
        assert_eq!(problems.len(), 12);
        assert_well_formed(&problems, 100, 249, 3, 4);
    }

    /// The whole point of a 3-or-4 range: a page should mix both counts,
    /// so the student has to check how many values there are instead of
    /// reusing one divisor down the column.
    #[test]
    fn mixes_value_counts() {
        let p = params(DigitRange::new(3, 4), DigitRange::new(1, 2));
        let problems = generate_problems(&p, 3, 4, 1, 2, false);
        let mut saw_three = false;
        let mut saw_four = false;
        for nums in &problems {
            match nums.len() - 2 {
                3 => saw_three = true,
                4 => saw_four = true,
                other => panic!("unexpected value count {other}"),
            }
        }
        assert!(saw_three, "no 3-value problems generated");
        assert!(saw_four, "no 4-value problems generated");
    }

    #[test]
    fn rejects_all_identical_values() {
        let p = params(DigitRange::new(3, 4), DigitRange::new(1, 2));
        let problems = generate_problems(&p, 3, 4, 1, 2, false);
        for nums in &problems {
            let n = nums.len() - 2;
            let values = &nums[..n];
            assert!(
                values.iter().any(|v| *v != values[0]),
                "all values identical, no averaging to do: {nums:?}"
            );
        }
    }

    /// Level 3: eight 2-digit values with a tenth place. The mean has to
    /// stay exact in tenths — `sum = count * mean` in scaled units —
    /// otherwise the answer key would round and the worksheet would be
    /// unmarkable.
    #[test]
    fn tenths_keep_the_mean_exact() {
        let p = params_dp(DigitRange::fixed(8), DigitRange::fixed(2), true);
        let problems = generate_problems(&p, 8, 8, 2, 2, true);
        assert_eq!(problems.len(), 12);
        for nums in &problems {
            let n = nums.len() - 2;
            assert_eq!(n, 8, "expected 8 values: {nums:?}");
            let values = &nums[..n];
            let sum = nums[n];
            let mean = nums[n + 1];
            assert_eq!(values.iter().sum::<u32>(), sum, "sum mismatch: {nums:?}");
            assert_eq!(sum, mean * n as u32, "mean not exact: {nums:?}");
            for v in values {
                assert!(
                    (100..=999).contains(v),
                    "scaled value {v} outside 10.0-99.9: {nums:?}"
                );
            }
        }
    }

    /// The three shipped levels each have to fit one page on *every*
    /// supported paper. This is the regression guard for a bug that
    /// already happened once: the open-workspace height was tuned against
    /// A4's content area, and US Letter — 1.8cm shorter — silently
    /// paginated the same two problems onto two pages.
    #[test]
    fn shipped_levels_fit_one_page_on_every_paper() {
        // (count, digits, decimals, problems, cols) mirroring
        // WORKSHEET_LEVELS["mean"] in frontend/astro/src/lib/levels.ts.
        let levels: &[(DigitRange, DigitRange, bool, u32, u32)] = &[
            (DigitRange::new(3, 4), DigitRange::new(1, 2), false, 4, 2),
            (DigitRange::new(3, 4), DigitRange::fixed(3), false, 4, 2),
            (DigitRange::fixed(8), DigitRange::fixed(2), true, 2, 1),
        ];
        for (idx, (count, digits, decimals, problems, cols)) in levels.iter().enumerate() {
            for paper in Paper::ALL {
                let mut p = params_dp(*count, *digits, *decimals);
                p.num_problems = *problems;
                p.cols = *cols;
                p.paper = *paper;
                let doc = crate::Document::from_params(&p).unwrap_or_else(|e| {
                    panic!("level {} on {paper}: {e:#}", idx + 1)
                });
                assert_eq!(
                    doc.pages,
                    1,
                    "level {} put {problems} problems on {} pages on {paper}",
                    idx + 1,
                    doc.pages,
                );
            }
        }
    }

    #[test]
    fn reproducible_with_seed() {
        let p = params(DigitRange::new(3, 4), DigitRange::new(1, 2));
        let a = generate_problems(&p, 3, 4, 1, 2, false);
        let b = generate_problems(&p, 3, 4, 1, 2, false);
        assert_eq!(a, b);
    }

    #[test]
    fn value_range_reads_digit_bounds() {
        // Small values: four of them sum to at most 396, so the natural
        // upper bound survives.
        assert_eq!(value_range(1, 2, 4, false), (1, 99));
        assert_eq!(value_range(2, 2, 4, false), (10, 99));
        // Three-digit values get pulled down so four of them stay under
        // MAX_SUM — 4 x 249 = 996.
        assert_eq!(value_range(3, 3, 4, false), (100, 249));
        assert_eq!(value_range(3, 3, 3, false), (100, 333));
        // Tenths: a 2-digit value spans 10.0-99.9, i.e. 100-999 scaled.
        // Eight of them cap at 9990/8, which doesn't bite.
        assert_eq!(value_range(2, 2, 8, true), (100, 999));
    }

    /// The cell reserves long-division work space by dividend digits, so
    /// a sum that outgrows three digits would silently make every cell
    /// 2cm taller and halve the problems per page.
    #[test]
    fn sums_never_exceed_three_digits() {
        for (dmin, dmax) in [(1u32, 2u32), (3, 3), (2, 3)] {
            let p = params(DigitRange::new(3, 4), DigitRange::new(dmin, dmax));
            let problems = generate_problems(&p, 3, 4, dmin, dmax, false);
            for nums in &problems {
                let n = nums.len() - 2;
                assert!(
                    nums[n] <= super::MAX_SUM,
                    "sum {} exceeds MAX_SUM for digits {dmin}-{dmax}: {nums:?}",
                    nums[n]
                );
            }
        }
    }
}

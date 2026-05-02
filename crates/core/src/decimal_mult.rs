//! Decimal multiplication.
//!
//! Both operands are encoded as scaled integers; the answer's decimal
//! places equal the sum of the operands'. With
//! `bottom_decimal_places=0` the multiplier renders as a whole number
//! (e.g. `2.5 × 3`); with `bottom_decimal_places=1` it renders with one
//! decimal place (`123.4 × 5.6`). The typst component handles the
//! per-slot decimal-point placement via the `decimal-places: (top_dp,
//! bot_dp, ans_dp)` opt.

use crate::document;
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (digits, top_dp, mult_min, mult_max, bot_dp) = match &params.worksheet {
        WorksheetType::DecimalMultiply {
            digits,
            decimal_places,
            multiplier_min,
            multiplier_max,
            bottom_decimal_places,
        } => (
            *digits,
            *decimal_places,
            *multiplier_min,
            *multiplier_max,
            *bottom_decimal_places,
        ),
        _ => unreachable!(),
    };

    let problems = generate_problems(digits, top_dp, mult_min, mult_max, bot_dp, params);
    let max_digits = document::max_digits(&problems);
    let operator = params
        .symbol
        .clone()
        .unwrap_or_else(|| "sym.times".to_string());

    // Per-slot dp: top, bottom, answer (= top + bottom).
    let dp_list = vec![top_dp, bot_dp, top_dp + bot_dp];

    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            operator,
            divide_operator: String::new(),
            width_cm: document::box_width_cm(&params.worksheet, max_digits),
            answer_rows: 1,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            decimal_places: dp_list,
            reserve_remainder: false,
        },
    })
}

fn generate_problems(
    digits: crate::DigitRange,
    top_dp: u32,
    mult_min: u32,
    mult_max: u32,
    bot_dp: u32,
    params: &WorksheetParams,
) -> Vec<Vec<u32>> {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let top_scale = 10u32.pow(top_dp);
    let bot_scale = 10u32.pow(bot_dp);

    let pick_top = |rng: &mut SmallRng| -> u32 {
        let int_part = digits.random(rng);
        let frac = if top_scale > 1 {
            rng.gen_range(0..top_scale)
        } else {
            0
        };
        int_part * top_scale + frac
    };
    let pick_mult = |rng: &mut SmallRng| -> u32 {
        let int_part = rng.gen_range(mult_min..=mult_max);
        let frac = if bot_scale > 1 {
            rng.gen_range(0..bot_scale)
        } else {
            0
        };
        int_part * bot_scale + frac
    };

    let total = params.total_problems();
    let max_attempts = total * 100;
    let mut problems = Vec::with_capacity(total as usize);
    let mut attempts = 0;

    while problems.len() < total as usize && attempts < max_attempts {
        let a = pick_top(&mut rng);
        let b = pick_mult(&mut rng);
        attempts += 1;
        // Skip degenerate × 0 problems (they're trivial).
        if b == 0 {
            continue;
        }
        let candidate = vec![a, b];
        if !problems.contains(&candidate) {
            problems.push(candidate);
        }
    }

    crate::pad_with_duplicates(&mut problems, total as usize, &mut rng);

    problems
        .into_iter()
        .map(|nums| vec![nums[0], nums[1], nums[0] * nums[1]])
        .collect()
}

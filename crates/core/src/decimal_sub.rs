//! Decimal subtraction worksheet.
//!
//! Same encoding as `decimal_add`: scaled integers, uniform
//! `decimal_places` per worksheet. First operand is always ≥ second so
//! the answer is non-negative.

use crate::document;
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (digits, decimal_places) = match &params.worksheet {
        WorksheetType::DecimalSubtract {
            digits,
            decimal_places,
        } => (digits.clone(), *decimal_places),
        _ => unreachable!(),
    };

    let problems = generate_problems(&digits, decimal_places, params);
    let max_digits = document::max_digits(&problems);
    let operator = params
        .symbol
        .clone()
        .unwrap_or_else(|| "sym.minus".to_string());

    let dp_list = vec![decimal_places; digits.len() + 1];

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
    digits: &[crate::DigitRange],
    decimal_places: u32,
    params: &WorksheetParams,
) -> Vec<Vec<u32>> {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let scale = 10u32.pow(decimal_places);
    let pick = |range: crate::DigitRange, rng: &mut SmallRng| -> u32 {
        let int_part = range.random(rng);
        let frac = rng.gen_range(0..scale);
        int_part * scale + frac
    };

    let top_range = digits[0];
    let bot_range = *digits.get(1).unwrap_or(&top_range);

    let total = params.total_problems();
    let max_attempts = total * 100;
    let mut problems = Vec::with_capacity(total as usize);
    let mut attempts = 0;

    while problems.len() < total as usize && attempts < max_attempts {
        let mut a = pick(top_range, &mut rng);
        let mut b = pick(bot_range, &mut rng);
        attempts += 1;
        if a < b {
            std::mem::swap(&mut a, &mut b);
        }
        let candidate = vec![a, b];
        if !problems.contains(&candidate) {
            problems.push(candidate);
        }
    }

    crate::pad_with_duplicates(&mut problems, total as usize, &mut rng);

    problems
        .into_iter()
        .map(|nums| vec![nums[0], nums[1], nums[0] - nums[1]])
        .collect()
}

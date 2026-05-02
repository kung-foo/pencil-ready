//! Decimal addition worksheet.
//!
//! Operands and answer are stored as scaled integers (e.g. `1.23` →
//! `123`). All numbers on a worksheet share the same `decimal_places`,
//! so the printed decimal points line up column-wise.

use crate::document;
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (digits, decimal_places) = match &params.worksheet {
        WorksheetType::DecimalAdd {
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
        .unwrap_or_else(|| "sym.plus".to_string());

    // Same dp for every slot (operands + answer). Number of slots =
    // operand count + 1.
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
    // Pick the integer-part value and a random fraction in [0, scale).
    // Encoded result = int * scale + frac.
    let pick = |range: crate::DigitRange, rng: &mut SmallRng| -> u32 {
        let int_part = range.random(rng);
        let frac = rng.gen_range(0..scale);
        int_part * scale + frac
    };

    let total = params.total_problems();
    let max_attempts = total * 100;
    let mut problems = Vec::with_capacity(total as usize);
    let mut attempts = 0;

    while problems.len() < total as usize && attempts < max_attempts {
        let nums: Vec<u32> = digits.iter().map(|d| pick(*d, &mut rng)).collect();
        attempts += 1;
        if !problems.contains(&nums) {
            problems.push(nums);
        }
    }

    crate::pad_with_duplicates(&mut problems, total as usize, &mut rng);

    // Append the encoded sum.
    problems
        .into_iter()
        .map(|nums| {
            let sum = nums.iter().sum::<u32>();
            let mut row = nums;
            row.push(sum);
            row
        })
        .collect()
}

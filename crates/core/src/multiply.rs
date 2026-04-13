//! Multiplication worksheet.

use crate::template;
use crate::{WorksheetParams, WorksheetType};

pub fn generate_typ(params: &WorksheetParams) -> String {
    let problems = generate_problems(params);
    // Space needed below the line for partial products + final answer.
    //   1-digit multiplier → 1 row (just the product)
    //   N-digit multiplier (N ≥ 2) → N partials + 1 final sum = N+1 rows
    let max_multiplier = problems
        .iter()
        .map(|nums| nums[1])
        .max()
        .unwrap_or(0);
    let mult_digits = if max_multiplier == 0 { 1 } else { max_multiplier.ilog10() + 1 };
    let answer_rows = if mult_digits <= 1 { 1 } else { mult_digits + 1 };
    template::render("sym.times", &problems, params, answer_rows)
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let digits = match &params.worksheet {
        WorksheetType::Multiply { digits } => digits,
        _ => unreachable!(),
    };

    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let top_range = digits[0];
    let bot_range = *digits.get(1).unwrap_or(&top_range);

    let max_attempts = params.num_problems * 100;
    let mut problems = Vec::with_capacity(params.num_problems as usize);
    let mut attempts = 0;

    while problems.len() < params.num_problems as usize && attempts < max_attempts {
        let a = top_range.random(&mut rng);
        let b = bot_range.random(&mut rng);
        let candidate = vec![a, b];
        attempts += 1;

        if !problems.contains(&candidate) {
            problems.push(candidate);
        }
    }

    problems
}

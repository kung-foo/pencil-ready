//! Multiplication worksheet.

use crate::template;
use crate::{WorksheetParams, WorksheetType};

pub fn generate_typ(params: &WorksheetParams) -> String {
    let problems = generate_problems(params);
    template::render("sym.times", &problems, params)
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

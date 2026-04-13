//! Division worksheets (simple + long).

use crate::template;
use crate::{WorksheetParams, WorksheetType};

/// Simple division: times-table recall. Divisor (2-9) × quotient (1-max_quotient).
pub fn generate_simple(params: &WorksheetParams) -> anyhow::Result<String> {
    let max_quotient = match &params.worksheet {
        WorksheetType::SimpleDivision { max_quotient } => *max_quotient,
        _ => unreachable!(),
    };

    let problems = generate_simple_problems(params, max_quotient);
    template::render("sym.div", &problems, params, 1)
}

/// Long division: algorithm practice. Dividend has N digits, divisor is 1 digit (2-9).
pub fn generate_long(params: &WorksheetParams) -> anyhow::Result<String> {
    let (digit_range, remainder) = match &params.worksheet {
        WorksheetType::LongDivision { digits, remainder } => (*digits, *remainder),
        _ => unreachable!(),
    };

    let problems = generate_long_problems(params, digit_range, remainder);
    // Solve space: roughly 2 rows per dividend digit (multiply + bring-down).
    // Use the max dividend in the set so all cells get uniform sizing.
    let max_dividend_digits = problems
        .iter()
        .map(|nums| if nums[0] == 0 { 1 } else { nums[0].ilog10() + 1 })
        .max()
        .unwrap_or(1);
    let answer_rows = 2 * max_dividend_digits;
    template::render_long_division(&problems, params, answer_rows)
}

fn generate_simple_problems(params: &WorksheetParams, max_quotient: u32) -> Vec<Vec<u32>> {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let total = params.total_problems();
    let max_attempts = total * 100;
    let mut problems = Vec::with_capacity(total as usize);
    let mut attempts = 0;

    while problems.len() < total as usize && attempts < max_attempts {
        let divisor = rng.gen_range(2..=9u32);
        let quotient = rng.gen_range(1..=max_quotient);
        let dividend = divisor * quotient;
        let candidate = vec![dividend, divisor];
        attempts += 1;

        if !problems.contains(&candidate) {
            problems.push(candidate);
        }
    }

    // Append the quotient so the typst component can render the answer
    // in solved mode.
    problems
        .into_iter()
        .map(|nums| vec![nums[0], nums[1], nums[0] / nums[1]])
        .collect()
}

fn generate_long_problems(
    params: &WorksheetParams,
    digit_range: crate::DigitRange,
    remainder: bool,
) -> Vec<Vec<u32>> {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let total = params.total_problems();
    let max_attempts = total * 100;
    let mut problems = Vec::with_capacity(total as usize);
    let mut attempts = 0;

    while problems.len() < total as usize && attempts < max_attempts {
        let divisor = rng.gen_range(2..=9u32);
        // Pick a random digit count from the range for this problem.
        let dd = rng.gen_range(digit_range.min..=digit_range.max);
        let dividend_min = 10u32.pow(dd - 1);
        let dividend_max = 10u32.pow(dd) - 1;
        attempts += 1;

        let candidate = if remainder {
            let dividend = rng.gen_range(dividend_min..=dividend_max);
            // With remainder: compute quotient for the solved render.
            // Remainder is not currently rendered in solved mode.
            vec![dividend, divisor, dividend / divisor]
        } else {
            let q_min = (dividend_min + divisor - 1) / divisor;
            let q_max = dividend_max / divisor;
            if q_min > q_max {
                continue;
            }
            let quotient = rng.gen_range(q_min..=q_max);
            let dividend = divisor * quotient;
            vec![dividend, divisor, quotient]
        };

        if !problems.contains(&candidate) {
            problems.push(candidate);
        }
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_no_remainder() {
        let params = simple_params(10);
        let problems = generate_simple_problems(&params, 10);
        assert_eq!(problems.len(), 12);
        for nums in &problems {
            let (dividend, divisor) = (nums[0], nums[1]);
            assert!(divisor >= 2 && divisor <= 9);
            assert_eq!(dividend % divisor, 0);
            assert!(dividend / divisor <= 10);
        }
    }

    #[test]
    fn test_simple_max_quotient_5() {
        let params = simple_params(5);
        let problems = generate_simple_problems(&params, 5);
        for nums in &problems {
            assert!(nums[0] / nums[1] <= 5);
        }
    }

    #[test]
    fn test_long_clean() {
        let params = long_params(3, false);
        let problems = generate_long_problems(&params, crate::DigitRange::fixed(3), false);
        assert_eq!(problems.len(), 12);
        for nums in &problems {
            let (dividend, divisor) = (nums[0], nums[1]);
            assert!(dividend >= 100 && dividend <= 999);
            assert!(divisor >= 2 && divisor <= 9);
            assert_eq!(dividend % divisor, 0);
        }
    }

    #[test]
    fn test_long_with_remainder() {
        let params = long_params(2, true);
        let problems = generate_long_problems(&params, crate::DigitRange::fixed(2), true);
        for nums in &problems {
            assert!(nums[0] >= 10 && nums[0] <= 99);
        }
    }

    fn simple_params(max_quotient: u32) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::SimpleDivision { max_quotient },
            num_problems: 12,
            cols: 4,
            paper: "a4".into(),
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Default::default(),
            pages: 1,
            solve_first: false,
        }
    }

    fn long_params(digits: u32, remainder: bool) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::LongDivision { digits: crate::DigitRange::fixed(digits), remainder },
            num_problems: 12,
            cols: 4,
            paper: "a4".into(),
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Default::default(),
            pages: 1,
            solve_first: false,
        }
    }
}

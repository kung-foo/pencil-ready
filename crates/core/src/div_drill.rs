//! Division drill — horizontal times-table recall (reversed).
//!
//! Same facts as multiplication drill but reversed:
//!   56 ÷ 7 = ___  (instead of 7 × 8 = ___)

use crate::document;
use crate::{ComponentOpts, DigitRange, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (divisor_ranges, max_quotient) = match &params.worksheet {
        WorksheetType::DivisionDrill { divisor, max_quotient } => {
            (divisor, *max_quotient)
        }
        _ => unreachable!(),
    };

    let problems = generate_problems(params, divisor_ranges, max_quotient);
    let operator = params
        .symbol
        .clone()
        .unwrap_or_else(|| params.locale.divide_symbol().to_string());
    let max_digits = document::max_digits(&problems);
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            operator,
            width_cm: document::box_width_cm(&params.worksheet, max_digits),
            answer_rows: 1,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
        },
    })
}

fn generate_problems(
    params: &WorksheetParams,
    divisor_ranges: &[DigitRange],
    max_quotient: DigitRange,
) -> Vec<Vec<u32>> {
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    // Expand divisor ranges into a flat set.
    let divisors: Vec<u32> = divisor_ranges
        .iter()
        .flat_map(|r| r.min..=r.max)
        .collect();

    let quotients: Vec<u32> = (max_quotient.min..=max_quotient.max).collect();

    // Generate all pairs: dividend = divisor × quotient.
    // Division isn't commutative, so no dedup needed.
    let mut pairs: Vec<(u32, u32)> = Vec::new();
    for &d in &divisors {
        for &q in &quotients {
            let dividend = d * q;
            pairs.push((dividend, d));
        }
    }

    // Remove exact duplicates (e.g. different divisor×quotient yielding same problem).
    pairs.sort();
    pairs.dedup();

    // Shuffle.
    pairs.shuffle(&mut rng);

    // 0 = all problems.
    if params.num_problems > 0 {
        pairs.truncate(params.total_problems() as usize);
    }

    // Include the quotient as a third element so the typst component can
    // render the answer when `solved` is on.
    pairs.iter().map(|&(dividend, divisor)| vec![dividend, divisor, dividend / divisor]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sevens_table() {
        let params = test_params(vec![DigitRange::fixed(7)], DigitRange::new(1, 10));
        let problems = generate_problems(
            &params,
            &[DigitRange::fixed(7)],
            DigitRange::new(1, 10),
        );
        assert_eq!(problems.len(), 10);
        for nums in &problems {
            assert_eq!(nums[0] % nums[1], 0, "{} ÷ {} has remainder", nums[0], nums[1]);
            assert_eq!(nums[1], 7);
        }
    }

    #[test]
    fn test_all_clean_division() {
        let ranges = vec![DigitRange::new(2, 9)];
        let params = test_params(ranges.clone(), DigitRange::new(1, 10));
        let problems = generate_problems(&params, &ranges, DigitRange::new(1, 10));
        for nums in &problems {
            assert_eq!(nums[0] % nums[1], 0, "{} ÷ {} has remainder", nums[0], nums[1]);
        }
    }

    fn test_params(divisor: Vec<DigitRange>, max_quotient: DigitRange) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::DivisionDrill { divisor, max_quotient },
            num_problems: 0,
            cols: 3,
            paper: crate::Paper::A4,
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Default::default(),
            pages: 1,
            solve_first: false,
            include_answers: false,
            student_name: None,
        }
    }
}

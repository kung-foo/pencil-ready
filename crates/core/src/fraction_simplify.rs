//! Fraction simplification worksheet — `num/den = ___`.
//!
//! Student rewrites the fraction in its simplest form. The simplified
//! form is one of:
//!   - a reduced proper fraction (e.g. 6/8 → 3/4)
//!   - a mixed number          (e.g. 11/4 → 2 3/4)
//!   - the same fraction       (e.g. 7/12 → 7/12, already in lowest terms)
//!   - a whole number          (e.g. 12/4 → 3, only when `include_whole` is set)

use crate::document;
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let problems = generate_problems(params);
    let max_digits = document::max_digits(&problems);
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        // No operator in this worksheet — the fraction bar and `=` are
        // universal. `ComponentOpts` keys are ignored by the component
        // (see `opts_body` in document.rs emits an empty `opts: ()`).
        opts: ComponentOpts {
            operator: String::new(),
            width_cm: document::box_width_cm(&params.worksheet, max_digits),
            answer_rows: 0,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
        },
    })
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (denominators, max_numerator, include_improper, include_whole) = match &params.worksheet {
        WorksheetType::FractionSimplify {
            denominators,
            max_numerator,
            include_improper,
            include_whole,
        } => (
            denominators,
            *max_numerator,
            *include_improper,
            *include_whole,
        ),
        _ => unreachable!(),
    };

    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    // Enumerate every valid (num, den) pair. Filtering rules:
    //   - skip num == den (unless include_whole): degenerate, answer is 1.
    //   - skip num > den that reduces to a whole (unless include_whole).
    //   - skip num >= den when !include_improper.
    let mut all: Vec<(u32, u32)> = Vec::new();
    for &den in denominators {
        for num in 1..=max_numerator {
            if !include_improper && num >= den {
                continue;
            }
            let rd = den / gcd(num, den);
            // rd == 1 means the reduced denominator collapses — answer is
            // a pure whole number. Exclude unless explicitly requested.
            if !include_whole && rd == 1 {
                continue;
            }
            all.push((num, den));
        }
    }

    all.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    if total > 0 && all.len() > total {
        all.truncate(total);
    }

    // Narrow search spaces (e.g. denominators=[2] with no improper, no
    // whole) can leave us under target. Pad with duplicates rather than
    // returning a short worksheet — same policy as add/subtract.
    crate::pad_with_duplicates(&mut all, total, &mut rng);

    all.into_iter().map(|(n, d)| vec![n, d]).collect()
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_whole_answers_by_default() {
        // 4/2, 6/2, 6/3, 9/3 etc. all reduce to whole numbers — excluded.
        let params = test_params(vec![2, 3, 4], 20, true, false);
        let problems = generate_problems(&params);
        for p in &problems {
            let (n, d) = (p[0], p[1]);
            let rd = d / gcd(n, d);
            assert!(rd > 1, "{n}/{d} reduces to a whole — should be excluded");
        }
    }

    #[test]
    fn proper_only_when_improper_disabled() {
        let params = test_params(vec![4, 6, 8], 20, false, false);
        let problems = generate_problems(&params);
        for p in &problems {
            let (n, d) = (p[0], p[1]);
            assert!(n < d, "{n}/{d} is improper but improper was disabled");
        }
    }

    #[test]
    fn includes_mix_of_reducible_and_already_reduced() {
        // With denominators 2-12 and max_num 20, the pool is large and
        // should contain both reducible proper (like 6/8) and coprime
        // (like 7/12) fractions.
        let params = test_params(vec![2, 3, 4, 5, 6, 8, 10, 12], 20, true, false);
        let problems = generate_problems(&params);
        assert!(problems.len() >= 12);

        let mut any_reducible = false;
        let mut any_already_reduced = false;
        for p in &problems {
            let (n, d) = (p[0], p[1]);
            if gcd(n, d) == 1 {
                any_already_reduced = true;
            } else {
                any_reducible = true;
            }
        }
        assert!(any_reducible, "no reducible fractions in sample");
        assert!(any_already_reduced, "no already-reduced fractions in sample");
    }

    fn test_params(
        denominators: Vec<u32>,
        max_numerator: u32,
        include_improper: bool,
        include_whole: bool,
    ) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::FractionSimplify {
                denominators,
                max_numerator,
                include_improper,
                include_whole,
            },
            num_problems: 12,
            cols: 3,
            paper: crate::Paper::A4,
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Default::default(),
            solve_first: false,
            include_answers: false,
            student_name: None,
        }
    }
}

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
            divide_operator: String::new(),
            width_cm: document::box_width_cm(&params.worksheet, max_digits),
            answer_rows: 0,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            decimal_places: Vec::new(),
            reserve_remainder: false,
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

    // Enumerate every valid (num, den) pair, splitting into two pools:
    //   - `needs_work`: requires reduction or improper→mixed conversion.
    //   - `already_simplest`: proper fraction already in lowest terms
    //     (no work for the student). Capped to keep the worksheet useful.
    // Filtering rules:
    //   - skip num >= den when !include_improper.
    //   - skip num > den that reduces to a whole (unless include_whole).
    let mut needs_work: Vec<(u32, u32)> = Vec::new();
    let mut already_simplest: Vec<(u32, u32)> = Vec::new();
    for &den in denominators {
        for num in 1..=max_numerator {
            if !include_improper && num >= den {
                continue;
            }
            let g = gcd(num, den);
            let rd = den / g;
            // rd == 1 means the reduced denominator collapses — answer is
            // a pure whole number. Exclude unless explicitly requested.
            if !include_whole && rd == 1 {
                continue;
            }
            // A proper fraction with gcd 1 is already in simplest form —
            // the student writes it back unchanged. Improper fractions
            // still need mixed-number conversion even when coprime.
            if num < den && g == 1 {
                already_simplest.push((num, den));
            } else {
                needs_work.push((num, den));
            }
        }
    }

    needs_work.shuffle(&mut rng);
    already_simplest.shuffle(&mut rng);

    let total = params.total_problems() as usize;
    const MAX_ALREADY_SIMPLEST: usize = 2;

    let mut all: Vec<(u32, u32)> = if needs_work.is_empty() {
        // Degenerate config (e.g. only 1/2 available) — fall back to
        // already-simplest so we still produce a worksheet.
        already_simplest
    } else {
        let already_take = MAX_ALREADY_SIMPLEST
            .min(already_simplest.len())
            .min(total);
        let needs_target = total.saturating_sub(already_take);
        if total > 0 && needs_work.len() > needs_target {
            needs_work.truncate(needs_target);
        }
        // Pad needs-work first so duplicates don't push us over the
        // already-simplest cap.
        crate::pad_with_duplicates(&mut needs_work, needs_target, &mut rng);
        needs_work.extend(already_simplest.into_iter().take(already_take));
        needs_work
    };

    all.shuffle(&mut rng);
    if total > 0 && all.len() > total {
        all.truncate(total);
    }
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
    fn caps_already_simplest_proper_fractions() {
        // Pool has many coprime proper fractions (7/12, 11/12, 5/6, …).
        // Without a cap, ~half of problems would be no-work copy-overs.
        let params = test_params(vec![2, 3, 4, 5, 6, 8, 10, 12], 20, false, false);
        let problems = generate_problems(&params);
        let count = problems
            .iter()
            .filter(|p| p[0] < p[1] && gcd(p[0], p[1]) == 1)
            .count();
        assert!(
            count <= 2,
            "expected at most 2 already-simplest proper fractions, got {count}"
        );
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
            instructions: None,
            share_url: None,
        }
    }
}

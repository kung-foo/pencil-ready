//! One-step linear equation worksheet — solve for `x` in a single move.
//!
//! Four forms, each one operation away from the answer:
//!   form = 0 → addition       `x + b = c` → x = c − b
//!   form = 1 → subtraction    `x − b = c` → x = c + b
//!   form = 2 → multiplication `a · x = c` → x = c ÷ a
//!   form = 3 → division       `x ÷ a = c` → x = c · a
//!
//! Each form is a separate toggle; defaults turn on add + subtract so the
//! first encounter mirrors what students see right after introducing
//! variables. Multiply and divide layer in once times-tables are fluent.
//!
//! Problem representation as `Vec<u32>`: `[form, p, x, c]` where `p` is
//! either the constant `b` (forms 0, 1) or the coefficient/divisor `a`
//! (forms 2, 3). `x` is the answer. `c` is the RHS.
//!
//! All answers are non-negative integers by construction:
//!   - subtraction skips triples with `x < b` (would make `c < 0`)
//!   - division iterates `x` in multiples of `a` (clean integer `c`)

use crate::document;
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let problems = generate_problems(params);
    let variable = match &params.worksheet {
        WorksheetType::AlgebraOneStep { variable, .. } => variable.clone(),
        _ => unreachable!(),
    };
    // Algebra renders `·` for multiplication regardless of locale —
    // matches algebra-two-step's reasoning (× looks too much like the
    // variable x). The explicit --symbol flag overrides the multiply
    // operator. Division uses the locale's horizontal symbol (÷ in US,
    // : in Norway) since there's no x-vs-÷ confusion to dodge.
    let mult_op = params
        .symbol
        .clone()
        .unwrap_or_else(|| "sym.dot.op".to_string());
    let div_op = params.locale.divide_symbol().to_string();
    let max_digits = document::max_digits(&problems);
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            operator: mult_op,
            divide_operator: div_op,
            width_cm: document::box_width_cm(&params.worksheet, max_digits),
            answer_rows: 1,
            pad_width: 0,
            implicit: false,
            variable,
        },
    })
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (a_range, b_range, x_range, ops) = match &params.worksheet {
        WorksheetType::AlgebraOneStep {
            a_range,
            b_range,
            x_range,
            add,
            subtract,
            multiply,
            divide,
            ..
        } => (
            *a_range,
            *b_range,
            *x_range,
            [*add, *subtract, *multiply, *divide],
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

    // Enumerate per-op pools separately so we can sample evenly across
    // enabled ops. The add pool grows much faster than mul/div as ranges
    // widen — a single-pool shuffle would let add dominate the page even
    // when all four toggles are on.
    let mut pools: Vec<Vec<(u32, u32, u32, u32)>> = (0..4).map(|_| Vec::new()).collect();

    if ops[0] {
        // x + b = c
        for b in b_range.min..=b_range.max {
            for x in x_range.min..=x_range.max {
                pools[0].push((0, b, x, x + b));
            }
        }
    }
    if ops[1] {
        // x - b = c, with x >= b so c stays non-negative
        for b in b_range.min..=b_range.max {
            let lo = x_range.min.max(b);
            if lo > x_range.max {
                continue;
            }
            for x in lo..=x_range.max {
                pools[1].push((1, b, x, x - b));
            }
        }
    }
    if ops[2] {
        // a · x = c
        for a in a_range.min..=a_range.max {
            for x in x_range.min..=x_range.max {
                pools[2].push((2, a, x, a * x));
            }
        }
    }
    if ops[3] {
        // x ÷ a = c, iterating x in multiples of a so c is a clean integer
        for a in a_range.min..=a_range.max {
            let lo = x_range.min;
            let hi = x_range.max;
            let start = lo.div_ceil(a) * a;
            let mut x = start;
            while x <= hi {
                pools[3].push((3, a, x, x / a));
                x = x.saturating_add(a);
                if x < a {
                    break;
                }
            }
        }
    }

    let total = params.total_problems() as usize;
    let enabled: Vec<usize> = (0..4).filter(|i| !pools[*i].is_empty()).collect();
    let mut all: Vec<(u32, u32, u32, u32)> = Vec::new();

    if !enabled.is_empty() && total > 0 {
        // Shuffle each enabled pool independently, then take an even
        // share. Round up so 10 problems across 4 ops gives 3+3+3+3 = 12
        // before the global truncate brings it back to 10 (and we still
        // get representation from every op).
        let per_op = total.div_ceil(enabled.len());
        for &i in &enabled {
            pools[i].shuffle(&mut rng);
            let take = per_op.min(pools[i].len());
            all.extend(pools[i][..take].iter().copied());
        }
        // If pools were smaller than per_op (narrow range + few ops),
        // top up by sampling more from whichever pools have leftover
        // capacity. Avoids a short page when one op is starved.
        if all.len() < total {
            for &i in &enabled {
                let already = all.iter().filter(|p| p.0 as usize == i).count();
                let mut extra = pools[i][already..].iter().copied().collect::<Vec<_>>();
                while all.len() < total && !extra.is_empty() {
                    all.push(extra.remove(0));
                }
                if all.len() >= total {
                    break;
                }
            }
        }
    }

    all.shuffle(&mut rng);
    if total > 0 && all.len() > total {
        all.truncate(total);
    }
    crate::pad_with_duplicates(&mut all, total, &mut rng);

    all.into_iter()
        .map(|(f, p, x, c)| vec![f, p, x, c])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DigitRange;

    fn test_params(
        a_range: DigitRange,
        b_range: DigitRange,
        x_range: DigitRange,
        add: bool,
        subtract: bool,
        multiply: bool,
        divide: bool,
    ) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::AlgebraOneStep {
                a_range,
                b_range,
                x_range,
                variable: "x".into(),
                add,
                subtract,
                multiply,
                divide,
            },
            num_problems: 12,
            cols: 2,
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

    #[test]
    fn invariants_per_form() {
        let params = test_params(
            DigitRange::new(2, 10),
            DigitRange::new(1, 30),
            DigitRange::new(0, 20),
            true,
            true,
            true,
            true,
        );
        let problems = generate_problems(&params);
        assert!(!problems.is_empty());
        for p in &problems {
            let (form, pp, x, c) = (p[0], p[1], p[2], p[3]);
            match form {
                0 => assert_eq!(x + pp, c, "add: x + b == c"),
                1 => assert_eq!(c + pp, x, "sub: x - b == c, i.e. x == c + b"),
                2 => assert_eq!(pp * x, c, "mul: a*x == c"),
                3 => {
                    assert_eq!(c * pp, x, "div: x/a == c, i.e. x == c*a");
                    assert_eq!(x % pp, 0, "div: x divisible by a");
                }
                _ => panic!("unexpected form {form}"),
            }
        }
    }

    #[test]
    fn add_only_yields_only_form_0() {
        let params = test_params(
            DigitRange::new(2, 5),
            DigitRange::new(1, 10),
            DigitRange::new(0, 10),
            true,
            false,
            false,
            false,
        );
        let problems = generate_problems(&params);
        assert!(!problems.is_empty());
        for p in &problems {
            assert_eq!(p[0], 0);
        }
    }

    #[test]
    fn subtraction_never_negative() {
        // Force a setup where subtraction would often go negative if
        // unchecked (small x range, large b range).
        let params = test_params(
            DigitRange::new(2, 3),
            DigitRange::new(20, 30),
            DigitRange::new(0, 5),
            false,
            true,
            false,
            false,
        );
        let problems = generate_problems(&params);
        // x_range max (5) < b_range min (20) → no valid subtraction triples.
        // Generator should bail rather than emit negatives. Padded duplicates
        // can't appear from an empty pool, so problems is empty.
        assert!(
            problems.is_empty(),
            "no subtraction triple should satisfy x >= b in this config"
        );
    }

    #[test]
    fn division_only_clean_integers() {
        let params = test_params(
            DigitRange::new(2, 9),
            DigitRange::new(1, 1),
            DigitRange::new(0, 50),
            false,
            false,
            false,
            true,
        );
        let problems = generate_problems(&params);
        assert!(!problems.is_empty());
        for p in &problems {
            let (form, a, x, c) = (p[0], p[1], p[2], p[3]);
            assert_eq!(form, 3);
            assert_eq!(x % a, 0);
            assert_eq!(x / a, c);
        }
    }

    #[test]
    fn no_ops_enabled_yields_empty() {
        let params = test_params(
            DigitRange::new(2, 5),
            DigitRange::new(1, 5),
            DigitRange::new(0, 5),
            false,
            false,
            false,
            false,
        );
        let problems = generate_problems(&params);
        assert!(problems.is_empty());
    }

    #[test]
    fn ranges_respected() {
        let a = DigitRange::new(3, 5);
        let b = DigitRange::new(2, 8);
        let x = DigitRange::new(0, 10);
        let params = test_params(a, b, x, true, true, true, true);
        let problems = generate_problems(&params);
        for p in &problems {
            let (form, pp, xv) = (p[0], p[1], p[2]);
            assert!(xv >= x.min && xv <= x.max);
            match form {
                0 | 1 => assert!(pp >= b.min && pp <= b.max),
                2 | 3 => assert!(pp >= a.min && pp <= a.max),
                _ => panic!("unexpected form {form}"),
            }
        }
    }

    #[test]
    fn pads_when_pool_smaller_than_target() {
        // Tiny pool: only mul, with a=2..2 and x=1..2 → 2 triples.
        // Asking for 12 should pad with duplicates rather than truncate.
        let mut params = test_params(
            DigitRange::new(2, 2),
            DigitRange::new(1, 1),
            DigitRange::new(1, 2),
            false,
            false,
            true,
            false,
        );
        params.num_problems = 12;
        let problems = generate_problems(&params);
        assert_eq!(problems.len(), 12);
    }
}

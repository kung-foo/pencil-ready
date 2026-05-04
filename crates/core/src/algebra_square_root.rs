//! Squares and square-roots worksheet — `x² ± b = c` and `√x ± b = c`.
//!
//! Six equation forms covering both families. The user toggles whole
//! families on/off via the `squares` and `roots` flags; at least one
//! must be enabled. Within each enabled family, all three form
//! variants (canonical, const-first, canonical-minus) always mix —
//! by the time a student is doing squares-and-roots they've already
//! met form variation in two-step.
//!
//! Problem representation as `Vec<u32>`: `[form, b, inner, answer, c]`
//!   form 0 → `x² + b = c`   (canonical plus, square family)
//!   form 1 → `b + x² = c`   (const-first plus, square family)
//!   form 2 → `x² − b = c`   (canonical minus, square; only when x² ≥ b)
//!   form 3 → `√x + b = c`   (canonical plus, root family)
//!   form 4 → `b + √x = c`   (const-first plus, root family)
//!   form 5 → `√x − b = c`   (canonical minus, root; only when r ≥ b)
//!
//! `inner` is the integer "inside" the operator on the LHS:
//!   - Square forms: inner = x (also equals the answer).
//!   - Root forms:   inner = √x = r, and answer = r².
//!
//! `answer` is always the value of x in the solution `x = ___` row,
//! so the typst component never has to know which family a problem
//! belongs to to render the final row.

use crate::document;
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

/// Inner-integer range (`x` for squares, `√x` for roots). Pinned at
/// 0..=10 because the curriculum target is "knows squares 0²–10² and
/// the matching roots by heart" — exposing a slider would invite
/// off-curriculum config.
///
/// `inner == 1` is skipped: `x² + b = c` and `√x + b = c` both
/// degenerate to a one-step `1 + b = c` problem, which doesn't
/// exercise the squaring / rooting step the worksheet is teaching.
const INNER_MIN: u32 = 0;
const INNER_MAX: u32 = 10;

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let problems = generate_problems(params);
    let variable = match &params.worksheet {
        WorksheetType::AlgebraSquareRoot { variable, .. } => variable.clone(),
        _ => unreachable!(),
    };
    // Same locale convention as two-step: `·` regardless of locale —
    // never `×` once a variable is on the page. Explicit --symbol still
    // overrides (here it would only show up via opts plumbing for
    // future use; the component itself doesn't render `·` today).
    let operator = params
        .symbol
        .clone()
        .unwrap_or_else(|| "sym.dot.op".to_string());
    let max_digits = document::max_digits(&problems);
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
            variable,
            decimal_places: Vec::new(),
            reserve_remainder: false,
        },
    })
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (b_range, squares, roots) = match &params.worksheet {
        WorksheetType::AlgebraSquareRoot {
            b_range,
            squares,
            roots,
            ..
        } => (*b_range, *squares, *roots),
        _ => unreachable!(),
    };

    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    // Enumerate (inner, b) pairs for each enabled family. For each pair
    // pick a random form within that family; the form-2 minus variant
    // is only emitted when the LHS-inner ≥ b (so c stays non-negative).
    let mut all: Vec<(u32, u32, u32, u32, u32)> = Vec::new();
    for inner in INNER_MIN..=INNER_MAX {
        if inner == 1 {
            continue;
        }
        for b in b_range.min..=b_range.max {
            if squares {
                let x_squared = inner * inner;
                let mut valid = vec![0u32, 1];
                if x_squared >= b {
                    valid.push(2);
                }
                let form = valid[rng.gen_range(0..valid.len())];
                let c = if form == 2 {
                    x_squared - b
                } else {
                    x_squared + b
                };
                let answer = inner;
                all.push((form, b, inner, answer, c));
            }
            if roots {
                let mut valid = vec![3u32, 4];
                if inner >= b {
                    valid.push(5);
                }
                let form = valid[rng.gen_range(0..valid.len())];
                let c = if form == 5 { inner - b } else { inner + b };
                let answer = inner * inner;
                all.push((form, b, inner, answer, c));
            }
        }
    }

    all.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    if total > 0 && all.len() > total {
        all.truncate(total);
    }

    all.into_iter()
        .map(|(form, b, inner, answer, c)| vec![form, b, inner, answer, c])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DigitRange;

    fn test_params(b_range: DigitRange, squares: bool, roots: bool) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::AlgebraSquareRoot {
                b_range,
                variable: "x".into(),
                squares,
                roots,
            },
            num_problems: 24,
            cols: 2,
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

    #[test]
    fn invariants_hold_for_all_forms() {
        let params = test_params(DigitRange::new(1, 30), true, true);
        let problems = generate_problems(&params);
        assert!(!problems.is_empty());
        for p in &problems {
            let (form, b, inner, answer, c) = (p[0], p[1], p[2], p[3], p[4]);
            match form {
                0 | 1 => {
                    assert_eq!(inner * inner + b, c, "square-plus: inner² + b == c");
                    assert_eq!(answer, inner, "square family: answer == inner");
                }
                2 => {
                    assert_eq!(inner * inner, c + b, "square-minus: inner² == c + b");
                    assert!(inner * inner >= b, "form 2 invariant: x² ≥ b");
                    assert_eq!(answer, inner);
                }
                3 | 4 => {
                    assert_eq!(inner + b, c, "root-plus: inner + b == c");
                    assert_eq!(answer, inner * inner, "root family: answer == inner²");
                }
                5 => {
                    assert_eq!(inner, c + b, "root-minus: inner == c + b");
                    assert!(inner >= b, "form 5 invariant: r ≥ b");
                    assert_eq!(answer, inner * inner);
                }
                other => panic!("unexpected form: {other}"),
            }
        }
    }

    #[test]
    fn squares_only_emits_only_square_forms() {
        let params = test_params(DigitRange::new(1, 10), true, false);
        let problems = generate_problems(&params);
        assert!(!problems.is_empty());
        for p in &problems {
            assert!(p[0] <= 2, "squares-only should yield forms 0/1/2, got {}", p[0]);
        }
    }

    #[test]
    fn roots_only_emits_only_root_forms() {
        let params = test_params(DigitRange::new(1, 10), false, true);
        let problems = generate_problems(&params);
        assert!(!problems.is_empty());
        for p in &problems {
            assert!(p[0] >= 3, "roots-only should yield forms 3/4/5, got {}", p[0]);
        }
    }

    #[test]
    fn inner_range_respected() {
        let params = test_params(DigitRange::new(1, 10), true, true);
        let problems = generate_problems(&params);
        for p in &problems {
            assert!(p[2] >= INNER_MIN && p[2] <= INNER_MAX);
            assert_ne!(p[2], 1, "inner == 1 is skipped");
        }
    }

    #[test]
    fn minus_form_never_negative() {
        // Setup where form 2 / form 5 would often go negative if unchecked
        // (small inner, large b).
        let params = test_params(DigitRange::new(20, 30), true, true);
        let problems = generate_problems(&params);
        for p in &problems {
            let (form, b, inner) = (p[0], p[1], p[2]);
            if form == 2 {
                assert!(inner * inner >= b, "form 2: x² < b emitted");
            }
            if form == 5 {
                assert!(inner >= b, "form 5: r < b emitted");
            }
        }
    }
}

//! Fraction addition and subtraction — `a/b ± c/d`, answer in simplest
//! form (reduced; improper results as mixed numbers).
//!
//! One generator for both kinds. Operands are proper fractions in lowest
//! terms. `relation` picks how the two denominators relate, which is
//! what sets the difficulty:
//!   - same:      3/8 + 1/8 — combine the numerators
//!   - multiple:  1/2 + 1/4 — rewrite one fraction
//!   - unrelated: 2/3 + 3/4 — rewrite both over a new common denominator
//!
//! Subtraction keeps the first fraction strictly larger, so answers are
//! positive. The worked steps (common denominator, combine, simplify)
//! are derived by the typst component; the generator supplies the final
//! answer, which the component asserts against.

use crate::rng::{Rng, SliceShuffle};
use crate::document;
use crate::{ComponentOpts, DenominatorRelation, Sheet, WorksheetParams, WorksheetType};

/// Largest common denominator a problem may need. Keeps the rewritten
/// fractions, and so the cell width, small: 1/9 + 1/4 → 36ths is the
/// widest allowed.
pub(crate) const MAX_COMMON_DENOMINATOR: u32 = 36;

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (denominators, relation, subtract) = terms(&params.worksheet);
    let problems = generate_problems(params, denominators, relation, subtract);
    let max_digits = document::max_digits(&problems);
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            // + and − read the same in every locale.
            operator: if subtract { "sym.minus" } else { "sym.plus" }.to_string(),
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

pub(crate) fn terms(ws: &WorksheetType) -> (&[u32], DenominatorRelation, bool) {
    match ws {
        WorksheetType::FractionAdd {
            denominators,
            relation,
        } => (denominators, *relation, false),
        WorksheetType::FractionSubtract {
            denominators,
            relation,
        } => (denominators, *relation, true),
        _ => unreachable!(),
    }
}

/// Every valid problem `(a, b, c, d)` for the configuration, in a fixed
/// order. Validation calls this too, to reject configurations with no
/// problems at all.
pub(crate) fn candidates(
    denominators: &[u32],
    relation: DenominatorRelation,
    subtract: bool,
) -> Vec<[u32; 4]> {
    let mut dens = denominators.to_vec();
    dens.sort_unstable();
    dens.dedup();
    let mut out = Vec::new();
    for &b in &dens {
        for &d in &dens {
            if !relation.allows(b, d) || lcm(b, d) > MAX_COMMON_DENOMINATOR {
                continue;
            }
            for a in (1..b).filter(|&a| gcd(a, b) == 1) {
                for c in (1..d).filter(|&c| gcd(c, d) == 1) {
                    // a/b > c/d, so the difference is positive.
                    if subtract && a * d <= c * b {
                        continue;
                    }
                    out.push([a, b, c, d]);
                }
            }
        }
    }
    out
}

/// `a/b ± c/d` in simplest form, as `(whole, num, den)`: `num == 0` is a
/// whole number (den 1), `whole == 0` a proper fraction.
pub(crate) fn answer([a, b, c, d]: [u32; 4], subtract: bool) -> (u32, u32, u32) {
    let (x, y) = (a * d, c * b);
    let num = if subtract { x - y } else { x + y };
    let den = b * d;
    let g = gcd(num, den);
    let (num, den) = (num / g, den / g);
    if num % den == 0 {
        (num / den, 0, 1)
    } else {
        (num / den, num % den, den)
    }
}

fn generate_problems(
    params: &WorksheetParams,
    denominators: &[u32],
    relation: DenominatorRelation,
    subtract: bool,
) -> Vec<Vec<u32>> {
    let mut rng = Rng::from_seed(params.seed);
    let mut pool = candidates(denominators, relation, subtract);
    pool.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    if total > 0 && pool.len() > total {
        pool.truncate(total);
    }
    crate::pad_with_duplicates(&mut pool, total, &mut rng);
    pool.into_iter()
        .map(|p| {
            let (w, n, r) = answer(p, subtract);
            vec![p[0], p[1], p[2], p[3], w, n, r]
        })
        .collect()
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn lcm(a: u32, b: u32) -> u32 {
    a / gcd(a, b) * b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, Paper};

    const DENS: &[u32] = &[2, 3, 4, 5, 6, 8, 10, 12];

    fn params(subtract: bool, relation: DenominatorRelation, n: u32) -> WorksheetParams {
        let denominators = DENS.to_vec();
        WorksheetParams {
            worksheet: if subtract {
                WorksheetType::FractionSubtract { denominators, relation }
            } else {
                WorksheetType::FractionAdd { denominators, relation }
            },
            num_problems: n,
            cols: 2,
            paper: Paper::default(),
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Locale::Us,
            solve_first: false,
            include_answers: false,
            student_name: None,
            instructions: None,
            share_url: None,
        }
    }

    #[test]
    fn answers_are_in_simplest_form() {
        assert_eq!(answer([1, 2, 1, 3], false), (0, 5, 6));
        assert_eq!(answer([2, 3, 3, 4], false), (1, 5, 12));
        assert_eq!(answer([1, 2, 1, 2], false), (1, 0, 1));
        assert_eq!(answer([5, 6, 1, 3], true), (0, 1, 2));
        assert_eq!(answer([3, 4, 1, 4], true), (0, 1, 2));
    }

    #[test]
    fn relations_constrain_denominators() {
        use DenominatorRelation::*;
        for (rel, ok) in [
            (Same, (|b, d| b == d) as fn(u32, u32) -> bool),
            (Multiple, |b, d| b != d && (b % d == 0 || d % b == 0)),
            (Unrelated, |b, d| b % d != 0 && d % b != 0),
        ] {
            let pool = candidates(DENS, rel, false);
            assert!(!pool.is_empty(), "{rel:?}");
            for [_, b, _, d] in pool {
                assert!(ok(b, d), "{rel:?}: {b}, {d}");
                assert!(lcm(b, d) <= MAX_COMMON_DENOMINATOR);
            }
        }
    }

    #[test]
    fn subtraction_is_always_positive() {
        for rel in [
            DenominatorRelation::Same,
            DenominatorRelation::Multiple,
            DenominatorRelation::Unrelated,
            DenominatorRelation::Any,
        ] {
            for [a, b, c, d] in candidates(DENS, rel, true) {
                assert!(a * d > c * b, "{a}/{b} − {c}/{d}");
            }
        }
    }

    #[test]
    fn operands_are_proper_and_reduced() {
        for [a, b, c, d] in candidates(DENS, DenominatorRelation::Any, false) {
            assert!(a < b && c < d);
            assert_eq!(gcd(a, b), 1);
            assert_eq!(gcd(c, d), 1);
        }
    }

    #[test]
    fn fills_the_page_without_repeats() {
        for subtract in [false, true] {
            let probs = generate_problems(
                &params(subtract, DenominatorRelation::Same, 12),
                DENS,
                DenominatorRelation::Same,
                subtract,
            );
            assert_eq!(probs.len(), 12);
            let mut uniq = probs.clone();
            uniq.sort();
            uniq.dedup();
            assert_eq!(uniq.len(), 12, "subtract={subtract}");
        }
    }
}

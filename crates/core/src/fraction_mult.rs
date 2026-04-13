//! Fraction multiplication worksheet — `whole × num/den = ___`
//!
//! Always produces whole-integer answers (no remainders).

use crate::template;
use crate::{WorksheetParams, WorksheetType};

pub fn generate_typ(params: &WorksheetParams) -> anyhow::Result<String> {
    let problems = generate_problems(params);
    let solve_first = match &params.worksheet {
        WorksheetType::FractionMultiply { solve_first, .. } => *solve_first,
        _ => unreachable!(),
    };
    // Locale-aware multiply symbol (× for US, · for Norway).
    let symbol = params.locale.multiply_symbol();
    template::render_horizontal_fraction(symbol, &problems, params, solve_first)
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (denominators, min_whole, max_whole, unit_only) = match &params.worksheet {
        WorksheetType::FractionMultiply {
            denominators,
            min_whole,
            max_whole,
            unit_only,
            ..
        } => (denominators, *min_whole, *max_whole, *unit_only),
        _ => unreachable!(),
    };

    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    // Enumerate every (whole, numerator, denominator) triple where the answer
    // is a whole integer. For whole × num/den to be integer, whole must be
    // divisible by den/gcd(num, den). Simplest construction: pick den, pick
    // num, then pick whole as a multiple of (den / gcd(num, den)).
    let mut all: Vec<(u32, u32, u32)> = Vec::new();
    for &den in denominators {
        let num_range: Vec<u32> = if unit_only {
            vec![1]
        } else {
            (1..den).collect()
        };
        for num in num_range {
            let stride = den / gcd(num, den);
            // smallest multiple of stride that is >= min_whole
            let start = ((min_whole + stride - 1) / stride) * stride;
            let mut w = start;
            while w <= max_whole {
                all.push((w, num, den));
                w += stride;
            }
        }
    }

    // Shuffle then truncate.
    all.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    if total > 0 && all.len() > total {
        all.truncate(total);
    }

    // If too few were possible, just return what we have — the template
    // will catch under-count and bail with a clear error.
    all.into_iter()
        .map(|(w, n, d)| vec![w, n, d])
        .collect()
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_only_halves() {
        let params = test_params(vec![2], 2, 20, true);
        let problems = generate_problems(&params);
        for p in &problems {
            let (w, n, d) = (p[0], p[1], p[2]);
            assert_eq!(n, 1);
            assert_eq!(d, 2);
            assert_eq!((w * n) % d, 0, "{w} × {n}/{d} not integer");
        }
    }

    #[test]
    fn test_proper_fraction_clean() {
        let params = test_params(vec![2, 3, 4, 5], 2, 30, false);
        let problems = generate_problems(&params);
        for p in &problems {
            let (w, n, d) = (p[0], p[1], p[2]);
            assert!(w >= 2 && w <= 30);
            assert!(n >= 1 && n < d);
            assert_eq!((w * n) % d, 0, "{w} × {n}/{d} not integer");
        }
    }

    fn test_params(
        denominators: Vec<u32>,
        min_whole: u32,
        max_whole: u32,
        unit_only: bool,
    ) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::FractionMultiply {
                denominators,
                min_whole,
                max_whole,
                unit_only,
                solve_first: false,
            },
            num_problems: 12,
            cols: 2,
            font: "B612".into(),
            paper: "a4".into(),
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Default::default(),
            pages: 1,
        }
    }
}

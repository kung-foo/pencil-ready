//! Fraction ↔ decimal ↔ percent: one value written three ways
//! (`3/4 = 0.75 = 75%`), with one form given and two to fill in.
//!
//! Denominators are limited to divisors of 1000, so every decimal
//! terminates within three places. `improper` adds values between 1 and
//! 2, written as mixed numbers, for denominators up to 10 only, so they
//! stay a minority on the page.
//!
//! Decimal and percent go to the component as scaled integers
//! (`0.125` = 125 with 3 places), so typst never formats a float.

use crate::rng::{Rng, SliceShuffle};
use crate::{ComponentOpts, ConversionGiven, Sheet, WorksheetParams, WorksheetType};

/// Denominators whose fractions terminate within three decimal places
/// and stay readable on a worksheet.
pub(crate) const ALLOWED_DENOMINATORS: &[u32] = &[2, 4, 5, 8, 10, 20, 25, 40, 50, 100];

/// Values with `improper` set stay below this whole number.
const MAX_WHOLE: u32 = 2;

/// Values above 1 only use denominators up to this (`1 3/8`, not
/// `1 37/50`): the mixed numbers are for recognising, not for drilling.
const MAX_IMPROPER_DENOMINATOR: u32 = 10;

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems: generate_problems(params),
        opts: ComponentOpts {
            operator: String::new(),
            divide_operator: String::new(),
            width_cm: 0.0,
            answer_rows: 0,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            decimal_places: Vec::new(),
            reserve_remainder: false,
        },
    })
}

fn terms(ws: &WorksheetType) -> (&[u32], bool, ConversionGiven) {
    match ws {
        WorksheetType::FractionDecimalPercent {
            denominators,
            improper,
            given,
        } => (denominators, *improper, *given),
        _ => unreachable!(),
    }
}

/// Every value `(num, den)` in lowest terms for the configuration, in a
/// fixed order. Whole numbers are excluded (nothing to convert).
pub(crate) fn candidates(denominators: &[u32], improper: bool) -> Vec<(u32, u32)> {
    let mut dens = denominators.to_vec();
    dens.sort_unstable();
    dens.dedup();
    let mut out = Vec::new();
    for d in dens {
        let max = if improper && d <= MAX_IMPROPER_DENOMINATOR {
            MAX_WHOLE * d
        } else {
            d
        };
        out.extend((1..max).filter(|&n| gcd(n, d) == 1).map(|n| (n, d)));
    }
    out
}

/// `num/den` scaled to the fewest decimal places that make it a whole
/// number: `(value, places)`. `den` divides 1000, so places ≤ 3.
fn scaled(num: u32, den: u32) -> (u32, u32) {
    let mut places = 0;
    let mut unit = 1;
    while (num * unit) % den != 0 {
        places += 1;
        unit *= 10;
    }
    (num * unit / den, places)
}

/// One problem's data: `(num, den, given, dec, dec_places, pct, pct_places)`.
pub(crate) fn problem(num: u32, den: u32, given: u32) -> Vec<u32> {
    let (dec, dp) = scaled(num, den);
    let (pct, pp) = scaled(num * 100, den);
    vec![num, den, given, dec, dp, pct, pp]
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (denominators, improper, given) = terms(&params.worksheet);
    let mut rng = Rng::from_seed(params.seed);
    let mut pool = candidates(denominators, improper);
    pool.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    if total > 0 && pool.len() > total {
        pool.truncate(total);
    }
    crate::pad_with_duplicates(&mut pool, total, &mut rng);

    // With `given=any`, each form is printed on a third of the page,
    // in shuffled order.
    let mut givens: Vec<u32> = match given {
        ConversionGiven::Fraction => vec![0; pool.len()],
        ConversionGiven::Decimal => vec![1; pool.len()],
        ConversionGiven::Percent => vec![2; pool.len()],
        ConversionGiven::Any => (0..pool.len() as u32).map(|i| i % 3).collect(),
    };
    givens.shuffle(&mut rng);
    pool.into_iter()
        .zip(givens)
        .map(|((n, d), g)| problem(n, d, g))
        .collect()
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms_agree() {
        assert_eq!(problem(3, 4, 0), vec![3, 4, 0, 75, 2, 75, 0]);
        assert_eq!(problem(1, 8, 1), vec![1, 8, 1, 125, 3, 125, 1]);
        assert_eq!(problem(5, 4, 2), vec![5, 4, 2, 125, 2, 125, 0]);
        assert_eq!(problem(1, 40, 0), vec![1, 40, 0, 25, 3, 25, 1]);
        assert_eq!(problem(7, 100, 0), vec![7, 100, 0, 7, 2, 7, 0]);
    }

    #[test]
    fn every_allowed_denominator_terminates_within_three_places() {
        for (n, d) in candidates(ALLOWED_DENOMINATORS, true) {
            let p = problem(n, d, 0);
            assert!(p[4] <= 3 && p[6] <= 1, "{n}/{d}: {p:?}");
            assert!(n < MAX_WHOLE * d);
            assert_eq!(gcd(n, d), 1);
        }
    }

    #[test]
    fn proper_only_unless_improper() {
        assert!(candidates(&[2, 4, 5], false).iter().all(|&(n, d)| n < d));
        assert!(candidates(&[2, 4, 5], true).iter().any(|&(n, d)| n > d));
        // Values above 1 stay below 2, with small denominators only.
        for (n, d) in candidates(ALLOWED_DENOMINATORS, true) {
            assert!(n < d || (n < 2 * d && d <= MAX_IMPROPER_DENOMINATOR), "{n}/{d}");
        }
    }

    #[test]
    fn level_pools_fill_a_page() {
        // Tenths and hundredths in lowest terms: 4 + 40.
        assert_eq!(candidates(&[10, 100], false).len(), 44);
        assert!(candidates(&[2, 4, 5, 10, 20, 25, 50], false).len() >= 20);
    }
}

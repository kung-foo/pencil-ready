//! Factor pairs — list every factor of a number as pairs (`1 × 24`,
//! `2 × 12`, …), trying 1, 2, 3, … until the pairs meet.
//!
//! Numbers come from the top two thirds of the range up to `max_number`
//! (10–30 for `max_number` 30), so each level practises its own size of
//! number. Primes (one pair, `1 × p`) only with `primes`.
//!
//! The cell reserves a row per pair for the number in the range with the
//! most pairs, so every cell is the same size and the blank rows don't
//! give away how many pairs a number has.

use crate::rng::{Rng, SliceShuffle};
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

/// Smallest number on any sheet.
pub(crate) const MIN_NUMBER: u32 = 4;

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (max_number, primes) = match &params.worksheet {
        WorksheetType::FactorPairs { max_number, primes, .. } => (*max_number, *primes),
        _ => unreachable!(),
    };
    let mut rng = Rng::from_seed(params.seed);
    let mut pool = candidates(max_number, primes);
    pool.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    pool.truncate(total);
    crate::pad_with_duplicates(&mut pool, total, &mut rng);
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems: pool.into_iter().map(|n| vec![n]).collect(),
        opts: ComponentOpts {
            operator: params.locale.multiply_symbol().to_string(),
            divide_operator: String::new(),
            width_cm: 0.0,
            answer_rows: rows(max_number, primes),
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            decimal_places: Vec::new(),
            reserve_remainder: false,
        },
    })
}

pub(crate) fn is_prime(n: u32) -> bool {
    n > 1 && (2..n).take_while(|d| d * d <= n).all(|d| n % d != 0)
}

/// Smallest number for a range up to `max_number`: its top two thirds.
pub(crate) fn min_number(max_number: u32) -> u32 {
    (max_number / 3).max(MIN_NUMBER)
}

/// The numbers a sheet draws from, ascending.
pub(crate) fn candidates(max_number: u32, primes: bool) -> Vec<u32> {
    (min_number(max_number)..=max_number)
        .filter(|&n| primes || !is_prime(n))
        .collect()
}

fn pair_count(n: u32) -> u32 {
    (1..=n).take_while(|d| d * d <= n).filter(|d| n % d == 0).count() as u32
}

/// Pair rows each cell reserves: the most pairs any candidate has.
pub(crate) fn rows(max_number: u32, primes: bool) -> u32 {
    candidates(max_number, primes).into_iter().map(pair_count).max().unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_cover_the_range() {
        assert_eq!(pair_count(24), 4);
        assert_eq!(pair_count(36), 5);
        assert_eq!(pair_count(37), 1);
        assert_eq!(rows(30, false), 4);
        assert_eq!(rows(100, true), 6);
        assert_eq!(candidates(30, false).first(), Some(&10));
        assert!(!candidates(30, false).contains(&11));
        assert!(candidates(30, true).contains(&11));
    }
}

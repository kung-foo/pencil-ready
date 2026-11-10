//! Prime factorization with a factor tree — `24 = 2 · 2 · 2 · 3`.
//!
//! Composite numbers from the top two thirds of the range up to
//! `max_number` (at most 100, so the biggest tree, 64's six leaves,
//! fits the cell). The typst component builds the tree and the answer.

use crate::factor_pairs::{is_prime, min_number};
use crate::rng::{Rng, SliceShuffle};
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let max_number = match &params.worksheet {
        WorksheetType::PrimeFactors { max_number } => *max_number,
        _ => unreachable!(),
    };
    let mut rng = Rng::from_seed(params.seed);
    let mut pool = candidates(max_number);
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
            answer_rows: 0,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            decimal_places: Vec::new(),
            reserve_remainder: false,
        },
    })
}

/// Composite numbers in the range, ascending.
pub(crate) fn candidates(max_number: u32) -> Vec<u32> {
    (min_number(max_number)..=max_number).filter(|&n| !is_prime(n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_composites() {
        let c = candidates(30);
        assert_eq!(c.first(), Some(&10));
        assert!(c.iter().all(|&n| !is_prime(n)));
        assert!(c.contains(&30) && !c.contains(&29));
    }
}

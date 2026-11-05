//! Greatest common factor — `GCF(12, 18) = ___` (`SFF` in Norwegian),
//! with room to list each number's factors.
//!
//! Practice for simplifying fractions, so a pair reads like a fraction:
//! the smaller number first, both from 4 to `max_number`. By default
//! every pair shares a factor other than 1; with `coprime`, pairs with
//! GCF 1 (a fraction already in simplest form) are mixed in.
//!
//! Pairs are dealt round-robin across GCF values, so the answers vary
//! instead of being mostly 2 (the most common GCF by far). GCFs stop at
//! 12, within the times tables: above that a pair is almost always a
//! number and its double (`GCF(37, 74)`), which teaches nothing.

use crate::rng::{Rng, SliceShuffle};
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

/// Smallest number. Below 4 the factor lists are too short to practise on.
pub(crate) const MIN_NUMBER: u32 = 4;

/// Largest GCF a pair can have.
pub(crate) const MAX_GCF: u32 = 12;

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

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Every pair `(a, b)` with `a < b` for the configuration, grouped by
/// GCF (ascending), each group in a fixed order.
pub(crate) fn candidates(max_number: u32, coprime: bool) -> Vec<Vec<(u32, u32)>> {
    let mut by_gcd: Vec<Vec<(u32, u32)>> = vec![Vec::new(); max_number as usize + 1];
    for b in MIN_NUMBER..=max_number {
        for a in MIN_NUMBER..b {
            let g = gcd(a, b);
            if (g > 1 || coprime) && g <= MAX_GCF {
                by_gcd[g as usize].push((a, b));
            }
        }
    }
    by_gcd.retain(|g| !g.is_empty());
    by_gcd
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (max_number, coprime) = match &params.worksheet {
        WorksheetType::Gcf { max_number, coprime, .. } => (*max_number, *coprime),
        _ => unreachable!(),
    };
    let mut rng = Rng::from_seed(params.seed);
    let mut by_gcd = candidates(max_number, coprime);
    for g in &mut by_gcd {
        g.shuffle(&mut rng);
    }
    // Deal from the GCF values in random order: with more values than
    // problems, sorted order would only ever reach the smallest.
    by_gcd.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    let mut pool = Vec::new();
    let mut i = 0;
    while pool.len() < total && by_gcd.iter().any(|g| i < g.len()) {
        for g in &by_gcd {
            if let Some(&c) = g.get(i) {
                if pool.len() < total {
                    pool.push(c);
                }
            }
        }
        i += 1;
    }
    pool.shuffle(&mut rng);
    crate::pad_with_duplicates(&mut pool, total, &mut rng);
    pool.into_iter().map(|(a, b)| vec![a, b, gcd(a, b)]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, Paper};

    fn params(max_number: u32, coprime: bool, n: u32) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::Gcf { max_number, coprime, slots: false },
            num_problems: n,
            cols: 1,
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
    fn pairs_share_a_factor_unless_coprime() {
        for p in generate_problems(&params(30, false, 20)) {
            let (a, b, g) = (p[0], p[1], p[2]);
            assert!(MIN_NUMBER <= a && a < b && b <= 30, "{p:?}");
            assert_eq!(g, gcd(a, b));
            assert!(1 < g && g <= MAX_GCF, "{p:?}");
        }
        assert!(candidates(30, true).iter().flatten().any(|&(a, b)| gcd(a, b) == 1));
    }

    #[test]
    fn answers_vary() {
        let probs = generate_problems(&params(100, false, 6));
        let mut gcds: Vec<u32> = probs.iter().map(|p| p[2]).collect();
        gcds.sort();
        gcds.dedup();
        assert_eq!(gcds.len(), 6, "one problem per GCF: {probs:?}");
    }
}

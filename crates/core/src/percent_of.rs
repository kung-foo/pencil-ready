//! Percent of a number — `20% of 350 = ___`.
//!
//! By default only pairs with a whole-number answer: `percent × base`
//! must be a multiple of 100, so the easy percents (10 %, 50 %) get many
//! bases and the awkward ones (15 %, 35 %) only the bases that work out.
//! With `decimals`, any pair is allowed and the answer has up to two
//! decimal places (`37% of 48 = 17.76`). Bases run from 10 to `max_base`.
//!
//! The answer goes to typst as a scaled integer plus its decimal places
//! (17.76 = 1776, 2), so typst never formats a float.

use crate::rng::{Rng, SliceShuffle};
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

/// Smallest base. Below 10, 10 % and 20 % have almost nothing to work with.
pub(crate) const MIN_BASE: u32 = 10;

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems: generate_problems(params),
        // The working (a decimal column multiplication) uses the
        // locale's multiply sign: × or · in Norwegian.
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

/// Every `(percent, base)` for the configuration, in a fixed order:
/// all pairs with `decimals`, otherwise only whole-number answers.
pub(crate) fn candidates(percents: &[u32], max_base: u32, decimals: bool) -> Vec<(u32, u32)> {
    let mut ps = percents.to_vec();
    ps.sort_unstable();
    ps.dedup();
    let mut out = Vec::new();
    for p in ps {
        out.extend(
            (MIN_BASE..=max_base)
                .filter(|b| decimals || p * b % 100 == 0)
                .map(|b| (p, b)),
        );
    }
    out
}

/// One problem's data: `(percent, base, answer, places)`, the answer
/// scaled by 10^places with trailing zeros dropped (58% of 340 =
/// 197.2 → 1972, 1).
pub(crate) fn problem(p: u32, b: u32) -> Vec<u32> {
    let (mut answer, mut places) = (p * b, 2);
    while places > 0 && answer % 10 == 0 {
        answer /= 10;
        places -= 1;
    }
    vec![p, b, answer, places]
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (percents, max_base, decimals) = match &params.worksheet {
        WorksheetType::PercentOf {
            percents,
            max_base,
            decimals,
        } => (percents, *max_base, *decimals),
        _ => unreachable!(),
    };
    let mut rng = Rng::from_seed(params.seed);
    // Shuffle per percent, then deal round-robin, so a percent with many
    // valid bases (50 %) doesn't crowd out one with few (35 %).
    let mut by_pct: Vec<Vec<(u32, u32)>> = Vec::new();
    for c in candidates(percents, max_base, decimals) {
        match by_pct.last_mut() {
            Some(g) if g[0].0 == c.0 => g.push(c),
            _ => by_pct.push(vec![c]),
        }
    }
    for g in &mut by_pct {
        g.shuffle(&mut rng);
    }
    // Deal from the percents in random order: with more percents than
    // problems (`1-99`), sorted order would only ever reach the smallest.
    by_pct.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    let mut pool = Vec::new();
    let mut i = 0;
    while pool.len() < total && by_pct.iter().any(|g| i < g.len()) {
        for g in &by_pct {
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
    pool.into_iter().map(|(p, b)| problem(p, b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, Paper};

    fn params(percents: Vec<u32>, max_base: u32, n: u32) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::PercentOf {
                percents,
                max_base,
                decimals: false,
            },
            num_problems: n,
            cols: 3,
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
    fn answers_are_whole_numbers() {
        for (p, b) in candidates(&[5, 15, 25, 35, 75], 1000, false) {
            assert_eq!(p * b % 100, 0, "{p}% of {b}");
            assert!(b >= MIN_BASE);
            assert_eq!(problem(p, b)[3], 0);
        }
    }

    #[test]
    fn decimal_answers_drop_trailing_zeros() {
        assert_eq!(problem(37, 48), vec![37, 48, 1776, 2]);
        assert_eq!(problem(58, 340), vec![58, 340, 1972, 1]);
        assert_eq!(problem(25, 84), vec![25, 84, 21, 0]);
        assert_eq!(candidates(&[37], 99, true).len(), 90);
    }

    #[test]
    fn many_percents_are_sampled_across_the_range() {
        let probs = generate_problems(&params((1..=99).collect(), 1000, 8));
        assert_eq!(probs.len(), 8);
        // Not just 1%, 2%, … 8%.
        assert!(probs.iter().any(|p| p[0] > 20), "{probs:?}");
        let mut pcts: Vec<u32> = probs.iter().map(|p| p[0]).collect();
        pcts.sort();
        pcts.dedup();
        assert_eq!(pcts.len(), 8, "one problem per percent: {probs:?}");
    }

    #[test]
    fn every_percent_appears() {
        let probs = generate_problems(&params(vec![35, 50], 400, 12));
        assert_eq!(probs.len(), 12);
        assert!(probs.iter().any(|p| p[0] == 35));
        assert!(probs.iter().any(|p| p[0] == 50));
        for p in &probs {
            assert_eq!(p[0] * p[1], p[2] * 100);
            assert_eq!(p[3], 0);
        }
    }
}

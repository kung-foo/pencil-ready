//! Percent of a number — `20% of 350 = ___`.
//!
//! Only pairs with a whole-number answer: `percent × base` must be a
//! multiple of 100. Bases run from 10 up to `max_base`, so the easy
//! percents (10 %, 50 %) get many bases and the awkward ones (15 %,
//! 35 %) only the bases that work out.

use crate::rng::{Rng, SliceShuffle};
use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

/// Smallest base. Below 10, 10 % and 20 % have almost nothing to work with.
pub(crate) const MIN_BASE: u32 = 10;

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

/// Every `(percent, base)` with a whole-number answer, in a fixed order.
pub(crate) fn candidates(percents: &[u32], max_base: u32) -> Vec<(u32, u32)> {
    let mut ps = percents.to_vec();
    ps.sort_unstable();
    ps.dedup();
    let mut out = Vec::new();
    for p in ps {
        out.extend((MIN_BASE..=max_base).filter(|b| p * b % 100 == 0).map(|b| (p, b)));
    }
    out
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (percents, max_base) = match &params.worksheet {
        WorksheetType::PercentOf { percents, max_base } => (percents, *max_base),
        _ => unreachable!(),
    };
    let mut rng = Rng::from_seed(params.seed);
    // Shuffle per percent, then deal round-robin, so a percent with many
    // valid bases (50 %) doesn't crowd out one with few (35 %).
    let mut by_pct: Vec<Vec<(u32, u32)>> = Vec::new();
    for c in candidates(percents, max_base) {
        match by_pct.last_mut() {
            Some(g) if g[0].0 == c.0 => g.push(c),
            _ => by_pct.push(vec![c]),
        }
    }
    for g in &mut by_pct {
        g.shuffle(&mut rng);
    }
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
    pool.into_iter().map(|(p, b)| vec![p, b, p * b / 100]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, Paper};

    fn params(percents: Vec<u32>, max_base: u32, n: u32) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::PercentOf { percents, max_base },
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
        for (p, b) in candidates(&[5, 15, 25, 35, 75], 1000) {
            assert_eq!(p * b % 100, 0, "{p}% of {b}");
            assert!(b >= MIN_BASE);
        }
    }

    #[test]
    fn every_percent_appears() {
        let probs = generate_problems(&params(vec![35, 50], 400, 12));
        assert_eq!(probs.len(), 12);
        assert!(probs.iter().any(|p| p[0] == 35));
        assert!(probs.iter().any(|p| p[0] == 50));
        for p in &probs {
            assert_eq!(p[0] * p[1], p[2] * 100);
        }
    }
}

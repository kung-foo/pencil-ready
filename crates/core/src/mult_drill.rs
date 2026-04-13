//! Multiplication drill — horizontal times-table recall.

use crate::template;
use crate::{DigitRange, WorksheetParams, WorksheetType};

pub fn generate_typ(params: &WorksheetParams) -> anyhow::Result<String> {
    let (multiplicand, multiplier) = match &params.worksheet {
        WorksheetType::MultiplicationDrill { multiplicand, multiplier } => {
            (multiplicand, *multiplier)
        }
        _ => unreachable!(),
    };

    let problems = generate_problems(params, multiplicand, multiplier);
    // Locale determines the default symbol; --symbol overrides it.
    let default_symbol = params.locale.multiply_symbol();
    template::render_horizontal(default_symbol, &problems, params)
}

fn generate_problems(
    params: &WorksheetParams,
    multiplicand_ranges: &[DigitRange],
    multiplier_range: DigitRange,
) -> Vec<Vec<u32>> {
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::SeedableRng;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    // Expand multiplicand ranges into a flat set of values.
    let multiplicands: Vec<u32> = multiplicand_ranges
        .iter()
        .flat_map(|r| r.min..=r.max)
        .collect();

    let multipliers: Vec<u32> = (multiplier_range.min..=multiplier_range.max).collect();

    // Generate all pairs.
    let mut pairs: Vec<(u32, u32)> = Vec::new();
    for &m in &multiplicands {
        for &n in &multipliers {
            pairs.push((m, n));
        }
    }

    // Deduplicate commutative pairs: keep (min, max) form only.
    // e.g. if both (2,7) and (7,2) exist, keep only (2,7).
    pairs.sort();
    pairs.dedup();
    let mut seen = std::collections::HashSet::new();
    pairs.retain(|&(a, b)| {
        let key = if a <= b { (a, b) } else { (b, a) };
        seen.insert(key)
    });

    // Shuffle.
    pairs.shuffle(&mut rng);

    // 0 = all problems (use the full enumerated set).
    if params.num_problems > 0 {
        pairs.truncate(params.total_problems() as usize);
    }

    pairs.iter().map(|&(a, b)| vec![a, b]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twos_table() {
        let params = test_params(vec![DigitRange::fixed(2)], DigitRange::new(1, 10));
        let problems = generate_problems(&params, &[DigitRange::fixed(2)], DigitRange::new(1, 10));
        // 2×1 through 2×10 = 10 problems
        assert_eq!(problems.len(), 10);
        for nums in &problems {
            assert!(nums[0] == 2 || nums[1] == 2, "{:?} doesn't involve 2", nums);
        }
    }

    #[test]
    fn test_commutative_dedup() {
        // Tables 2 and 3, multiplier 1-10.
        // 2×3 and 3×2 should only appear once.
        let mcands = vec![DigitRange::fixed(2), DigitRange::fixed(3)];
        let mplier = DigitRange::new(1, 10);
        let params = test_params(mcands.clone(), mplier);
        let problems = generate_problems(&params, &mcands, mplier);

        let mut seen = std::collections::HashSet::new();
        for nums in &problems {
            let key = if nums[0] <= nums[1] {
                (nums[0], nums[1])
            } else {
                (nums[1], nums[0])
            };
            assert!(seen.insert(key), "duplicate commutative pair: {:?}", nums);
        }
    }

    fn test_params(multiplicand: Vec<DigitRange>, multiplier: DigitRange) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::MultiplicationDrill { multiplicand, multiplier },
            num_problems: 40,
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

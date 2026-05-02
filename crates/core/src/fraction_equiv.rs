//! Equivalent fractions worksheet — find the missing number in a/b = c/d.
//!
//! The base fraction is always in lowest terms. A scale factor k turns it
//! into an equivalent fraction. One of the four slots is left blank.

use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let problems = generate_problems(params);
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            operator: String::new(),
            divide_operator: String::new(),
            width_cm: 5.5,
            answer_rows: 0,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            decimal_places: Vec::new(),
            reserve_remainder: false,
        },
    })
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (denominators, scale, missing, proper_only) = match &params.worksheet {
        WorksheetType::FractionEquiv {
            denominators,
            scale,
            missing,
            proper_only,
        } => (denominators, *scale, missing, *proper_only),
        _ => unreachable!(),
    };

    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rand::seq::SliceRandom;
    use rand::Rng;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    // Enumerate all (base_num, base_den, scale_k) triples.
    // Base fraction is always in lowest terms (gcd == 1).
    let mut all: Vec<(u32, u32, u32)> = Vec::new();
    for &den in denominators {
        let num_max = if proper_only { den - 1 } else { den * scale.max };
        for num in 1..=num_max {
            if proper_only && num >= den {
                continue;
            }
            if gcd(num, den) != 1 {
                continue;
            }
            for k in scale.min..=scale.max {
                all.push((num, den, k));
            }
        }
    }

    all.shuffle(&mut rng);
    let total = params.total_problems() as usize;
    if total > 0 && all.len() > total {
        all.truncate(total);
    }
    crate::pad_with_duplicates(&mut all, total, &mut rng);

    all.into_iter()
        .map(|(base_num, base_den, k)| {
            let scaled_num = base_num * k;
            let scaled_den = base_den * k;

            // Randomly flip which side is the base fraction.
            let flip = rng.gen_bool(0.5);
            let (ln, ld, rn, rd) = if flip {
                (scaled_num, scaled_den, base_num, base_den)
            } else {
                (base_num, base_den, scaled_num, scaled_den)
            };

            // Pick which slot is blank: either fixed by param or random.
            let missing_idx = match missing {
                MissingSlot::Any => rng.gen_range(0u32..4),
                MissingSlot::LeftNum => 0,
                MissingSlot::LeftDen => 1,
                MissingSlot::RightNum => 2,
                MissingSlot::RightDen => 3,
            };

            vec![ln, ld, rn, rd, missing_idx]
        })
        .collect()
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Which slot is left blank for the student to fill in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MissingSlot {
    /// Randomly choose a slot per problem.
    #[default]
    Any,
    LeftNum,
    LeftDen,
    RightNum,
    RightDen,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DigitRange, Locale};

    fn test_params(
        denominators: Vec<u32>,
        scale: DigitRange,
        missing: MissingSlot,
        proper_only: bool,
    ) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::FractionEquiv {
                denominators,
                scale,
                missing,
                proper_only,
            },
            num_problems: 12,
            cols: 3,
            paper: crate::Paper::A4,
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Locale::Us,
            solve_first: false,
            include_answers: false,
            student_name: None,
        }
    }

    #[test]
    fn base_fraction_always_lowest_terms() {
        let problems = generate_problems(&test_params(
            vec![2, 3, 4, 5, 6, 8, 10],
            DigitRange::new(2, 5),
            MissingSlot::Any,
            true,
        ));
        for p in &problems {
            let (ln, ld, rn, rd) = (p[0], p[1], p[2], p[3]);
            // One side must be in lowest terms (the base fraction).
            let left_reduced = gcd(ln, ld) == 1;
            let right_reduced = gcd(rn, rd) == 1;
            assert!(
                left_reduced || right_reduced,
                "neither side is in lowest terms: {ln}/{ld} = {rn}/{rd}"
            );
        }
    }

    #[test]
    fn fractions_are_equivalent() {
        let problems = generate_problems(&test_params(
            vec![2, 3, 4, 5, 6],
            DigitRange::new(2, 4),
            MissingSlot::Any,
            true,
        ));
        for p in &problems {
            let (ln, ld, rn, rd) = (p[0], p[1], p[2], p[3]);
            // Cross-multiply: ln*rd == rn*ld.
            assert_eq!(
                ln * rd, rn * ld,
                "{ln}/{ld} != {rn}/{rd} (not equivalent)"
            );
        }
    }

    #[test]
    fn missing_idx_in_range() {
        let problems = generate_problems(&test_params(
            vec![3, 4, 5],
            DigitRange::new(2, 3),
            MissingSlot::Any,
            true,
        ));
        for p in &problems {
            assert!(p[4] < 4, "missing index out of range: {}", p[4]);
        }
    }

    #[test]
    fn fixed_missing_slot_respected() {
        let problems = generate_problems(&test_params(
            vec![2, 3, 4],
            DigitRange::new(2, 3),
            MissingSlot::RightDen,
            true,
        ));
        for p in &problems {
            assert_eq!(p[4], 3, "expected right-den (3), got {}", p[4]);
        }
    }
}

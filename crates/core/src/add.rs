//! Addition worksheet.

use crate::template;
use crate::{CarryMode, WorksheetParams, WorksheetType};

pub fn generate_typ(params: &WorksheetParams) -> String {
    let problems = generate_problems(params);
    template::render("sym.plus", &problems, params)
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    // Extract our variant-specific fields.
    // The dispatch in generate() guarantees this is WorksheetType::Add.
    let (digits, carry) = match &params.worksheet {
        WorksheetType::Add { digits, carry } => (digits, *carry),
        _ => unreachable!(),
    };

    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let max_attempts = params.num_problems * 100;
    let mut problems = Vec::with_capacity(params.num_problems as usize);
    let mut attempts = 0;

    while problems.len() < params.num_problems as usize && attempts < max_attempts {
        let nums: Vec<u32> = digits.iter().map(|d| d.random(&mut rng)).collect();
        attempts += 1;

        let ok = match carry {
            CarryMode::Any => true,
            CarryMode::None => !has_carry(&nums),
            CarryMode::Force => has_carry(&nums),
            CarryMode::Ripple => has_ripple_carry(&nums),
        };

        if ok && !problems.contains(&nums) {
            problems.push(nums);
        }
    }

    problems
}

fn column_sum(nums: &[u32], pos: u32) -> u32 {
    let divisor = 10u32.pow(pos);
    nums.iter().map(|n| (n / divisor) % 10).sum()
}

fn max_digits(nums: &[u32]) -> u32 {
    nums.iter()
        .map(|&n| if n == 0 { 1 } else { n.ilog10() + 1 })
        .max()
        .unwrap_or(1)
}

fn has_carry(nums: &[u32]) -> bool {
    let cols = max_digits(nums);
    let mut carry = 0u32;
    for pos in 0..cols {
        let total = column_sum(nums, pos) + carry;
        carry = total / 10;
        if carry > 0 {
            return true;
        }
    }
    false
}

fn has_ripple_carry(nums: &[u32]) -> bool {
    let cols = max_digits(nums);
    let mut carry = 0u32;
    let mut consecutive = 0u32;
    for pos in 0..cols {
        let total = column_sum(nums, pos) + carry;
        carry = total / 10;
        if carry > 0 {
            consecutive += 1;
            if consecutive >= 2 {
                return true;
            }
        } else {
            consecutive = 0;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_carry_two_operands() {
        assert!(!has_carry(&[23, 14]));
        assert!(has_carry(&[27, 15]));
    }

    #[test]
    fn test_has_carry_three_operands() {
        assert!(!has_carry(&[11, 12, 13]));
        assert!(has_carry(&[15, 13, 14]));
    }

    #[test]
    fn test_has_ripple_carry() {
        assert!(has_ripple_carry(&[95, 15]));
        assert!(!has_ripple_carry(&[27, 15]));
    }

    #[test]
    fn test_has_ripple_carry_three_operands() {
        assert!(has_ripple_carry(&[35, 35, 35]));
        assert!(!has_ripple_carry(&[11, 12, 18]));
    }

    #[test]
    fn test_no_carry_problems() {
        let params = test_params(CarryMode::None, vec![DR::fixed(2), DR::fixed(2)]);
        let problems = generate_problems(&params);
        for nums in &problems {
            assert!(!has_carry(nums), "{nums:?} has a carry");
        }
    }

    #[test]
    fn test_force_carry_three_operands() {
        let params = test_params(CarryMode::Force, vec![DR::fixed(2), DR::fixed(2), DR::fixed(2)]);
        let problems = generate_problems(&params);
        assert_eq!(problems.len(), 20);
        for nums in &problems {
            assert_eq!(nums.len(), 3);
            assert!(has_carry(nums), "{nums:?} has no carry");
        }
    }

    use crate::DigitRange as DR;

    fn test_params(carry: CarryMode, digits: Vec<crate::DigitRange>) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::Add { digits, carry },
            num_problems: 20,
            cols: 4,
            font: "Cascadia Code".into(),
            paper: "a4".into(),
            debug: false,
            seed: Some(42),
            symbol: None,
        }
    }
}

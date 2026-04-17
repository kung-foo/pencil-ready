//! Addition worksheet.

use crate::template;
use crate::{CarryMode, WorksheetParams, WorksheetType};

pub fn generate_typ(params: &WorksheetParams) -> anyhow::Result<String> {
    let binary = match &params.worksheet {
        WorksheetType::Add { binary, .. } => *binary,
        _ => unreachable!(),
    };
    let problems = generate_problems(params);
    if binary {
        // Operand digit-count (a.k.a. bit-count) is the pad width used by
        // the typst component to left-pad with zeros for column alignment.
        let pad_width = match &params.worksheet {
            WorksheetType::Add { digits, .. } => digits.iter().map(|d| d.max).max().unwrap_or(0),
            _ => unreachable!(),
        };
        template::render_padded("sym.plus", &problems, params, 1, pad_width)
    } else {
        template::render("sym.plus", &problems, params, 1)
    }
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    // Extract our variant-specific fields.
    // The dispatch in generate() guarantees this is WorksheetType::Add.
    let (digits, carry, binary) = match &params.worksheet {
        WorksheetType::Add { digits, carry, binary } => (digits, *carry, *binary),
        _ => unreachable!(),
    };

    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    // In binary mode, `digits` is really a bit-count range per operand.
    // Draw values in [0, 2^d − 1]; `radix` controls carry detection.
    let radix: u32 = if binary { 2 } else { 10 };
    let pick = |range: crate::DigitRange, rng: &mut SmallRng| -> u32 {
        if binary {
            let d = rng.gen_range(range.min..=range.max);
            let lo = if d == 1 { 0 } else { 1u32 << (d - 1) };
            let hi = (1u32 << d) - 1;
            rng.gen_range(lo..=hi)
        } else {
            range.random(rng)
        }
    };

    let total = params.total_problems();
    let max_attempts = total * 100;
    let mut problems = Vec::with_capacity(total as usize);
    let mut attempts = 0;

    while problems.len() < total as usize && attempts < max_attempts {
        let nums: Vec<u32> = digits.iter().map(|d| pick(*d, &mut rng)).collect();
        attempts += 1;

        let ok = match carry {
            CarryMode::Any => true,
            CarryMode::None => !has_carry(&nums, radix),
            CarryMode::Force => has_carry(&nums, radix),
            CarryMode::Ripple => has_ripple_carry(&nums, radix),
        };

        if ok && !problems.contains(&nums) {
            problems.push(nums);
        }
    }

    // Compute the sum (in actual numeric space) and append as the answer.
    // For binary mode, operand and sum values are re-encoded as their
    // base-2 digits interpreted in base 10 (e.g. 0b1011 → u32 1011) so
    // `str(n)` in the typst component displays the binary representation.
    // Leading zeros are restored by the component's pad-width.
    problems
        .into_iter()
        .map(|nums| {
            let sum = nums.iter().sum::<u32>();
            let mut row: Vec<u32> = if binary {
                nums.iter().map(|&n| encode_binary(n)).collect()
            } else {
                nums
            };
            row.push(if binary { encode_binary(sum) } else { sum });
            row
        })
        .collect()
}

/// Convert a numeric value to its binary representation encoded as a
/// base-10 integer: `0b1011` → `1011`. Leading zeros are lost in the
/// encoding and restored at render time via the component's pad-width.
fn encode_binary(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut result = 0u32;
    let mut place = 1u32;
    let mut tmp = n;
    while tmp > 0 {
        result += (tmp & 1) * place;
        tmp >>= 1;
        place *= 10;
    }
    result
}

fn bit_count(n: u32) -> u32 {
    if n == 0 { 1 } else { 32 - n.leading_zeros() }
}

fn column_sum(nums: &[u32], pos: u32, radix: u32) -> u32 {
    let divisor = radix.pow(pos);
    nums.iter().map(|n| (n / divisor) % radix).sum()
}

fn max_columns(nums: &[u32], radix: u32) -> u32 {
    nums.iter()
        .map(|&n| match radix {
            2 => bit_count(n),
            10 => if n == 0 { 1 } else { n.ilog10() + 1 },
            _ => unreachable!("only base-2 and base-10 are supported"),
        })
        .max()
        .unwrap_or(1)
}

fn has_carry(nums: &[u32], radix: u32) -> bool {
    let cols = max_columns(nums, radix);
    let mut carry = 0u32;
    for pos in 0..cols {
        let total = column_sum(nums, pos, radix) + carry;
        carry = total / radix;
        if carry > 0 {
            return true;
        }
    }
    false
}

fn has_ripple_carry(nums: &[u32], radix: u32) -> bool {
    let cols = max_columns(nums, radix);
    let mut carry = 0u32;
    let mut consecutive = 0u32;
    for pos in 0..cols {
        let total = column_sum(nums, pos, radix) + carry;
        carry = total / radix;
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
        assert!(!has_carry(&[23, 14], 10));
        assert!(has_carry(&[27, 15], 10));
    }

    #[test]
    fn test_has_carry_three_operands() {
        assert!(!has_carry(&[11, 12, 13], 10));
        assert!(has_carry(&[15, 13, 14], 10));
    }

    #[test]
    fn test_has_ripple_carry() {
        assert!(has_ripple_carry(&[95, 15], 10));
        assert!(!has_ripple_carry(&[27, 15], 10));
    }

    #[test]
    fn test_has_ripple_carry_three_operands() {
        assert!(has_ripple_carry(&[35, 35, 35], 10));
        assert!(!has_ripple_carry(&[11, 12, 18], 10));
    }

    #[test]
    fn test_binary_carry_detection() {
        // 0b11 + 0b01 = 0b100 → has carry
        assert!(has_carry(&[0b11, 0b01], 2));
        // 0b10 + 0b01 = 0b11 → no carry
        assert!(!has_carry(&[0b10, 0b01], 2));
        // 0b1111 + 0b0001 = 0b10000 → ripple carry
        assert!(has_ripple_carry(&[0b1111, 0b0001], 2));
    }

    #[test]
    fn test_encode_binary() {
        assert_eq!(encode_binary(0), 0);
        assert_eq!(encode_binary(5), 101);     // 0b101
        assert_eq!(encode_binary(11), 1011);   // 0b1011
        assert_eq!(encode_binary(30), 11110);  // 0b11110
    }

    #[test]
    fn test_no_carry_problems() {
        let params = test_params(CarryMode::None, vec![DR::fixed(2), DR::fixed(2)]);
        let problems = generate_problems(&params);
        for nums in &problems {
            // Last element is the sum; operands are the prefix.
            let operands = &nums[..nums.len() - 1];
            assert!(!has_carry(operands, 10), "{operands:?} has a carry");
        }
    }

    #[test]
    fn test_force_carry_three_operands() {
        let params = test_params(CarryMode::Force, vec![DR::fixed(2), DR::fixed(2), DR::fixed(2)]);
        let problems = generate_problems(&params);
        assert_eq!(problems.len(), 20);
        for nums in &problems {
            // 3 operands + 1 appended sum = 4 entries.
            assert_eq!(nums.len(), 4);
            let operands = &nums[..nums.len() - 1];
            assert!(has_carry(operands, 10), "{operands:?} has no carry");
        }
    }

    use crate::DigitRange as DR;

    fn test_params(carry: CarryMode, digits: Vec<crate::DigitRange>) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::Add { digits, carry, binary: false },
            num_problems: 20,
            cols: 4,
            paper: "a4".into(),
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Default::default(),
            pages: 1,
            solve_first: false,
        }
    }
}

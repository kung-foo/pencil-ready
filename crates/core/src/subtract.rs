//! Subtraction worksheet.

use crate::document;
use crate::{BorrowMode, ComponentOpts, Sheet, WorksheetParams, WorksheetType};

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let problems = generate_problems(params);
    let max_digits = document::max_digits(&problems);
    let operator = params
        .symbol
        .clone()
        .unwrap_or_else(|| "sym.minus".to_string());
    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            operator,
            width_cm: document::box_width_cm(&params.worksheet, max_digits),
            answer_rows: 1,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
        },
    })
}

fn generate_problems(params: &WorksheetParams) -> Vec<Vec<u32>> {
    let (digits, borrow) = match &params.worksheet {
        WorksheetType::Subtract { digits, borrow } => (digits, *borrow),
        _ => unreachable!(),
    };

    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let top_range = digits[0];
    let bot_range = *digits.get(1).unwrap_or(&top_range);

    let total = params.total_problems();
    let max_attempts = total * 100;
    let mut problems = Vec::with_capacity(total as usize);
    let mut attempts = 0;

    while problems.len() < total as usize && attempts < max_attempts {
        let mut a = top_range.random(&mut rng);
        let mut b = bot_range.random(&mut rng);
        attempts += 1;

        if a < b {
            std::mem::swap(&mut a, &mut b);
        }

        let ok = match borrow {
            BorrowMode::Any => true,
            BorrowMode::None => !has_borrow(a, b),
            BorrowMode::NoAcrossZero => !has_borrow_across_zero(a, b),
            BorrowMode::Force => has_borrow(a, b),
            BorrowMode::Ripple => has_ripple_borrow(a, b),
        };

        if ok {
            let candidate = vec![a, b];
            if !problems.contains(&candidate) {
                problems.push(candidate);
            }
        }
    }

    crate::pad_with_duplicates(&mut problems, total as usize, &mut rng);

    // Append the difference so the typst component can render the answer
    // in solved mode.
    problems
        .into_iter()
        .map(|nums| vec![nums[0], nums[1], nums[0] - nums[1]])
        .collect()
}

fn has_borrow(a: u32, b: u32) -> bool {
    let mut a = a;
    let mut b = b;
    while a > 0 || b > 0 {
        if (a % 10) < (b % 10) {
            return true;
        }
        a /= 10;
        b /= 10;
    }
    false
}

fn has_ripple_borrow(a: u32, b: u32) -> bool {
    let a_digs = digit_vec(a);
    let b_digs = digit_vec(b);
    let len = a_digs.len().max(b_digs.len());
    let mut borrow: i32 = 0;
    let mut consecutive = 0u32;

    for i in 0..len {
        let ad = *a_digs.get(i).unwrap_or(&0) as i32 - borrow;
        let bd = *b_digs.get(i).unwrap_or(&0) as i32;
        if ad < bd {
            consecutive += 1;
            if consecutive >= 2 {
                return true;
            }
            borrow = 1;
        } else {
            consecutive = 0;
            borrow = 0;
        }
    }
    false
}

fn has_borrow_across_zero(a: u32, b: u32) -> bool {
    let a_digs = digit_vec(a);
    let b_digs = digit_vec(b);
    let len = a_digs.len().max(b_digs.len());
    let mut borrow: i32 = 0;

    for i in 0..len {
        let ad = *a_digs.get(i).unwrap_or(&0) as i32 - borrow;
        let bd = *b_digs.get(i).unwrap_or(&0) as i32;
        if ad < bd {
            if let Some(&next_a) = a_digs.get(i + 1) {
                if next_a == 0 {
                    return true;
                }
            }
            borrow = 1;
        } else {
            borrow = 0;
        }
    }
    false
}

fn digit_vec(mut n: u32) -> Vec<u32> {
    if n == 0 {
        return vec![0];
    }
    let mut ds = Vec::new();
    while n > 0 {
        ds.push(n % 10);
        n /= 10;
    }
    ds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_borrow() {
        assert!(!has_borrow(86, 43));
        assert!(has_borrow(83, 46));
    }

    #[test]
    fn test_has_ripple_borrow() {
        assert!(has_ripple_borrow(534, 278));
        assert!(!has_ripple_borrow(83, 46));
    }

    #[test]
    fn test_has_borrow_across_zero() {
        assert!(has_borrow_across_zero(403, 157));
        assert!(!has_borrow_across_zero(413, 157));
    }

    #[test]
    fn test_mode_none() {
        let params = test_params(BorrowMode::None, vec![DR::fixed(2), DR::fixed(2)]);
        let problems = generate_problems(&params);
        for nums in &problems {
            assert!(!has_borrow(nums[0], nums[1]), "{:?} has a borrow", nums);
        }
    }

    #[test]
    fn test_mode_ripple() {
        let params = test_params(BorrowMode::Ripple, vec![DR::fixed(3), DR::fixed(3)]);
        let problems = generate_problems(&params);
        for nums in &problems {
            assert!(has_ripple_borrow(nums[0], nums[1]), "{:?} no ripple", nums);
        }
    }

    use crate::DigitRange as DR;

    fn test_params(borrow: BorrowMode, digits: Vec<crate::DigitRange>) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::Subtract { digits, borrow },
            num_problems: 20,
            cols: 4,
            paper: crate::Paper::A4,
            debug: false,
            seed: Some(42),
            symbol: None,
            locale: Default::default(),
            pages: 1,
            solve_first: false,
            include_answers: false,
            student_name: None,
        }
    }
}

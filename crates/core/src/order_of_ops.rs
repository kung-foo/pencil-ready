//! Order of operations — horizontal expressions that mix additive and
//! multiplicative operators, optionally with one parenthesis group.
//!
//! Each problem is an inline expression like `5 + 6 × 2 = ___`. The
//! whole point of the worksheet is precedence (× / ÷ before + / −), so
//! every problem mixes at least one of {+, −} with at least one of
//! {×, ÷}. Four forms cover the curriculum progression:
//!
//!   form 0: two ops, no parens         (`a OP1 b OP2 c`)
//!   form 1: three ops, no parens       (`a OP1 b OP2 c OP3 d`)
//!   form 2: two ops, parens around lhs (`(a OP1 b) OP2 c`)
//!   form 3: two ops, parens around rhs (`a OP1 (b OP2 c)`)
//!
//! Forms 2 and 3 are both emitted when `use_parens` is on so the
//! parens land on either side of the multiplicative op — a kid solving
//! the page sees both shapes.
//!
//! The form discriminator is the first element of each problem vector;
//! the typst component reads it to choose the per-row layout. Op codes
//! are 0=+, 1=−, 2=×, 3=÷. Locale only swaps the × / ÷ glyphs — the
//! generator emits the same codes regardless of locale.
//!
//! Divisor-of-1 is rejected at the `apply` level: `n ÷ 1 = n` is a
//! free trick that doesn't exercise precedence, so the generator
//! excludes any problem that would print a `÷ 1` step.

use crate::{ComponentOpts, Sheet, WorksheetParams, WorksheetType, pad_with_duplicates};

/// Per-operand range used when sampling. Operands are single-digit so
/// the worst-case answer (form 1: `a + b × c + d ≤ 9 + 81 + 9 = 99`)
/// stays under the cap without aggressive rejection.
const OPERAND_MIN: i64 = 1;
const OPERAND_MAX: i64 = 9;
/// Reject any problem whose final or intermediate value exceeds this.
/// Keeps answers compact and the answer-key column narrow.
const VALUE_CAP: i64 = 100;

/// Op codes in `Sheet.problems`. Kept inline (small, only-used-here).
const OP_PLUS: u32 = 0;
const OP_MINUS: u32 = 1;
const OP_TIMES: u32 = 2;
const OP_DIVIDE: u32 = 3;

fn is_additive(op: u32) -> bool {
    op == OP_PLUS || op == OP_MINUS
}

fn is_multiplicative(op: u32) -> bool {
    op == OP_TIMES || op == OP_DIVIDE
}

/// Apply an op to two integers; returns `None` for divide-by-zero,
/// divide-by-one (trivial, doesn't exercise precedence), or
/// non-integer division.
fn apply(op: u32, a: i64, b: i64) -> Option<i64> {
    match op {
        OP_PLUS => Some(a + b),
        OP_MINUS => Some(a - b),
        OP_TIMES => Some(a * b),
        OP_DIVIDE => {
            if b == 0 || b == 1 || a % b != 0 {
                None
            } else {
                Some(a / b)
            }
        }
        _ => None,
    }
}

pub fn generate(params: &WorksheetParams) -> anyhow::Result<Sheet> {
    let (operations, use_parens) = match &params.worksheet {
        WorksheetType::OrderOfOperations {
            operations,
            use_parens,
        } => (*operations, *use_parens),
        _ => unreachable!(),
    };

    let problems = generate_problems(params, operations, use_parens);

    // Locale-driven defaults; `--symbol` (if set) wins for the times
    // glyph, matching the drills' convention.
    let times_op = params
        .symbol
        .clone()
        .unwrap_or_else(|| params.locale.multiply_symbol().to_string());
    let div_op = params.locale.divide_symbol().to_string();

    Ok(Sheet {
        worksheet: params.worksheet.clone(),
        problems,
        opts: ComponentOpts {
            operator: times_op,
            divide_operator: div_op,
            width_cm: 0.0,
            answer_rows: 1,
            pad_width: 0,
            implicit: false,
            variable: "x".to_string(),
            decimal_places: Vec::new(),
            reserve_remainder: false,
        },
    })
}

fn generate_problems(
    params: &WorksheetParams,
    operations: u32,
    use_parens: bool,
) -> Vec<Vec<u32>> {
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use std::collections::HashSet;

    let mut rng = match params.seed {
        Some(s) => SmallRng::seed_from_u64(s),
        None => SmallRng::from_entropy(),
    };

    let target = params.total_problems() as usize;
    if target == 0 {
        return Vec::new();
    }

    let mut seen: HashSet<Vec<u32>> = HashSet::new();
    let mut problems: Vec<Vec<u32>> = Vec::new();
    // The constraint space is fairly large (~thousands of valid tuples),
    // so a generous attempt budget hits the target without spinning.
    let max_attempts = target.saturating_mul(400).max(2_000);
    for _ in 0..max_attempts {
        if problems.len() >= target {
            break;
        }
        if let Some(p) = sample_one(&mut rng, operations, use_parens) {
            if seen.insert(p.clone()) {
                problems.push(p);
            }
        }
    }
    pad_with_duplicates(&mut problems, target, &mut rng);
    problems
}

fn sample_one(rng: &mut rand::rngs::SmallRng, operations: u32, use_parens: bool) -> Option<Vec<u32>> {
    use rand::Rng;

    let pick_op = |rng: &mut rand::rngs::SmallRng| -> u32 { rng.gen_range(0..4) };
    let pick_val = |rng: &mut rand::rngs::SmallRng| -> i64 { rng.gen_range(OPERAND_MIN..=OPERAND_MAX) };

    if use_parens {
        // Two parens shapes:
        //   form 2: `(a + b) × c` — parens wrap the additive LHS
        //   form 3: `a × (b + c)` — parens wrap the additive RHS
        // Coin flip per problem so a page mixes both layouts.
        if rng.r#gen::<bool>() {
            // Form 2: `(a op1 b) op2 c`. op1 additive, op2 multiplicative.
            let op1 = if rng.r#gen::<bool>() { OP_PLUS } else { OP_MINUS };
            let op2 = if rng.r#gen::<bool>() { OP_TIMES } else { OP_DIVIDE };
            let a = pick_val(rng);
            let b = pick_val(rng);
            let c = pick_val(rng);
            if op1 == OP_MINUS && a < b {
                return None;
            }
            let inner = apply(op1, a, b)?;
            if inner < 0 {
                return None;
            }
            let final_ = apply(op2, inner, c)?;
            if !(0..=VALUE_CAP).contains(&final_) {
                return None;
            }
            Some(vec![
                2,
                a as u32,
                op1,
                b as u32,
                op2,
                c as u32,
                final_ as u32,
            ])
        } else {
            // Form 3: `a op1 (b op2 c)`. op1 multiplicative, op2 additive.
            let op1 = if rng.r#gen::<bool>() { OP_TIMES } else { OP_DIVIDE };
            let op2 = if rng.r#gen::<bool>() { OP_PLUS } else { OP_MINUS };
            let a = pick_val(rng);
            let b = pick_val(rng);
            let c = pick_val(rng);
            if op2 == OP_MINUS && b < c {
                return None;
            }
            let inner = apply(op2, b, c)?;
            // `apply` already rejects divisor 0 or 1; explicitly bail
            // here so a zero inner (e.g. `5 - 5 = 0`) doesn't propagate
            // into a divide-by-zero on the next step.
            if inner < 0 {
                return None;
            }
            let final_ = apply(op1, a, inner)?;
            if !(0..=VALUE_CAP).contains(&final_) {
                return None;
            }
            Some(vec![
                3,
                a as u32,
                op1,
                b as u32,
                op2,
                c as u32,
                final_ as u32,
            ])
        }
    } else if operations == 2 {
        // Form 0: `a op1 b op2 c`, evaluated by standard precedence.
        let op1 = pick_op(rng);
        let op2 = pick_op(rng);
        let has_add = is_additive(op1) || is_additive(op2);
        let has_mult = is_multiplicative(op1) || is_multiplicative(op2);
        if !(has_add && has_mult) {
            return None;
        }
        let a = pick_val(rng);
        let b = pick_val(rng);
        let c = pick_val(rng);

        // Exactly one of {op1, op2} is multiplicative (constraint above).
        let answer = if is_multiplicative(op1) {
            // (a op1 b) op2 c
            let inter = apply(op1, a, b)?;
            if inter < 0 {
                return None;
            }
            if op2 == OP_MINUS && inter < c {
                return None;
            }
            apply(op2, inter, c)?
        } else {
            // a op1 (b op2 c)
            let inter = apply(op2, b, c)?;
            if inter < 0 {
                return None;
            }
            if op1 == OP_MINUS && a < inter {
                return None;
            }
            apply(op1, a, inter)?
        };
        if !(0..=VALUE_CAP).contains(&answer) {
            return None;
        }

        Some(vec![0, a as u32, op1, b as u32, op2, c as u32, answer as u32])
    } else {
        // Form 1: `a op1 b op2 c op3 d`. Evaluate multiplicative ops
        // left-to-right first, then additive ops left-to-right.
        let op1 = pick_op(rng);
        let op2 = pick_op(rng);
        let op3 = pick_op(rng);
        let ops_arr = [op1, op2, op3];
        if !ops_arr.iter().any(|&o| is_additive(o)) {
            return None;
        }
        if !ops_arr.iter().any(|&o| is_multiplicative(o)) {
            return None;
        }
        let a = pick_val(rng);
        let b = pick_val(rng);
        let c = pick_val(rng);
        let d = pick_val(rng);

        let mut vals: Vec<i64> = vec![a, b, c, d];
        let mut ops: Vec<u32> = vec![op1, op2, op3];

        // Pass 1: collapse multiplicative ops left-to-right.
        let mut i = 0;
        while i < ops.len() {
            if is_multiplicative(ops[i]) {
                let r = apply(ops[i], vals[i], vals[i + 1])?;
                if r < 0 || r > VALUE_CAP {
                    return None;
                }
                vals[i] = r;
                vals.remove(i + 1);
                ops.remove(i);
            } else {
                i += 1;
            }
        }
        // Pass 2: collapse additive ops left-to-right.
        let mut acc = vals[0];
        for (idx, &op) in ops.iter().enumerate() {
            // Subtraction underflow check before applying.
            if op == OP_MINUS && acc < vals[idx + 1] {
                return None;
            }
            acc = apply(op, acc, vals[idx + 1])?;
            if !(0..=VALUE_CAP).contains(&acc) {
                return None;
            }
        }

        Some(vec![
            1,
            a as u32,
            op1,
            b as u32,
            op2,
            c as u32,
            op3,
            d as u32,
            acc as u32,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, Paper};

    fn params(operations: u32, use_parens: bool) -> WorksheetParams {
        WorksheetParams {
            worksheet: WorksheetType::OrderOfOperations {
                operations,
                use_parens,
            },
            num_problems: 12,
            cols: 2,
            paper: Paper::A4,
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
    fn two_op_flat_generates_target_count() {
        let p = params(2, false);
        let problems = generate_problems(&p, 2, false);
        assert_eq!(problems.len(), 12);
        for nums in &problems {
            assert_eq!(nums.len(), 7);
            assert_eq!(nums[0], 0, "form 0");
            // Constraint: at least one additive AND one multiplicative.
            let op1 = nums[2];
            let op2 = nums[4];
            assert!(
                (is_additive(op1) || is_additive(op2))
                    && (is_multiplicative(op1) || is_multiplicative(op2)),
                "problem {:?} lacks the mixed-precedence constraint",
                nums,
            );
        }
    }

    #[test]
    fn three_op_flat_generates_target_count() {
        let p = params(3, false);
        let problems = generate_problems(&p, 3, false);
        assert_eq!(problems.len(), 12);
        for nums in &problems {
            assert_eq!(nums.len(), 9);
            assert_eq!(nums[0], 1, "form 1");
        }
    }

    #[test]
    fn parens_forms_cover_both_shapes() {
        let p = params(2, true);
        let problems = generate_problems(&p, 2, true);
        assert_eq!(problems.len(), 12);
        let mut saw_form2 = false;
        let mut saw_form3 = false;
        for nums in &problems {
            assert_eq!(nums.len(), 7);
            let form = nums[0];
            let op1 = nums[2];
            let op2 = nums[4];
            match form {
                2 => {
                    saw_form2 = true;
                    assert!(is_additive(op1), "form 2: op1 must be + or −");
                    assert!(is_multiplicative(op2), "form 2: op2 must be × or ÷");
                }
                3 => {
                    saw_form3 = true;
                    assert!(is_multiplicative(op1), "form 3: op1 must be × or ÷");
                    assert!(is_additive(op2), "form 3: op2 must be + or −");
                }
                _ => panic!("unexpected form {form} in parens-mode output: {nums:?}"),
            }
        }
        // With seed 42 + 12 problems the coin flip should land on both
        // shapes; if this ever flakes, raise problem count or relax.
        assert!(saw_form2, "no form 2 problems generated");
        assert!(saw_form3, "no form 3 problems generated");
    }

    #[test]
    fn no_divisor_is_one() {
        // Across every form, no rendered `÷` step should ever be `÷ 1`.
        // For forms 0/1/2 the divisor is always a literal operand; for
        // form 3 with op1 = ÷, the divisor is the (b op2 c) inner —
        // both must be ≠ 1.
        for &(ops, parens) in &[(2u32, false), (3, false), (2, true)] {
            let p = params(ops, parens);
            let problems = generate_problems(&p, ops, parens);
            for nums in &problems {
                let form = nums[0];
                match form {
                    0 => {
                        let op1 = nums[2];
                        let b = nums[3] as i64;
                        let op2 = nums[4];
                        let c = nums[5] as i64;
                        if op1 == OP_DIVIDE {
                            assert_ne!(b, 1, "form 0 divisor=1 in {nums:?}");
                        }
                        if op2 == OP_DIVIDE {
                            assert_ne!(c, 1, "form 0 divisor=1 in {nums:?}");
                        }
                    }
                    1 => {
                        for (op_idx, val_idx) in [(2, 3), (4, 5), (6, 7)] {
                            if nums[op_idx] == OP_DIVIDE {
                                assert_ne!(
                                    nums[val_idx], 1,
                                    "form 1 divisor=1 in {nums:?}"
                                );
                            }
                        }
                    }
                    2 => {
                        // op2 is the only divide-capable slot in form 2.
                        let op2 = nums[4];
                        let c = nums[5] as i64;
                        if op2 == OP_DIVIDE {
                            assert_ne!(c, 1, "form 2 divisor=1 in {nums:?}");
                        }
                    }
                    3 => {
                        // op1 is the only divide-capable slot in form 3,
                        // but the divisor is the inner (b op2 c).
                        let op1 = nums[2];
                        let b = nums[3] as i64;
                        let op2 = nums[4];
                        let c = nums[5] as i64;
                        if op1 == OP_DIVIDE {
                            let inner = apply(op2, b, c).unwrap();
                            assert_ne!(inner, 1, "form 3 inner=1 in {nums:?}");
                        }
                    }
                    other => panic!("unexpected form {other} in {nums:?}"),
                }
            }
        }
    }

    #[test]
    fn answers_under_value_cap() {
        for &(ops, parens) in &[(2u32, false), (3, false), (2, true)] {
            let p = params(ops, parens);
            let problems = generate_problems(&p, ops, parens);
            for nums in &problems {
                let answer = *nums.last().unwrap() as i64;
                assert!(
                    (0..=VALUE_CAP).contains(&answer),
                    "answer {} out of range for form {:?}",
                    answer,
                    nums[0]
                );
            }
        }
    }

    #[test]
    fn evaluations_match_encoded_answer_form0() {
        let p = params(2, false);
        let problems = generate_problems(&p, 2, false);
        for nums in &problems {
            let a = nums[1] as i64;
            let op1 = nums[2];
            let b = nums[3] as i64;
            let op2 = nums[4];
            let c = nums[5] as i64;
            let answer = nums[6] as i64;
            let computed = if is_multiplicative(op1) {
                apply(op2, apply(op1, a, b).unwrap(), c).unwrap()
            } else if is_multiplicative(op2) {
                apply(op1, a, apply(op2, b, c).unwrap()).unwrap()
            } else {
                unreachable!()
            };
            assert_eq!(computed, answer, "mismatched answer for {:?}", nums);
        }
    }

    #[test]
    fn evaluations_match_encoded_answer_parens() {
        let p = params(2, true);
        let problems = generate_problems(&p, 2, true);
        for nums in &problems {
            let form = nums[0];
            let a = nums[1] as i64;
            let op1 = nums[2];
            let b = nums[3] as i64;
            let op2 = nums[4];
            let c = nums[5] as i64;
            let answer = nums[6] as i64;
            let computed = if form == 2 {
                // `(a op1 b) op2 c` — parens force the additive LHS.
                let inner = apply(op1, a, b).unwrap();
                apply(op2, inner, c).unwrap()
            } else {
                // form 3: `a op1 (b op2 c)` — parens force the
                // additive RHS.
                let inner = apply(op2, b, c).unwrap();
                apply(op1, a, inner).unwrap()
            };
            assert_eq!(computed, answer, "mismatched answer for {:?}", nums);
        }
    }

    #[test]
    fn reproducible_with_seed() {
        let p = params(2, false);
        let a = generate_problems(&p, 2, false);
        let b = generate_problems(&p, 2, false);
        assert_eq!(a, b);
    }
}

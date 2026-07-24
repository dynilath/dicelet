use crate::error::{DiceletError, Result};
use crate::number::Number;
use crate::parser::ast::{BinOp, Expr, Level};
use crate::roll::roll_with_keep;
use crate::rng::Rng;

/// A single scalar result with its detailed roll information.
#[derive(Debug, Clone)]
pub struct ScalarValue {
    /// The numeric result
    pub value: Number,
    /// Detail string for display (e.g. `[5 + 3 + 1* + 6]`)
    pub detail: String,
}

/// The result of evaluating an expression.
#[derive(Debug, Clone)]
pub enum EvalValue {
    /// A single scalar result
    Scalar(ScalarValue),
    /// Multiple independent results (from `#` or `{}`)
    Set(Vec<ScalarValue>),
}

impl EvalValue {
    pub fn is_set(&self) -> bool {
        matches!(self, EvalValue::Set(_))
    }

    /// Get all numeric values as a vec.
    pub fn values(&self) -> Vec<Number> {
        match self {
            EvalValue::Scalar(s) => vec![s.value],
            EvalValue::Set(items) => items.iter().map(|s| s.value).collect(),
        }
    }

    /// Format the summary string (final result without detail).
    pub fn summary(&self) -> String {
        match self {
            EvalValue::Scalar(s) => s.value.to_string(),
            EvalValue::Set(items) => {
                let parts: Vec<String> = items.iter().map(|s| s.value.to_string()).collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }

    /// Format the detail string (roll details, empty if no dice).
    pub fn detail(&self) -> String {
        match self {
            EvalValue::Scalar(s) => s.detail.clone(),
            EvalValue::Set(items) => {
                let parts: Vec<String> = items.iter().map(|s| s.detail.clone()).collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }

    /// Format the full output: `detail = summary` or `{details} = {summary}`.
    pub fn full(&self) -> String {
        match self {
            EvalValue::Scalar(s) => {
                if s.detail.is_empty() {
                    s.value.to_string()
                } else {
                    format!("{} = {}", s.detail, s.value)
                }
            }
            EvalValue::Set(items) => {
                let detail_parts: Vec<String> =
                    items.iter().map(|s| s.detail.clone()).collect();
                let value_parts: Vec<String> =
                    items.iter().map(|s| s.value.to_string()).collect();
                format!("{{{}}} = {{{}}}", detail_parts.join(", "), value_parts.join(", "))
            }
        }
    }
}

/// Evaluate an expression with the given RNG.
pub fn evaluate(expr: &Expr, rng: &mut dyn Rng) -> Result<EvalValue> {
    match expr.level() {
        Level::Const => {
            let val = eval_const(&expr)?;
            Ok(EvalValue::Scalar(ScalarValue {
                value: val,
                detail: String::new(),
            }))
        }
        Level::Rand => {
            let (val, detail) = eval_rand(&expr, rng)?;
            Ok(EvalValue::Scalar(ScalarValue { value: val, detail }))
        }
        Level::Dicelet => eval_dicelet(&expr, rng),
    }
}

/// Evaluate a constant expression (no dice).
fn eval_const(expr: &Expr) -> Result<Number> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Neg(e) => Ok(-eval_const(e)?),
        Expr::Paren(e) => eval_const(e),
        Expr::BinOp(op, a, b) => {
            let a = eval_const(a)?;
            let b = eval_const(b)?;
            apply_binop(*op, a, b)
        }
        _ => Err(DiceletError::ParseError("expected const expression".into())),
    }
}

/// Evaluate a rand expression (contains dice, single result).
/// Returns (value, detail_string).
fn eval_rand(expr: &Expr, rng: &mut dyn Rng) -> Result<(Number, String)> {
    match expr {
        Expr::Dice(dice) => {
            let count = dice.count.to_i64() as i32;
            let faces = dice.faces.to_i64() as i32;
            if count < 1 || faces < 2 {
                return Err(DiceletError::InvalidDice);
            }
            let roll = roll_with_keep(rng, count, faces, dice.keep)?;
            Ok((Number::Int(roll.summary), roll.detail()))
        }
        Expr::Neg(e) => {
            let (val, detail) = eval_rand(e, rng)?;
            Ok((-val, detail))
        }
        Expr::Paren(e) => {
            let (val, detail) = eval_rand(e, rng)?;
            Ok((val, format!("({})", detail)))
        }
        Expr::BinOp(op, a, b) => {
            let level_a = a.level();
            let level_b = b.level();

            // Both sides are rand or lower — both may contain dice
            if level_a <= Level::Rand && level_b <= Level::Rand {
                let (a_val, a_detail) = eval_rand_or_const(a, rng)?;
                let (b_val, b_detail) = eval_rand_or_const(b, rng)?;
                let result = apply_binop(*op, a_val, b_val)?;
                let detail = format_binop_detail(*op, &a_detail, &b_detail);
                Ok((result, detail))
            } else {
                // One side is dicelet — shouldn't happen at rand level
                Err(DiceletError::ParseError(
                    "unexpected dicelet in rand expression".into(),
                ))
            }
        }
        _ => Err(DiceletError::ParseError("expected rand expression".into())),
    }
}

/// Evaluate an expression that could be const or rand, returning (value, detail).
/// Const expressions have empty detail.
fn eval_rand_or_const(expr: &Expr, rng: &mut dyn Rng) -> Result<(Number, String)> {
    match expr.level() {
        Level::Const => {
            let val = eval_const(expr)?;
            Ok((val, val.to_display_string()))
        }
        Level::Rand => eval_rand(expr, rng),
        _ => Err(DiceletError::ParseError("unexpected dicelet level".into())),
    }
}

/// Evaluate a dicelet expression (produces multiple results).
fn eval_dicelet(expr: &Expr, rng: &mut dyn Rng) -> Result<EvalValue> {
    match expr {
        Expr::Repeat { times, inner } => {
            let n = times.to_i64();
            if n < 1 {
                return Err(DiceletError::InvalidDice);
            }
            if n as i64 > crate::constants::MAX_DICE_UNIT_COUNT {
                return Err(DiceletError::UnitCountExceed(
                    n,
                    crate::constants::MAX_DICE_UNIT_COUNT,
                ));
            }

            let mut results = Vec::with_capacity(n as usize);
            for _ in 0..n {
                let (val, detail) = eval_rand_or_const(inner, rng)?;
                results.push(ScalarValue { value: val, detail });
            }
            Ok(EvalValue::Set(results))
        }
        Expr::Set(elements) => {
            let mut results = Vec::with_capacity(elements.len());
            for elem in elements {
                let (val, detail) = eval_rand_or_const(elem, rng)?;
                results.push(ScalarValue { value: val, detail });
            }
            Ok(EvalValue::Set(results))
        }
        Expr::Paren(e) => eval_dicelet(e, rng),
        Expr::Neg(e) => {
            let inner = eval_dicelet(e, rng)?;
            match inner {
                EvalValue::Set(items) => {
                    let negated: Vec<ScalarValue> = items
                        .into_iter()
                        .map(|s| ScalarValue {
                            value: -s.value,
                            detail: s.detail,
                        })
                        .collect();
                    Ok(EvalValue::Set(negated))
                }
                _ => Err(DiceletError::ParseError("expected set".into())),
            }
        }
        Expr::BinOp(op, a, b) => {
            let level_a = a.level();
            let level_b = b.level();

            // At least one side must be dicelet
            // Case 1: dicelet op dicelet
            if level_a == Level::Dicelet && level_b == Level::Dicelet {
                let a_set = eval_dicelet(a, rng)?;
                let b_set = eval_dicelet(b, rng)?;
                binary_set_set(*op, a_set, b_set)
            }
            // Case 2: dicelet op rand/const — rand/const is rolled once, applied to each
            else if level_a == Level::Dicelet {
                let a_set = eval_dicelet(a, rng)?;
                let (b_val, b_detail) = eval_rand_or_const(b, rng)?;
                binary_set_scalar(*op, a_set, b_val, b_detail, true)
            }
            // Case 3: rand/const op dicelet — rand/const is rolled once, applied to each
            else if level_b == Level::Dicelet {
                let (a_val, a_detail) = eval_rand_or_const(a, rng)?;
                let b_set = eval_dicelet(b, rng)?;
                binary_set_scalar(*op, b_set, a_val, a_detail, false)
            } else {
                Err(DiceletError::ParseError(
                    "expected at least one dicelet operand".into(),
                ))
            }
        }
        _ => Err(DiceletError::ParseError("expected dicelet expression".into())),
    }
}

/// Apply binary operation to two sets, element-wise.
/// If sets have different lengths, the shorter one reuses its single element
/// or pairs up to the minimum length (matching original behavior).
fn binary_set_set(
    op: BinOp,
    a: EvalValue,
    b: EvalValue,
) -> Result<EvalValue> {
    let a_items = match a {
        EvalValue::Set(s) => s,
        _ => return Err(DiceletError::ParseError("expected set".into())),
    };
    let b_items = match b {
        EvalValue::Set(s) => s,
        _ => return Err(DiceletError::ParseError("expected set".into())),
    };

    // Per original behavior: when a regular dice (single result) is used with a set,
    // it's rolled once and reused. But when two true sets (multi-element) are combined,
    // they pair up element-wise.
    // If one set has 1 element, it's reused for all elements of the other.
    let result = if a_items.len() == 1 && b_items.len() > 1 {
        let a_val = a_items[0].clone();
        b_items
            .iter()
            .map(|b_val| {
                let val = apply_binop(op, a_val.value, b_val.value).unwrap_or(Number::Int(0));
                let detail = format_binop_detail(op, &a_val.detail, &b_val.detail);
                ScalarValue { value: val, detail }
            })
            .collect::<Vec<_>>()
    } else if b_items.len() == 1 && a_items.len() > 1 {
        let b_val = b_items[0].clone();
        a_items
            .iter()
            .map(|a_val| {
                let val = apply_binop(op, a_val.value, b_val.value).unwrap_or(Number::Int(0));
                let detail = format_binop_detail(op, &a_val.detail, &b_val.detail);
                ScalarValue { value: val, detail }
            })
            .collect::<Vec<_>>()
    } else {
        // Pair element-wise up to the shorter length
        a_items
            .iter()
            .zip(b_items.iter())
            .map(|(a_val, b_val)| {
                let val = apply_binop(op, a_val.value, b_val.value).unwrap_or(Number::Int(0));
                let detail = format_binop_detail(op, &a_val.detail, &b_val.detail);
                ScalarValue { value: val, detail }
            })
            .collect::<Vec<_>>()
    };

    Ok(EvalValue::Set(result))
}

/// Apply binary operation between a set and a scalar value.
/// The scalar is applied to each element of the set.
/// `scalar_on_right` indicates whether the scalar is on the right side.
fn binary_set_scalar(
    op: BinOp,
    set: EvalValue,
    scalar_val: Number,
    scalar_detail: String,
    scalar_on_right: bool,
) -> Result<EvalValue> {
    let items = match set {
        EvalValue::Set(s) => s,
        _ => return Err(DiceletError::ParseError("expected set".into())),
    };

    let result: Vec<ScalarValue> = items
        .iter()
        .map(|item| {
            let (val, detail) = if scalar_on_right {
                let val = apply_binop(op, item.value, scalar_val).unwrap_or(Number::Int(0));
                let detail = format_binop_detail(op, &item.detail, &scalar_detail);
                (val, detail)
            } else {
                let val = apply_binop(op, scalar_val, item.value).unwrap_or(Number::Int(0));
                let detail = format_binop_detail(op, &scalar_detail, &item.detail);
                (val, detail)
            };
            ScalarValue { value: val, detail }
        })
        .collect();

    Ok(EvalValue::Set(result))
}

/// Apply a binary operation to two numbers.
fn apply_binop(op: BinOp, a: Number, b: Number) -> Result<Number> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mul => Ok(a * b),
        BinOp::Div => {
            if b.to_f64() == 0.0 {
                return Err(DiceletError::DivZero);
            }
            Ok(a / b)
        }
    }
}

/// Format the detail string for a binary operation.
fn format_binop_detail(op: BinOp, a_detail: &str, b_detail: &str) -> String {
    if a_detail.is_empty() && b_detail.is_empty() {
        String::new()
    } else if a_detail.is_empty() {
        format!("{} {}", op.symbol(), b_detail)
    } else if b_detail.is_empty() {
        format!("{} {}", a_detail, op.symbol())
    } else {
        format!("{} {} {}", a_detail, op.symbol(), b_detail)
    }
}
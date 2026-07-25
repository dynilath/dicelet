//! # Dicelet Core
//!
//! Dice expression parsing and evaluation engine.
//!
//! This crate provides the dicelet syntax parser and evaluator, a reimplementation
//! of the dice expression engine from [qq-dicebot](https://github.com/dynilath/qq-dicebot).
//!
//! ## Quick Start
//!
//! ```
//! use dicelet_core::{roll, RollOptions};
//!
//! let result = roll("4d6k3", RollOptions::default()).unwrap();
//! println!("{}", result.full); // e.g. "[5 + 3 + 1* + 6] = 14"
//! ```

pub mod constants;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod number;
pub mod parser;
pub mod rng;
pub mod roll;

pub use eval::{EvalValue, ScalarValue};
pub use error::{DiceletError, Result};
pub use number::Number;
pub use parser::ast::{BinOp, ComparisonOp, DiceExpr, Expr, KeepMode, Level};
pub use parser::{ParseResult, Parser};
pub use rng::{Rng, Xoroshiro128StarStar};
pub use roll::RollResult;

/// Options for rolling dice.
#[derive(Debug, Clone)]
pub struct RollOptions {
    /// Whether to show detailed roll results. Default: true.
    pub show_detail: bool,
    /// Optional random seed for deterministic results (testing).
    pub seed: Option<u64>,
}

impl Default for RollOptions {
    fn default() -> Self {
        Self {
            show_detail: true,
            seed: None,
        }
    }
}

/// The result of a roll operation.
#[derive(Debug, Clone)]
pub struct RollOutput {
    /// The source text that was successfully consumed.
    pub consumed: String,
    /// The remaining unparsed text (tail).
    pub tail: String,
    /// The summary string (final result without detail), e.g. "14" or "{10, 11, 13}".
    pub summary: String,
    /// The detail string (roll details), e.g. "[5 + 3 + 1* + 6]".
    pub detail: String,
    /// The full output combining detail and summary, e.g. "[5 + 3 + 1* + 6] = 14".
    pub full: String,
    /// Whether this is a multi-result set.
    pub is_set: bool,
    /// The numeric values of the result.
    pub values: Vec<f64>,
}

/// Parse and evaluate a dicelet expression.
///
/// This function combines parsing and evaluation into one step.
/// It uses strtol-style fault-tolerant parsing: if the input contains
/// invalid syntax, it parses as much as possible and returns the
/// unparsed remainder as the `tail`.
///
/// # Example
///
/// ```
/// use dicelet_core::{roll, RollOptions};
/// let result = roll("4d6k3", RollOptions::default()).unwrap();
/// assert!(result.consumed.contains("4d6k3"));
/// ```
pub fn roll(source: &str, options: RollOptions) -> Result<RollOutput> {
    let parse_result = Parser::parse(source);

    let ast = match parse_result.ast {
        Some(ast) => ast,
        None => {
            return Ok(RollOutput {
                consumed: String::new(),
                tail: parse_result.tail,
                summary: String::new(),
                detail: String::new(),
                full: String::new(),
                is_set: false,
                values: Vec::new(),
            });
        }
    };

    let mut rng = match options.seed {
        Some(seed) => Xoroshiro128StarStar::from_seed(seed),
        None => Xoroshiro128StarStar::from_entropy(),
    };

    let eval_value = evaluate(&ast, &mut rng)?;

    let summary = eval_value.summary();
    let detail = if options.show_detail {
        eval_value.detail()
    } else {
        String::new()
    };
    let full = if options.show_detail {
        eval_value.full()
    } else {
        eval_value.summary()
    };

    Ok(RollOutput {
        consumed: parse_result.consumed,
        tail: parse_result.tail,
        summary,
        detail,
        full,
        is_set: eval_value.is_set(),
        values: eval_value.values().iter().map(|n| n.to_f64()).collect(),
    })
}

/// Evaluate an AST expression with the given RNG.
pub fn evaluate(expr: &Expr, rng: &mut dyn Rng) -> Result<EvalValue> {
    eval::evaluate(expr, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_basic() {
        let result = roll("d20", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        assert!(result.consumed.contains("d20"));
        assert!(!result.summary.is_empty());
    }

    #[test]
    fn test_roll_with_detail() {
        let result = roll("4d6", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        assert!(result.detail.starts_with("["));
        assert!(result.detail.ends_with("]"));
    }

    #[test]
    fn test_roll_no_detail() {
        let result = roll("4d6", RollOptions { show_detail: false, seed: Some(42) }).unwrap();
        assert!(result.detail.is_empty());
    }

    #[test]
    fn test_roll_set() {
        let result = roll("6#4d6k3", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        assert!(result.is_set);
        assert!(result.summary.starts_with("{"));
        assert!(result.summary.ends_with("}"));
    }

    #[test]
    fn test_roll_strtol_recovery() {
        let result = roll("d20 + (d4+ 测试", RollOptions::default()).unwrap();
        assert_eq!(result.consumed, "d20");
        assert_eq!(result.tail, "+ (d4+ 测试");
    }

    #[test]
    fn test_roll_complex() {
        let result = roll(
            "(((4d6+3)/2+2d20)+4*1d6)*150%",
            RollOptions { seed: Some(42), ..Default::default() },
        ).unwrap();
        assert!(!result.summary.is_empty());
    }

    #[test]
    fn test_roll_set_operation() {
        let result = roll(
            "4#d20-{1,2,3,4}",
            RollOptions { seed: Some(42), ..Default::default() },
        ).unwrap();
        assert!(result.is_set);
    }

    #[test]
    fn test_roll_const() {
        let result = roll("2+3*4", RollOptions::default()).unwrap();
        assert_eq!(result.summary, "14");
    }

    #[test]
    fn test_roll_percentage() {
        let result = roll("10*150%", RollOptions::default()).unwrap();
        assert_eq!(result.summary, "15");
    }

    // --- New syntax: comparison ---

    #[test]
    fn test_roll_comparison_greater() {
        // 4d6>3 with seed 42 — count of dice > 3
        let result = roll("4d6>3", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        assert!(result.detail.starts_with("<"));
        assert!(result.detail.ends_with(">"));
        let count: i64 = result.summary.parse().unwrap();
        assert!(count >= 0 && count <= 4);
    }

    #[test]
    fn test_roll_comparison_less() {
        let result = roll("4d6<3", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        let count: i64 = result.summary.parse().unwrap();
        assert!(count >= 0 && count <= 4);
    }

    // --- New syntax: bonus dice ---

    #[test]
    fn test_roll_bonus() {
        // Use threshold 3 to ensure bonuses trigger with seed 42
        let result = roll("2d6b3", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        assert!(result.detail.contains('!'));
    }

    #[test]
    fn test_roll_bonus_with_keep() {
        let result = roll("2d6b5k3", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        assert!(result.detail.starts_with("["));
        assert!(!result.summary.is_empty());
    }

    // --- Combined: bonus + keep + comparison ---

    #[test]
    fn test_roll_bonus_keep_comparison() {
        let result = roll("2d6b5k3>3", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        assert!(result.detail.starts_with("<"));
        assert!(result.detail.ends_with(">"));
        assert!(!result.summary.is_empty());
    }

    // --- Comparison in larger expressions ---

    #[test]
    fn test_roll_comparison_in_binop() {
        // 4d6>3 + 2: count dice > 3, then add 2
        let comp = roll("4d6>3", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        let result = roll("4d6>3+2", RollOptions { seed: Some(42), ..Default::default() }).unwrap();
        let expected: i64 = comp.summary.parse::<i64>().unwrap() + 2;
        assert_eq!(result.summary.parse::<i64>().unwrap(), expected);
    }

    #[test]
    fn test_roll_bonus_no_crash() {
        // Bonus alone (without dice) should not crash — just be tail
        let result = roll("b5", RollOptions::default()).unwrap();
        assert_eq!(result.consumed, "");
        assert_eq!(result.tail, "b5");
    }
}
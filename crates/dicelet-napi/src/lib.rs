#![deny(clippy::all)]

use dicelet_core::{roll as core_roll, Parser, RollOptions};
use napi_derive::napi;

/// Options for rolling dice.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct Options {
  /// Whether to show detailed roll results. Default: true.
  pub show_detail: Option<bool>,
  /// Optional random seed for deterministic results (testing).
  pub seed: Option<i64>,
}

impl Default for Options {
  fn default() -> Self {
    Self {
      show_detail: Some(true),
      seed: None,
    }
  }
}

/// The result of a roll operation.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RollResult {
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
/// @example
/// ```typescript
/// const result = roll("4d6k3");
/// console.log(result.full); // e.g. "[5 + 3 + 1* + 6] = 14"
/// ```
#[napi]
pub fn roll(expression: String, options: Option<Options>) -> napi::Result<RollResult> {
  let opts = options.unwrap_or_default();
  let core_opts = RollOptions {
    show_detail: opts.show_detail.unwrap_or(true),
    seed: opts.seed.map(|s| s as u64),
  };

  match core_roll(&expression, core_opts) {
    Ok(result) => Ok(RollResult {
      consumed: result.consumed,
      tail: result.tail,
      summary: result.summary,
      detail: result.detail,
      full: result.full,
      is_set: result.is_set,
      values: result.values,
    }),
    Err(e) => Err(napi::Error::from_reason(format!("{}", e))),
  }
}

/// The result of parsing (without rolling).
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ParseOutput {
  /// Whether parsing succeeded.
  pub success: bool,
  /// The source text that was successfully consumed.
  pub consumed: String,
  /// The remaining unparsed text (tail).
  pub tail: String,
}

/// Parse a dicelet expression without rolling.
///
/// Returns the consumed portion and the remaining tail.
/// Uses strtol-style fault-tolerant parsing.
///
/// @example
/// ```typescript
/// const result = parse("d20 + (d4+ 测试");
/// console.log(result.success); // true
/// console.log(result.consumed); // "d20"
/// console.log(result.tail); // "+ (d4+ 测试"
/// ```
#[napi]
pub fn parse(expression: String) -> napi::Result<ParseOutput> {
  let result = Parser::parse(&expression);
  Ok(ParseOutput {
    success: result.ast.is_some(),
    consumed: result.consumed,
    tail: result.tail,
  })
}
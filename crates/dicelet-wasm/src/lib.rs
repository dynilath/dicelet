//! WASM bindings for dicelet-core.
//!
//! This crate provides WebAssembly bindings for the dicelet dice expression
//! engine, allowing it to be used in browser environments.
//!
//! ## Usage (JavaScript)
//!
//! ```javascript
//! import { roll, parse } from '@dynilath/dicelet/wasm';
//!
//! const result = roll('4d6k3');
//! console.log(result.full); // e.g. "[5 + 3 + 1* + 6] = 14"
//! ```

use dicelet_core::{roll as core_roll, Parser, RollOptions};
use js_sys::Date;
use wasm_bindgen::prelude::*;

/// Options for rolling dice.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Options {
    show_detail: bool,
    seed: Option<u64>,
}

#[wasm_bindgen]
impl Options {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            show_detail: true,
            seed: None,
        }
    }

    /// Whether to show detailed roll results. Default: true.
    #[wasm_bindgen(getter)]
    pub fn show_detail(&self) -> bool {
        self.show_detail
    }

    #[wasm_bindgen(setter)]
    pub fn set_show_detail(&mut self, val: bool) {
        self.show_detail = val;
    }

    /// Optional random seed for deterministic results (testing).
    #[wasm_bindgen(getter)]
    pub fn seed(&self) -> Option<f64> {
        self.seed.map(|s| s as f64)
    }

    #[wasm_bindgen(setter)]
    pub fn set_seed(&mut self, val: Option<f64>) {
        self.seed = val.map(|s| s as u64);
    }
}

impl Default for Options {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of a roll operation.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct RollResult {
    consumed: String,
    tail: String,
    summary: String,
    detail: String,
    full: String,
    is_set: bool,
    values: Vec<f64>,
}

#[wasm_bindgen]
impl RollResult {
    #[wasm_bindgen(getter)]
    pub fn consumed(&self) -> String {
        self.consumed.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn tail(&self) -> String {
        self.tail.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn summary(&self) -> String {
        self.summary.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn detail(&self) -> String {
        self.detail.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn full(&self) -> String {
        self.full.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn is_set(&self) -> bool {
        self.is_set
    }

    #[wasm_bindgen(getter)]
    pub fn values(&self) -> Vec<f64> {
        self.values.clone()
    }
}

/// The result of parsing (without rolling).
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct ParseOutput {
    success: bool,
    consumed: String,
    tail: String,
}

#[wasm_bindgen]
impl ParseOutput {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn consumed(&self) -> String {
        self.consumed.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn tail(&self) -> String {
        self.tail.clone()
    }
}

/// Get entropy from the JavaScript environment (Date.now()).
fn js_entropy() -> u64 {
    let now = Date::now();
    // Convert milliseconds to nanoseconds and add some extra entropy
    (now as u64).wrapping_mul(1_000_000).wrapping_add(0x9E3779B97F4A7C15)
}

/// Parse and evaluate a dicelet expression.
///
/// @param expression - The dicelet expression to roll
/// @param options - Optional configuration (showDetail, seed)
/// @returns The roll result
#[wasm_bindgen]
pub fn roll(expression: &str, options: Option<Options>) -> Result<RollResult, JsValue> {
    let opts = options.unwrap_or_default();
    let seed = opts.seed().map(|s| s as u64);

    // If no seed provided, use JS entropy
    let core_opts = RollOptions {
        show_detail: opts.show_detail,
        seed: seed.or_else(|| Some(js_entropy())),
    };

    match core_roll(expression, core_opts) {
        Ok(result) => Ok(RollResult {
            consumed: result.consumed,
            tail: result.tail,
            summary: result.summary,
            detail: result.detail,
            full: result.full,
            is_set: result.is_set,
            values: result.values,
        }),
        Err(e) => Err(JsValue::from_str(&format!("{}", e))),
    }
}

/// Parse a dicelet expression without rolling.
///
/// @param expression - The dicelet expression to parse
/// @returns The parse result with consumed and tail
#[wasm_bindgen]
pub fn parse(expression: &str) -> ParseOutput {
    let result = Parser::parse(expression);
    ParseOutput {
        success: result.ast.is_some(),
        consumed: result.consumed,
        tail: result.tail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_basic() {
        let result = roll("4d6k3", None).unwrap();
        assert!(result.summary().len() > 0);
        assert!(result.full().contains("="));
        assert!(!result.is_set());
    }

    #[test]
    fn test_roll_with_seed() {
        let mut opts = Options::new();
        opts.set_seed(Some(42.0));
        let result1 = roll("4d6", Some(opts.clone())).unwrap();
        let result2 = roll("4d6", Some(opts)).unwrap();
        assert_eq!(result1.summary(), result2.summary());
    }

    #[test]
    fn test_roll_set() {
        let result = roll("6#4d6k3", None).unwrap();
        assert!(result.is_set());
        assert!(result.summary().starts_with("{"));
    }

    #[test]
    fn test_parse_recovery() {
        let result = parse("d20 + (d4+ 测试");
        assert!(result.success());
        assert_eq!(result.consumed(), "d20");
        assert_eq!(result.tail(), "+ (d4+ 测试");
    }

    #[test]
    fn test_no_detail() {
        let mut opts = Options::new();
        opts.set_show_detail(false);
        let result = roll("4d6", Some(opts)).unwrap();
        assert!(result.detail().is_empty());
    }

    #[test]
    fn test_const_expr() {
        let result = roll("2+3*4", None).unwrap();
        assert_eq!(result.summary(), "14");
    }
}
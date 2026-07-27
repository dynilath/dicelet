#![deny(clippy::all)]

use dicelet_core::{roll as core_roll, Parser, RollOptions};
use pyo3::prelude::*;

/// The result of a roll operation.
#[pyclass(get_all)]
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

#[pymethods]
impl RollResult {
    fn __repr__(&self) -> String {
        format!(
            "RollResult(consumed={:?}, tail={:?}, full={:?}, is_set={}, values={:?})",
            self.consumed, self.tail, self.full, self.is_set, self.values
        )
    }

    fn __str__(&self) -> String {
        self.full.clone()
    }
}

/// The result of parsing (without rolling).
#[pyclass(get_all)]
#[derive(Debug, Clone)]
pub struct ParseOutput {
    /// Whether parsing succeeded.
    pub success: bool,
    /// The source text that was successfully consumed.
    pub consumed: String,
    /// The remaining unparsed text (tail).
    pub tail: String,
}

#[pymethods]
impl ParseOutput {
    fn __repr__(&self) -> String {
        format!(
            "ParseOutput(success={}, consumed={:?}, tail={:?})",
            self.success, self.consumed, self.tail
        )
    }
}

/// Parse and evaluate a dicelet expression.
///
/// This function combines parsing and evaluation into one step.
/// It uses strtol-style fault-tolerant parsing: if the input contains
/// invalid syntax, it parses as much as possible and returns the
/// unparsed remainder as the ``tail``.
///
/// Args:
///     expression: The dicelet expression string (e.g. "4d6k3").
///     show_detail: Whether to show detailed roll results (default True).
///     seed: Optional random seed for deterministic results (testing).
///
/// Returns:
///     RollResult with the evaluation results.
///
/// Examples:
///     >>> import dicelet
///     >>> result = dicelet.roll("4d6k3")
///     >>> print(result.full)
///     [5 + 3 + 1* + 6] = 14
#[pyfunction]
#[pyo3(signature = (expression, show_detail=true, seed=None))]
fn roll(expression: &str, show_detail: bool, seed: Option<u64>) -> PyResult<RollResult> {
    let opts = RollOptions {
        show_detail,
        seed,
    };

    match core_roll(expression, opts) {
        Ok(result) => Ok(RollResult {
            consumed: result.consumed,
            tail: result.tail,
            summary: result.summary,
            detail: result.detail,
            full: result.full,
            is_set: result.is_set,
            values: result.values,
        }),
        Err(e) => Err(pyo3::exceptions::PyValueError::new_err(format!("{}", e))),
    }
}

/// Parse a dicelet expression without rolling.
///
/// Returns the consumed portion and the remaining tail.
/// Uses strtol-style fault-tolerant parsing.
///
/// Args:
///     expression: The dicelet expression string.
///
/// Returns:
///     ParseOutput with the parsing results.
///
/// Examples:
///     >>> import dicelet
///     >>> result = dicelet.parse("d20 + (d4+ test")
///     >>> print(result.success)
///     True
///     >>> print(result.consumed)
///     d20
#[pyfunction]
fn parse(expression: &str) -> ParseOutput {
    let result = Parser::parse(expression);
    ParseOutput {
        success: result.ast.is_some(),
        consumed: result.consumed,
        tail: result.tail,
    }
}

/// Dicelet: dice expression parsing and evaluation engine.
///
/// A Python module providing dice expression parsing and evaluation
/// backed by a high-performance Rust implementation.
#[pymodule]
fn dicelet(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RollResult>()?;
    m.add_class::<ParseOutput>()?;
    m.add_function(wrap_pyfunction!(roll, m)?)?;
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    Ok(())
}

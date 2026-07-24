use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// A numeric value that supports integers, decimals, and percentages.
///
/// This mirrors the original C++ `number` type, which can represent
/// `42`, `3.14`, or `150%` (stored internally as `1.5`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Int(i64),
    Decimal(f64),
    Percent(f64),
}

impl Number {
    /// Parse a number from a string slice. Supports integers, decimals,
    /// and percentages (e.g. `"42"`, `"3.14"`, `"150%"`).
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        if s.ends_with('%') {
            let inner = &s[..s.len() - 1];
            let val: f64 = inner.parse().ok()?;
            // 150% → 1.5
            return Some(Number::Percent(val / 100.0));
        }

        // Try integer first
        if let Ok(i) = s.parse::<i64>() {
            return Some(Number::Int(i));
        }

        // Fall back to float
        let f: f64 = s.parse().ok()?;
        Some(Number::Decimal(f))
    }

    /// Convert to f64 for computation.
    pub fn to_f64(self) -> f64 {
        match self {
            Number::Int(i) => i as f64,
            Number::Decimal(f) => f,
            Number::Percent(f) => f,
        }
    }

    /// Returns true if the value is a positive integer (>= 1).
    pub fn is_positive_int(self) -> bool {
        match self {
            Number::Int(i) => i >= 1,
            Number::Decimal(f) => f >= 1.0 && f.fract() == 0.0,
            Number::Percent(f) => f >= 1.0 && f.fract() == 0.0,
        }
    }

    /// Convert to i64, rounding if necessary.
    pub fn to_i64(self) -> i64 {
        match self {
            Number::Int(i) => i,
            Number::Decimal(f) => f as i64,
            Number::Percent(f) => f as i64,
        }
    }

    /// Display string for the number in its original form (e.g. `150%` stays `150%`).
    pub fn to_display_string(self) -> String {
        match self {
            Number::Int(i) => i.to_string(),
            Number::Decimal(f) => {
                // Display integers without decimal point
                if f.fract() == 0.0 && f.abs() < 1e15 {
                    format!("{}", f as i64)
                } else {
                    format!("{}", f)
                }
            }
            Number::Percent(f) => {
                let pct = f * 100.0;
                if pct.fract() == 0.0 && pct.abs() < 1e15 {
                    format!("{}%", pct as i64)
                } else {
                    format!("{}%", pct)
                }
            }
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Computation display: always show the numeric value
        match self {
            Number::Int(i) => write!(f, "{}", i),
            Number::Decimal(d) => {
                if d.fract() == 0.0 && d.abs() < 1e15 {
                    write!(f, "{}", *d as i64)
                } else {
                    write!(f, "{}", d)
                }
            }
            Number::Percent(p) => {
                let val = *p;
                if val.fract() == 0.0 && val.abs() < 1e15 {
                    write!(f, "{}", val as i64)
                } else {
                    write!(f, "{}", val)
                }
            }
        }
    }
}

impl Add for Number {
    type Output = Number;
    fn add(self, other: Number) -> Number {
        // If both are integers, preserve integer type
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a + b),
            (a, b) => Number::Decimal(a.to_f64() + b.to_f64()),
        }
    }
}

impl Sub for Number {
    type Output = Number;
    fn sub(self, other: Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a - b),
            (a, b) => Number::Decimal(a.to_f64() - b.to_f64()),
        }
    }
}

impl Mul for Number {
    type Output = Number;
    fn mul(self, other: Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a * b),
            (a, b) => Number::Decimal(a.to_f64() * b.to_f64()),
        }
    }
}

impl Div for Number {
    type Output = Number;
    fn div(self, other: Number) -> Number {
        let denom = other.to_f64();
        if denom == 0.0 {
            // Division by zero will be caught by the evaluator
            return Number::Decimal(f64::INFINITY);
        }
        match (self, other) {
            (Number::Int(a), Number::Int(b)) if b != 0 => {
                if a % b == 0 {
                    Number::Int(a / b)
                } else {
                    Number::Decimal(a as f64 / b as f64)
                }
            }
            (a, b) => Number::Decimal(a.to_f64() / b.to_f64()),
        }
    }
}

impl Neg for Number {
    type Output = Number;
    fn neg(self) -> Number {
        match self {
            Number::Int(i) => Number::Int(-i),
            Number::Decimal(f) => Number::Decimal(-f),
            Number::Percent(f) => Number::Percent(-f),
        }
    }
}
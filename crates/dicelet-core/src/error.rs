use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum DiceletError {
    #[error("dice count exceeds maximum ({0}), max is {1}")]
    DiceCountExceed(i64, i64),

    #[error("dice face exceeds maximum ({0}), max is {1}")]
    DiceFaceExceed(i64, i64),

    #[error("repeat count exceeds maximum ({0}), max is {1}")]
    UnitCountExceed(i64, i64),

    #[error("dice count, face, or repeat must be a positive integer")]
    InvalidDice,

    #[error("division by zero")]
    DivZero,

    #[error("numeric value out of range")]
    OutOfRange,

    #[error("parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, DiceletError>;
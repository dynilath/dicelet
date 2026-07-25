/// Token types for the dicelet lexer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// End of input
    Eof,
    /// Numeric literal (integer, decimal, or percentage)
    Number,
    /// `d` / `D` keyword (dice)
    Dice,
    /// `k` / `K` keyword (keep high)
    KeepHigh,
    /// `kl` / `kL` / `Kl` / `KL` keyword (keep low)
    KeepLow,
    /// `b` / `B` keyword (bonus dice)
    Bonus,
    /// `>` (greater than)
    GreaterThan,
    /// `>=` (greater than or equal)
    GreaterEqual,
    /// `<` (less than)
    LessThan,
    /// `<=` (less than or equal)
    LessEqual,
    /// `!=` (not equal)
    NotEqual,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Asterisk,
    /// `/`
    Slash,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `,`
    Comma,
    /// `#`
    Sharp,
}

impl TokenKind {
    /// Returns true if this token is a multiplicative operator (`*` or `/`).
    pub fn is_mul_op(self) -> bool {
        matches!(self, TokenKind::Asterisk | TokenKind::Slash)
    }

    /// Returns true if this token is an additive operator (`+` or `-`).
    pub fn is_add_op(self) -> bool {
        matches!(self, TokenKind::Plus | TokenKind::Minus)
    }
}

/// A single token produced by the lexer.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    /// The raw text of the token
    pub text: String,
    /// Byte offset in the original source where this token starts
    pub start: usize,
    /// Byte offset one past the end of this token
    pub end: usize,
}

impl Token {
    pub fn new(kind: TokenKind, text: String, start: usize, end: usize) -> Self {
        Self { kind, text, start, end }
    }

    pub fn eof(pos: usize) -> Self {
        Self {
            kind: TokenKind::Eof,
            text: String::new(),
            start: pos,
            end: pos,
        }
    }
}
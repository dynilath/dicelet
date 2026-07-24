use crate::number::Number;

/// Binary operators supported in dicelet expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
        }
    }
}

/// Keep mode for dice selection (k = keep high, kl = keep low).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeepMode {
    /// Keep the N highest values
    High(i64),
    /// Keep the N lowest values
    Low(i64),
}

/// A dice expression: `[count]d[faces][k|kl N]`
#[derive(Debug, Clone)]
pub struct DiceExpr {
    pub count: Number,
    pub faces: Number,
    pub keep: Option<KeepMode>,
}

/// Expression level: determines evaluation semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Pure constant — no dice, single result.
    Const,
    /// Contains dice — single result with rolling.
    Rand,
    /// Contains `#` or `{}` — multiple independent results.
    Dicelet,
}

/// Unified expression AST node.
///
/// The three grammar tiers (const_expr, rand_expr, dicelet_expr) are represented
/// by the `Level` of each node, computed via [`Expr::level`].
#[derive(Debug, Clone)]
pub enum Expr {
    /// A number literal
    Number(Number),
    /// Unary negation: `-expr`
    Neg(Box<Expr>),
    /// Binary operation: `a op b`
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    /// A dice roll: `count d faces [k|kl N]`
    Dice(DiceExpr),
    /// Parenthesized expression: `(expr)` — preserved for display
    Paren(Box<Expr>),
    /// Repeat: `N # expr` — roll `expr` N times independently
    Repeat {
        times: Number,
        inner: Box<Expr>,
    },
    /// Set: `{expr, expr, ...}` — a collection of independent results
    Set(Vec<Expr>),
}

impl Expr {
    /// Returns the expression level, which determines evaluation semantics.
    pub fn level(&self) -> Level {
        match self {
            Expr::Number(_) => Level::Const,
            Expr::Neg(e) => e.level(),
            Expr::BinOp(_, a, b) => a.level().max(b.level()),
            Expr::Dice(_) => Level::Rand,
            Expr::Paren(e) => e.level(),
            Expr::Repeat { .. } => Level::Dicelet,
            Expr::Set(_) => Level::Dicelet,
        }
    }

    /// Returns true if this expression produces multiple results.
    pub fn is_set(&self) -> bool {
        self.level() == Level::Dicelet
    }
}
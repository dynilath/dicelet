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

/// Comparison operator for counting dice matching a condition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonOp {
    /// `> N` — strictly greater than
    Greater(Number),
    /// `>= N` — greater than or equal
    GreaterEqual(Number),
    /// `< N` — strictly less than
    Less(Number),
    /// `<= N` — less than or equal
    LessEqual(Number),
    /// `!= N` — not equal
    NotEqual(Number),
}

impl ComparisonOp {
    /// The threshold value being compared against.
    pub fn threshold(&self) -> Number {
        match self {
            ComparisonOp::Greater(n)
            | ComparisonOp::GreaterEqual(n)
            | ComparisonOp::Less(n)
            | ComparisonOp::LessEqual(n)
            | ComparisonOp::NotEqual(n) => *n,
        }
    }

    /// The operator symbol as a string.
    pub fn symbol(&self) -> &'static str {
        match self {
            ComparisonOp::Greater(_) => ">",
            ComparisonOp::GreaterEqual(_) => ">=",
            ComparisonOp::Less(_) => "<",
            ComparisonOp::LessEqual(_) => "<=",
            ComparisonOp::NotEqual(_) => "!=",
        }
    }
}

/// A dice expression: `[count]d[faces][bN][k|kl N][>N|>=N|<N|<=N|!=N]`
#[derive(Debug, Clone)]
pub struct DiceExpr {
    pub count: Number,
    pub faces: Number,
    pub keep: Option<KeepMode>,
    /// Bonus dice: if a die result >= this threshold, roll an extra die (recursive)
    pub bonus: Option<Number>,
    /// Comparison: count kept dice matching this condition instead of summing
    pub comparison: Option<ComparisonOp>,
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
pub mod ast;

use crate::lexer::scanner;
use crate::lexer::token::{Token, TokenKind};
use crate::lexer::Lexer;
use crate::number::Number;

use ast::{BinOp, ComparisonOp, DiceExpr, Expr, KeepMode};

/// Result of parsing a dicelet expression.
pub struct ParseResult {
    /// The parsed AST, or None if nothing could be parsed.
    pub ast: Option<Expr>,
    /// The source text that was successfully consumed.
    pub consumed: String,
    /// The remaining unparsed text (parser tail + scanner tail).
    pub tail: String,
}

/// Recursive descent parser with strtol-style fault-tolerant recovery.
///
/// The parser tries to parse as much of the input as possible. When it
/// encounters a token that cannot continue the current expression, it
/// returns the last successfully parsed expression and treats the
/// remaining input as the "tail".
pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    valid_source: &'a str,
    scanner_tail: &'a str,
}

impl<'a> Parser<'a> {
    /// Parse a source string into an expression with fault-tolerant recovery.
    pub fn parse(source: &'a str) -> ParseResult {
        // 1. Pre-scan: check parenthesis/brace matching, truncate if needed
        let scan_result = scanner::scan(source, true);

        // 2. Tokenize the valid portion
        let tokens = Lexer::new(scan_result.valid_source).tokenize();

        // 3. Parse
        let mut parser = Self {
            tokens,
            pos: 0,
            valid_source: scan_result.valid_source,
            scanner_tail: scan_result.tail,
        };

        let ast = parser.parse_additive();

        // 4. Compute consumed and tail
        let (consumed, tail) = parser.compute_result();

        ParseResult { ast, consumed, tail }
    }

    /// Compute the consumed source text and the tail.
    fn compute_result(&self) -> (String, String) {
        // The tail starts from the first unconsumed token's source position
        let tail_start = if self.pos < self.tokens.len() {
            self.tokens[self.pos].start
        } else {
            self.valid_source.len()
        };

        let consumed = self.valid_source[..tail_start].trim_end().to_string();

        // Parser tail: remaining valid source (skip leading spaces) + scanner tail
        let parser_tail = self.valid_source[tail_start..].trim_start();
        let tail = format!("{}{}", parser_tail, self.scanner_tail);

        (consumed, tail)
    }

    // --- Token access helpers ---

    fn peek_kind(&self) -> TokenKind {
        self.tokens[self.pos].kind
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn save_pos(&self) -> usize {
        self.pos
    }

    fn restore_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    // --- Parsing methods (precedence climbing) ---

    /// Parse an additive expression: `mul (('+' | '-') mul)*`
    fn parse_additive(&mut self) -> Option<Expr> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let kind = self.peek_kind();
            let op = match kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };

            // Save position before consuming the operator
            let saved = self.save_pos();
            self.advance(); // consume operator

            if let Some(right) = self.parse_multiplicative() {
                left = Expr::BinOp(op, Box::new(left), Box::new(right));
            } else {
                // Right side failed to parse; restore and return left
                self.restore_pos(saved);
                break;
            }
        }

        Some(left)
    }

    /// Parse a multiplicative expression: `unary (('*' | '/') unary)*`
    fn parse_multiplicative(&mut self) -> Option<Expr> {
        let mut left = self.parse_unary()?;

        loop {
            let kind = self.peek_kind();
            let op = match kind {
                TokenKind::Asterisk => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => break,
            };

            let saved = self.save_pos();
            self.advance(); // consume operator

            if let Some(right) = self.parse_unary() {
                left = Expr::BinOp(op, Box::new(left), Box::new(right));
            } else {
                self.restore_pos(saved);
                break;
            }
        }

        Some(left)
    }

    /// Parse a unary expression: `'-' unary | primary`
    fn parse_unary(&mut self) -> Option<Expr> {
        if self.peek_kind() == TokenKind::Minus {
            let saved = self.save_pos();
            self.advance(); // consume '-'

            if let Some(inner) = self.parse_unary() {
                return Some(Expr::Neg(Box::new(inner)));
            }
            self.restore_pos(saved);
            return None;
        }

        self.parse_primary()
    }

    /// Parse a primary expression: number, dice, parens, repeat, set.
    fn parse_primary(&mut self) -> Option<Expr> {
        match self.peek_kind() {
            TokenKind::Number => self.parse_number_or_dice_or_repeat(),
            TokenKind::Dice => self.parse_dice_no_count(),
            TokenKind::LParen => self.parse_paren(),
            TokenKind::LBrace => self.parse_set(),
            _ => None,
        }
    }

    /// Parse a number, which might be followed by `d` (dice) or `#` (repeat).
    fn parse_number_or_dice_or_repeat(&mut self) -> Option<Expr> {
        let num_tok = self.advance(); // consume number
        let num = Number::parse(&num_tok.text)?;

        match self.peek_kind() {
            TokenKind::Dice => {
                // number d faces [k|kl N]
                self.parse_dice_with_count(num)
            }
            TokenKind::Sharp => {
                // number # unit
                self.parse_repeat(num)
            }
            _ => Some(Expr::Number(num)),
        }
    }

    /// Parse a dice expression with an explicit count: `count d faces [bN][k|kl N][>N|>=N|<N|<=N|!=N]`
    fn parse_dice_with_count(&mut self, count: Number) -> Option<Expr> {
        self.advance(); // consume 'd'

        let faces_tok = self.advance();
        let faces = Number::parse(&faces_tok.text)?;

        // Parse optional modifiers in fixed order: bonus → keep → comparison
        let bonus = self.parse_bonus_mode();
        let keep = self.parse_keep_mode();
        let comparison = self.parse_comparison();

        Some(Expr::Dice(DiceExpr {
            count,
            faces,
            keep,
            bonus,
            comparison,
        }))
    }

    /// Parse a dice expression with implicit count 1: `d faces [bN][k|kl N][>N|>=N|<N|<=N|!=N]`
    fn parse_dice_no_count(&mut self) -> Option<Expr> {
        self.advance(); // consume 'd'

        let faces_tok = self.advance();
        let faces = Number::parse(&faces_tok.text)?;

        // Parse optional modifiers in fixed order: bonus → keep → comparison
        let bonus = self.parse_bonus_mode();
        let keep = self.parse_keep_mode();
        let comparison = self.parse_comparison();

        Some(Expr::Dice(DiceExpr {
            count: Number::Int(1),
            faces,
            keep,
            bonus,
            comparison,
        }))
    }

    /// Parse optional keep mode: `k N` or `kl N`
    fn parse_keep_mode(&mut self) -> Option<KeepMode> {
        match self.peek_kind() {
            TokenKind::KeepHigh => {
                self.advance(); // consume 'k'
                let n_tok = self.advance();
                let n = Number::parse(&n_tok.text)?;
                Some(KeepMode::High(n.to_i64()))
            }
            TokenKind::KeepLow => {
                self.advance(); // consume 'kl'
                let n_tok = self.advance();
                let n = Number::parse(&n_tok.text)?;
                Some(KeepMode::Low(n.to_i64()))
            }
            _ => None,
        }
    }

    /// Parse optional bonus mode: `b N` 
    fn parse_bonus_mode(&mut self) -> Option<Number> {
        if self.peek_kind() == TokenKind::Bonus {
            self.advance(); // consume 'b'
            let n_tok = self.advance();
            let n = Number::parse(&n_tok.text)?;
            Some(n)
        } else {
            None
        }
    }

    /// Parse optional comparison: `>N | >=N | <N | <=N | !=N`
    fn parse_comparison(&mut self) -> Option<ComparisonOp> {
        let kind = self.peek_kind();
        let op = match kind {
            TokenKind::GreaterThan => ComparisonOp::Greater,
            TokenKind::GreaterEqual => ComparisonOp::GreaterEqual,
            TokenKind::LessThan => ComparisonOp::Less,
            TokenKind::LessEqual => ComparisonOp::LessEqual,
            TokenKind::NotEqual => ComparisonOp::NotEqual,
            _ => return None,
        };

        self.advance(); // consume the comparison operator token
        let n_tok = self.advance();
        let n = Number::parse(&n_tok.text)?;
        Some(op(n))
    }

    /// Parse a repeat expression: `count # unit`
    fn parse_repeat(&mut self, times: Number) -> Option<Expr> {
        let saved = self.save_pos();
        self.advance(); // consume '#'

        // Right side: a unit (number, dice, or parenthesized expression)
        if let Some(inner) = self.parse_unit() {
            Some(Expr::Repeat {
                times,
                inner: Box::new(inner),
            })
        } else {
            // Failed to parse the right side; restore and return just the number
            self.restore_pos(saved);
            Some(Expr::Number(times))
        }
    }

    /// Parse a "unit" — the right-hand side of `#`.
    /// This is a number, dice, or parenthesized expression (but NOT a set).
    fn parse_unit(&mut self) -> Option<Expr> {
        match self.peek_kind() {
            TokenKind::Number => {
                let num_tok = self.advance();
                let num = Number::parse(&num_tok.text)?;
                if self.peek_kind() == TokenKind::Dice {
                    self.parse_dice_with_count(num)
                } else {
                    Some(Expr::Number(num))
                }
            }
            TokenKind::Dice => self.parse_dice_no_count(),
            TokenKind::LParen => self.parse_paren(),
            _ => None,
        }
    }

    /// Parse a parenthesized expression: `( expr )`
    fn parse_paren(&mut self) -> Option<Expr> {
        let saved = self.save_pos();
        self.advance(); // consume '('

        let inner = self.parse_additive();

        if inner.is_none() {
            self.restore_pos(saved);
            return None;
        }

        let inner = inner.unwrap();

        if self.peek_kind() == TokenKind::RParen {
            self.advance(); // consume ')'
            Some(Expr::Paren(Box::new(inner)))
        } else {
            // Missing closing paren — treat as failure
            self.restore_pos(saved);
            None
        }
    }

    /// Parse a set expression: `{ expr, expr, ... }`
    fn parse_set(&mut self) -> Option<Expr> {
        let saved = self.save_pos();
        self.advance(); // consume '{'

        let mut elements = Vec::new();

        // Parse first element
        let first = self.parse_additive();
        if first.is_none() {
            self.restore_pos(saved);
            return None;
        }
        elements.push(first.unwrap());

        // Parse remaining elements
        loop {
            if self.peek_kind() != TokenKind::Comma {
                break;
            }
            self.advance(); // consume ','

            let elem = self.parse_additive();
            if elem.is_none() {
                // Failed to parse element after comma; stop here
                // but don't fail the whole set — just use what we have
                break;
            }
            elements.push(elem.unwrap());
        }

        if self.peek_kind() == TokenKind::RBrace {
            self.advance(); // consume '}'
            Some(Expr::Set(elements))
        } else {
            // Missing closing brace — treat as failure
            self.restore_pos(saved);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_simple(s: &str) -> Option<Expr> {
        let result = Parser::parse(s);
        result.ast
    }

    #[test]
    fn test_number() {
        let ast = parse_simple("42").unwrap();
        assert!(matches!(ast, Expr::Number(_)));
    }

    #[test]
    fn test_dice() {
        let ast = parse_simple("d20").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.count, Number::Int(1));
                assert_eq!(d.faces, Number::Int(20));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_with_count() {
        let ast = parse_simple("4d6").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.count, Number::Int(4));
                assert_eq!(d.faces, Number::Int(6));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_keep_high() {
        let ast = parse_simple("4d6k3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.keep, Some(KeepMode::High(3)));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_keep_low() {
        let ast = parse_simple("2d20kl1").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.keep, Some(KeepMode::Low(1)));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_addition() {
        let ast = parse_simple("1d20+2d6+4").unwrap();
        assert!(matches!(ast, Expr::BinOp(BinOp::Add, _, _)));
    }

    #[test]
    fn test_complex_expr() {
        let ast = parse_simple("(((4d6+3)/2+2d20)+4*1d6)*150%");
        assert!(ast.is_some());
    }

    #[test]
    fn test_repeat() {
        let ast = parse_simple("6#4d6k3").unwrap();
        assert!(matches!(ast, Expr::Repeat { .. }));
    }

    #[test]
    fn test_set() {
        let ast = parse_simple("{4d6,3d6,2d6,1d6}").unwrap();
        assert!(matches!(ast, Expr::Set(_)));
    }

    #[test]
    fn test_repeat_with_subtraction() {
        let ast = parse_simple("4#d20-{1,2,3,4}").unwrap();
        assert!(ast.is_set());
    }

    #[test]
    fn test_strtol_recovery_paren() {
        let result = Parser::parse("d20 + (d4+ 测试");
        assert!(result.ast.is_some());
        assert_eq!(result.consumed, "d20");
        assert_eq!(result.tail, "+ (d4+ 测试");
    }

    #[test]
    fn test_strtol_recovery_trailing_op() {
        let result = Parser::parse("4d6 + ");
        assert!(result.ast.is_some());
        assert_eq!(result.consumed, "4d6");
        // The tail includes the unconsumed operator and trailing space
        assert_eq!(result.tail, "+ ");
    }

    #[test]
    fn test_strtol_recovery_garbled_line() {
        // From REFERENCE.md: invalid lines are ignored
        let result = Parser::parse("这行是来捣乱的");
        assert!(result.ast.is_none());
    }

    #[test]
    fn test_strtol_no_expression() {
        let result = Parser::parse("+ + +");
        assert!(result.ast.is_none());
    }

    #[test]
    fn test_percentage() {
        let ast = parse_simple("150%").unwrap();
        assert!(matches!(ast, Expr::Number(Number::Percent(_))));
    }

    #[test]
    fn test_decimal() {
        let ast = parse_simple("3.14").unwrap();
        assert!(matches!(ast, Expr::Number(Number::Decimal(_))));
    }

    // --- New syntax: bonus dice ---

    #[test]
    fn test_dice_bonus() {
        let ast = parse_simple("2d6b5").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.bonus, Some(Number::Int(5)));
                assert_eq!(d.keep, None);
                assert_eq!(d.comparison, None);
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_bonus_with_keep() {
        let ast = parse_simple("2d6b5k3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.bonus, Some(Number::Int(5)));
                assert_eq!(d.keep, Some(KeepMode::High(3)));
                assert_eq!(d.comparison, None);
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_bonus_keep_comparison() {
        let ast = parse_simple("2d6b5k3>3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.bonus, Some(Number::Int(5)));
                assert_eq!(d.keep, Some(KeepMode::High(3)));
                assert_eq!(d.comparison, Some(ComparisonOp::Greater(Number::Int(3))));
            }
            _ => panic!("expected dice"),
        }
    }

    // --- New syntax: comparison operators ---

    #[test]
    fn test_dice_comparison_greater() {
        let ast = parse_simple("4d6>3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.comparison, Some(ComparisonOp::Greater(Number::Int(3))));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_comparison_greater_equal() {
        let ast = parse_simple("4d6>=3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.comparison, Some(ComparisonOp::GreaterEqual(Number::Int(3))));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_comparison_less() {
        let ast = parse_simple("4d6<3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.comparison, Some(ComparisonOp::Less(Number::Int(3))));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_comparison_less_equal() {
        let ast = parse_simple("4d6<=3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.comparison, Some(ComparisonOp::LessEqual(Number::Int(3))));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_comparison_not_equal() {
        let ast = parse_simple("4d6!=3").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.comparison, Some(ComparisonOp::NotEqual(Number::Int(3))));
            }
            _ => panic!("expected dice"),
        }
    }

    #[test]
    fn test_dice_comparison_no_keep() {
        // Comparison without keep should still work
        let ast = parse_simple("d20>10").unwrap();
        match ast {
            Expr::Dice(d) => {
                assert_eq!(d.count, Number::Int(1));
                assert_eq!(d.faces, Number::Int(20));
                assert_eq!(d.comparison, Some(ComparisonOp::Greater(Number::Int(10))));
                assert_eq!(d.keep, None);
                assert_eq!(d.bonus, None);
            }
            _ => panic!("expected dice"),
        }
    }
}
pub mod scanner;
pub mod token;

use token::{Token, TokenKind};

/// The lexer (tokenizer) converts a source string into a sequence of tokens.
pub struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from the source string.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    /// Skip whitespace (spaces only; newlines should have been handled by the scanner).
    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos] == b' ' {
            self.pos += 1;
        }
    }

    /// Peek at the byte after the current one.
    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    /// Tokenize the entire source into a vector of tokens (ending with Eof).
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                break;
            }

            let start = self.pos;
            let c = self.bytes[self.pos];

            let token = match c {
                b'+' => self.consume_single(TokenKind::Plus, start),
                b'-' => self.consume_single(TokenKind::Minus, start),
                b'*' => self.consume_single(TokenKind::Asterisk, start),
                b'/' => self.consume_single(TokenKind::Slash, start),
                b'(' => self.consume_single(TokenKind::LParen, start),
                b')' => self.consume_single(TokenKind::RParen, start),
                b'{' => self.consume_single(TokenKind::LBrace, start),
                b'}' => self.consume_single(TokenKind::RBrace, start),
                b',' => self.consume_single(TokenKind::Comma, start),
                b'#' => self.consume_single(TokenKind::Sharp, start),
                b'd' | b'D' => self.consume_single(TokenKind::Dice, start),
                b'k' | b'K' => self.consume_keyword_k(start),
                b'0'..=b'9' | b'.' => self.consume_number(start),
                _ => {
                    // Unknown character: stop tokenizing here
                    // The parser will treat the remaining as tail
                    break;
                }
            };

            tokens.push(token);
        }

        tokens.push(Token::eof(self.pos));
        tokens
    }

    fn consume_single(&mut self, kind: TokenKind, start: usize) -> Token {
        let text = &self.source[start..start + 1];
        self.pos += 1;
        Token::new(kind, text.to_string(), start, start + 1)
    }

    /// Handle `k`, `K`, `kl`, `kL`, `Kl`, `KL`
    fn consume_keyword_k(&mut self, start: usize) -> Token {
        // Check for two-character "kl" variants
        if let Some(next) = self.peek_next() {
            let lower_next = next.to_ascii_lowercase();
            if lower_next == b'l' {
                let text = &self.source[start..start + 2];
                self.pos += 2;
                return Token::new(TokenKind::KeepLow, text.to_string(), start, start + 2);
            }
        }
        // Single character `k` or `K`
        let text = &self.source[start..start + 1];
        self.pos += 1;
        Token::new(TokenKind::KeepHigh, text.to_string(), start, start + 1)
    }

    /// Consume a number: integer, decimal, or percentage.
    /// Examples: `42`, `3.14`, `150%`, `.5`
    fn consume_number(&mut self, start: usize) -> Token {
        let mut i = start;
        let mut dotted = self.bytes[i] == b'.';

        if dotted {
            i += 1;
        }

        while i < self.bytes.len() {
            let c = self.bytes[i];
            if c.is_ascii_digit() {
                i += 1;
            } else if !dotted && c == b'.' {
                dotted = true;
                i += 1;
            } else if c == b'%' {
                i += 1;
                break;
            } else {
                break;
            }
        }

        let text = &self.source[start..i];
        self.pos = i;
        Token::new(TokenKind::Number, text.to_string(), start, i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens = Lexer::new("1d20+3").tokenize();
        assert_eq!(tokens.len(), 6); // 5 tokens + Eof
        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[0].text, "1");
        assert_eq!(tokens[1].kind, TokenKind::Dice);
        assert_eq!(tokens[1].text, "d");
        assert_eq!(tokens[2].kind, TokenKind::Number);
        assert_eq!(tokens[2].text, "20");
        assert_eq!(tokens[3].kind, TokenKind::Plus);
        assert_eq!(tokens[3].text, "+");
        assert_eq!(tokens[4].kind, TokenKind::Number);
        assert_eq!(tokens[4].text, "3");
        assert_eq!(tokens[5].kind, TokenKind::Eof);
    }

    #[test]
    fn test_keep_keywords() {
        let tokens = Lexer::new("4d6k3 2d6kl1").tokenize();
        assert_eq!(tokens[3].kind, TokenKind::KeepHigh);
        assert_eq!(tokens[3].text, "k");
        assert_eq!(tokens[8].kind, TokenKind::KeepLow);
        assert_eq!(tokens[8].text, "kl");
    }

    #[test]
    fn test_percentage() {
        let tokens = Lexer::new("150%").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[0].text, "150%");
    }

    #[test]
    fn test_braces_and_sharp() {
        let tokens = Lexer::new("{4d6,3d6} 6#4d6").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::LBrace);
        assert_eq!(tokens[10].kind, TokenKind::Sharp);
    }
}
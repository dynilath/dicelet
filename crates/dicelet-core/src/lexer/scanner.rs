/// Pre-scanning module that determines the valid parse range by checking
/// parenthesis/brace matching. This is a direct port of the original C++
/// `regulate_parenthesis` function.
///
/// The scanner walks the input and tracks matched pairs of `()`, `{}`.
/// If an unmatched opener is found, the valid range is truncated to the
/// position of the first unmatched opener. This implements the first layer
/// of fault-tolerant parsing: the input `"d20 + (d4+ 测试"` will be
/// truncated to `"d20 + "` with the tail starting at `"("`.

/// Result of the pre-scan.
pub struct ScanResult<'a> {
    /// The valid source portion to be parsed.
    pub valid_source: &'a str,
    /// The tail portion (everything after the valid source, trimmed of leading spaces).
    pub tail: &'a str,
    /// The byte index in the original source where the valid portion ends.
    pub valid_end: usize,
}

/// Scan the source and determine the valid parse range.
///
/// `parse_brace` should be true for dicelet expressions (which support `{}`).
pub fn scan(source: &str, parse_brace: bool) -> ScanResult<'_> {
    let bytes = source.as_bytes();
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut good_for_comma: i32 = 0;
    let mut terminate_pos: Option<usize> = None;

    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;

        match c {
            '{' => {
                if !parse_brace {
                    terminate_pos = Some(i);
                    break;
                }
                good_for_comma += 1;
                stack.push((c, i));
            }
            '(' => {
                stack.push((c, i));
            }
            '}' => {
                if !parse_brace {
                    terminate_pos = Some(i);
                    break;
                }
                good_for_comma -= 1;
                if stack.is_empty() {
                    terminate_pos = Some(i);
                    break;
                }
                if stack.last().map(|(ch, _)| *ch) == Some('{') {
                    stack.pop();
                } else {
                    terminate_pos = Some(i);
                    break;
                }
            }
            ')' => {
                if stack.is_empty() {
                    terminate_pos = Some(i);
                    break;
                }
                if stack.last().map(|(ch, _)| *ch) == Some('(') {
                    stack.pop();
                } else {
                    terminate_pos = Some(i);
                    break;
                }
            }
            ',' => {
                if !parse_brace {
                    terminate_pos = Some(i);
                    break;
                }
                if good_for_comma == 0 {
                    terminate_pos = Some(i);
                    break;
                }
            }
            '\n' => {
                terminate_pos = Some(i);
                break;
            }
            _ => {}
        }

        i += 1;
    }

    // If we terminated early due to an error char, the valid range is [0, terminate_pos)
    if let Some(pos) = terminate_pos {
        let valid_end = pos;
        let tail_start = skip_spaces(source, valid_end);
        return ScanResult {
            valid_source: &source[..valid_end],
            tail: &source[tail_start..],
            valid_end,
        };
    }

    // We consumed the entire string. Check if there are unmatched openers.
    if !stack.is_empty() {
        // Truncate to the position of the first unmatched opener
        let first_unmatched = stack[0].1;
        let tail_start = skip_spaces(source, first_unmatched);
        return ScanResult {
            valid_source: &source[..first_unmatched],
            tail: &source[tail_start..],
            valid_end: first_unmatched,
        };
    }

    // Everything is valid
    ScanResult {
        valid_source: source,
        tail: "",
        valid_end: source.len(),
    }
}

/// Skip leading space characters starting at `pos`.
fn skip_spaces(source: &str, pos: usize) -> usize {
    let bytes = source.as_bytes();
    let mut i = pos;
    while i < bytes.len() && bytes[i] == b' ' {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced() {
        let r = scan("(1+2)*3", true);
        assert_eq!(r.valid_source, "(1+2)*3");
        assert_eq!(r.tail, "");
    }

    #[test]
    fn test_unmatched_paren() {
        let r = scan("d20 + (d4+ 测试", true);
        assert_eq!(r.valid_source, "d20 + ");
        // tail should start at the unmatched "("
        assert_eq!(r.tail, "(d4+ 测试");
    }

    #[test]
    fn test_unmatched_brace() {
        let r = scan("d20 + {1, 2", true);
        assert_eq!(r.valid_source, "d20 + ");
        assert_eq!(r.tail, "{1, 2");
    }

    #[test]
    fn test_newline_terminates() {
        let r = scan("d20\ntail", true);
        assert_eq!(r.valid_source, "d20");
        // The newline is not a space, so it remains in the tail
        assert_eq!(r.tail, "\ntail");
    }

    #[test]
    fn test_comma_outside_brace() {
        let r = scan("d20, tail", true);
        assert_eq!(r.valid_source, "d20");
        assert_eq!(r.tail, ", tail");
    }

    #[test]
    fn test_balanced_with_tail_spaces() {
        let r = scan("(1+2)   ", true);
        assert_eq!(r.valid_source, "(1+2)   ");
        assert_eq!(r.tail, "");
    }
}
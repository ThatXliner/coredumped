//! Syntax highlighter for Glyph source. Tokenizes source into categorized
//! spans so the renderer can paint each category with its own color.

use bracket_lib::prelude::RGB;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tok {
    Paren,
    Keyword,
    String,
    Number,
    Comment,
    Special,
    Constant,
    Quote,
    Symbol,
}

#[derive(Clone, Debug)]
pub struct Span {
    pub text: String,
    pub tok: Tok,
}

impl Span {
    pub fn color(&self) -> RGB {
        match self.tok {
            Tok::Paren => RGB::named(bracket_lib::prelude::GRAY),
            Tok::Keyword => RGB::named(bracket_lib::prelude::MAGENTA),
            Tok::String => RGB::named(bracket_lib::prelude::GREEN),
            Tok::Number => RGB::named(bracket_lib::prelude::CYAN),
            Tok::Comment => RGB::named(bracket_lib::prelude::DARK_GRAY),
            Tok::Special => RGB::named(bracket_lib::prelude::WHITE),
            Tok::Constant => RGB::named(bracket_lib::prelude::YELLOW),
            Tok::Quote => RGB::named(bracket_lib::prelude::GRAY),
            Tok::Symbol => RGB::named(bracket_lib::prelude::WHITE),
        }
    }
}

/// Tokenize a line of Glyph source into categorized spans.
pub fn highlight(source: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Whitespace — emit as-is with no special color
        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            spans.push(Span {
                text: chars[start..i].iter().collect(),
                tok: Tok::Symbol,
            });
            continue;
        }

        // Comment — rest of line
        if c == ';' {
            let rest: String = chars[i..].iter().collect();
            spans.push(Span {
                text: rest,
                tok: Tok::Comment,
            });
            break;
        }

        // String
        if c == '"' {
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2;
                } else if chars[i] == '"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            spans.push(Span {
                text: chars[start..i].iter().collect(),
                tok: Tok::String,
            });
            continue;
        }

        // Keyword
        if c == ':' {
            let start = i;
            i += 1;
            while i < chars.len() && is_sym_char(chars[i]) {
                i += 1;
            }
            if i < chars.len() && chars[i] == '/' {
                i += 1;
                while i < chars.len() && is_sym_char(chars[i]) {
                    i += 1;
                }
            }
            spans.push(Span {
                text: chars[start..i].iter().collect(),
                tok: Tok::Keyword,
            });
            continue;
        }

        // Parens / brackets / braces
        if c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}' {
            spans.push(Span {
                text: c.to_string(),
                tok: Tok::Paren,
            });
            i += 1;
            continue;
        }

        // Reader macro prefixes (#[, #{)
        if c == '#' && i + 1 < chars.len() {
            match chars[i + 1] {
                '[' | '{' => {
                    spans.push(Span {
                        text: chars[i..i + 2].iter().collect(),
                        tok: Tok::Paren,
                    });
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }

        // Quote
        if c == '\'' {
            spans.push(Span {
                text: c.to_string(),
                tok: Tok::Quote,
            });
            i += 1;
            continue;
        }

        // Negative number
        if c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            let start = i;
            i += 1; // consume '-'
            read_number_body(&chars, &mut i);
            spans.push(Span {
                text: chars[start..i].iter().collect(),
                tok: Tok::Number,
            });
            continue;
        }

        // Positive number or hex/oct/bin prefix
        if c.is_ascii_digit() {
            let start = i;
            read_number_body(&chars, &mut i);
            spans.push(Span {
                text: chars[start..i].iter().collect(),
                tok: Tok::Number,
            });
            continue;
        }

        // Symbol (may include dotted access chain)
        if is_sym_char(c) {
            let start = i;
            while i < chars.len() && is_sym_char(chars[i]) {
                i += 1;
            }
            // Dotted access: symbol.keyword.keyword
            while i < chars.len() && chars[i] == '.' {
                let next = i + 1;
                if next < chars.len()
                    && (is_sym_char(chars[next]) || chars[next] == '-' || chars[next] == '>')
                {
                    i += 1;
                    while i < chars.len()
                        && (is_sym_char(chars[i]) || chars[i] == '-' || chars[i] == '>')
                    {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
            let text: String = chars[start..i].iter().collect();
            let tok = classify_symbol(&text);
            spans.push(Span { text, tok });
            continue;
        }

        // Fallback: unknown single char
        spans.push(Span {
            text: c.to_string(),
            tok: Tok::Symbol,
        });
        i += 1;
    }

    spans
}

fn read_number_body(chars: &[char], i: &mut usize) {
    // 0x / 0o / 0b prefixes
    if *i < chars.len() && chars[*i] == '0' {
        let next = *i + 1;
        if next < chars.len() {
            match chars[next] {
                'x' | 'X' | 'o' | 'O' | 'b' | 'B' => {
                    *i += 2;
                    while *i < chars.len() && chars[*i].is_ascii_alphanumeric() {
                        *i += 1;
                    }
                    return;
                }
                _ => {}
            }
        }
    }
    while *i < chars.len() && chars[*i].is_ascii_digit() {
        *i += 1;
    }
    if *i < chars.len() && chars[*i] == '.' {
        *i += 1;
        while *i < chars.len() && chars[*i].is_ascii_digit() {
            *i += 1;
        }
    }
    if *i < chars.len() && (chars[*i] == 'e' || chars[*i] == 'E') {
        *i += 1;
        if *i < chars.len() && (chars[*i] == '+' || chars[*i] == '-') {
            *i += 1;
        }
        while *i < chars.len() && chars[*i].is_ascii_digit() {
            *i += 1;
        }
    }
}

fn is_sym_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '?' | '!' | '+' | '-' | '*' | '/' | '=' | '<' | '>' | '_' | '%' | '&'
        )
}

fn classify_symbol(text: &str) -> Tok {
    let lower = text.to_lowercase();
    match lower.as_str() {
        "nil" | "true" | "false" => Tok::Constant,
        "defrule" | "if" | "fn" | "let" | "do" | "quote" | "defmacro" | "match" | "try"
        | "catch" | "const" | "set!" | "and" | "or" | "first" | "rest" | "cons" | "list"
        | "vec" | "map" | "set" | "get" | "keys" | "vals" | "type" | "=" | "<" | ">" | "<="
        | ">=" | "==" | "!=" | "+" | "-" | "*" | "/" | "%" | "not" | "println" | "str" | "len"
        | "range" | "nth" | "conj" | "assoc" | "dissoc" | "contains?" | "empty?" | "nil?"
        | "bool?" | "int?" | "float?" | "string?" | "keyword?" | "symbol?" | "list?" | "vec?"
        | "map?" | "set?" | "fn?" => Tok::Special,
        _ => Tok::Symbol,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert!(highlight("").is_empty());
    }

    #[test]
    fn test_parens() {
        let spans = highlight("()");
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].tok, Tok::Paren);
        assert_eq!(spans[1].tok, Tok::Paren);
    }

    #[test]
    fn test_keyword() {
        let spans = highlight(":phase :enemy-ai");
        assert_eq!(spans.len(), 3); // :phase, space, :enemy-ai
        assert_eq!(spans[0].tok, Tok::Keyword);
        assert_eq!(spans[2].tok, Tok::Keyword);
    }

    #[test]
    fn test_string() {
        let spans = highlight(r#""hello" "a\nb""#);
        assert_eq!(spans.len(), 3); // "hello", space, "a\nb"
        assert_eq!(spans[0].tok, Tok::String);
        assert_eq!(spans[2].tok, Tok::String);
    }

    #[test]
    fn test_numbers() {
        let spans = highlight("42 -17 3.14 0xff");
        assert_eq!(spans[0].tok, Tok::Number);
        assert_eq!(spans[2].tok, Tok::Number);
        assert_eq!(spans[4].tok, Tok::Number);
        assert_eq!(spans[6].tok, Tok::Number);
    }

    #[test]
    fn test_comment() {
        let spans = highlight("(+ 1 2) ; adds numbers");
        let last = spans.last().unwrap();
        assert_eq!(last.tok, Tok::Comment);
        assert!(last.text.contains("adds numbers"));
    }

    #[test]
    fn test_special_forms() {
        let spans = highlight("(if true :ok :no)");
        assert_eq!(spans[1].tok, Tok::Paren); // (
        assert_eq!(spans[2].tok, Tok::Special); // if
    }

    #[test]
    fn test_constants() {
        let spans = highlight("nil true false");
        assert_eq!(spans[0].tok, Tok::Constant);
        assert_eq!(spans[2].tok, Tok::Constant);
        assert_eq!(spans[4].tok, Tok::Constant);
    }

    #[test]
    fn test_quote() {
        let spans = highlight("'(1 2)");
        assert_eq!(spans[0].tok, Tok::Quote);
    }

    #[test]
    fn test_dotted_access() {
        let spans = highlight("player.pos");
        // whole thing is one symbol span
        let syms: Vec<_> = spans
            .iter()
            .filter(|s| s.tok == Tok::Symbol || s.tok == Tok::Special)
            .collect();
        assert_eq!(syms[0].text, "player.pos");
    }
}

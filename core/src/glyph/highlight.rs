//! Syntax highlighter for Glyph source. Tokenizes source into categorized
//! spans so the renderer can paint each category with its own color.
//!
//! This is intentionally a lightweight lexer, independent from `reader`. The
//! console calls it while source is incomplete or invalid, so highlighting must
//! never require a parsed AST.

use std::collections::HashSet;
use std::sync::OnceLock;

use bracket_color::prelude::RGB;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tok {
    Paren,
    ReaderMacro,
    Keyword,
    String,
    Escape,
    Number,
    Comment,
    Special,
    Builtin,
    Command,
    Operator,
    Definition,
    Binding,
    Constant,
    Quote,
    Property,
    Call,
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
            Tok::Paren => rgb(126, 136, 150),
            Tok::ReaderMacro => rgb(85, 210, 220),
            Tok::Keyword => rgb(238, 112, 210),
            Tok::String => rgb(122, 206, 116),
            Tok::Escape => rgb(248, 188, 83),
            Tok::Number => rgb(82, 190, 236),
            Tok::Comment => rgb(86, 94, 105),
            Tok::Special => rgb(177, 146, 255),
            Tok::Builtin => rgb(118, 177, 255),
            Tok::Command => rgb(255, 136, 96),
            Tok::Operator => rgb(242, 210, 92),
            Tok::Definition => rgb(255, 224, 94),
            Tok::Binding => rgb(132, 218, 198),
            Tok::Constant => rgb(255, 154, 94),
            Tok::Quote => rgb(158, 166, 178),
            Tok::Property => rgb(170, 210, 106),
            Tok::Call => rgb(158, 202, 255),
            Tok::Symbol => RGB::named(bracket_color::prelude::WHITE),
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

        if c.is_whitespace() {
            let start = i;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            push_span(&mut spans, &chars, start, i, Tok::Symbol);
            continue;
        }

        if c == ';' {
            push_span(&mut spans, &chars, i, chars.len(), Tok::Comment);
            break;
        }

        if c == '"' {
            read_string_literal(&mut spans, &chars, &mut i);
            continue;
        }

        if c == ':' {
            let start = i;
            i += 1;
            while i < chars.len() && (is_sym_char(chars[i]) || chars[i] == '/') {
                i += 1;
            }
            push_span(&mut spans, &chars, start, i, Tok::Keyword);
            continue;
        }

        if c == '#' && i + 1 < chars.len() {
            match chars[i + 1] {
                '[' | '{' => {
                    push_span(&mut spans, &chars, i, i + 2, Tok::ReaderMacro);
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }

        if matches!(c, '(' | ')' | '[' | ']' | '{' | '}') {
            push_span(&mut spans, &chars, i, i + 1, Tok::Paren);
            i += 1;
            continue;
        }

        if c == '\'' {
            push_span(&mut spans, &chars, i, i + 1, Tok::Quote);
            i += 1;
            continue;
        }

        if c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            let start = i;
            i += 1;
            read_number_body(&chars, &mut i);
            push_span(&mut spans, &chars, start, i, Tok::Number);
            continue;
        }

        if c.is_ascii_digit() {
            let start = i;
            read_number_body(&chars, &mut i);
            push_span(&mut spans, &chars, start, i, Tok::Number);
            continue;
        }

        if is_sym_char(c) {
            let start = i;
            while i < chars.len() && is_sym_char(chars[i]) {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let tok = classify_symbol(&text);
            spans.push(Span { text, tok });

            while i < chars.len() && chars[i] == '.' {
                let next = i + 1;
                if next < chars.len()
                    && (is_sym_char(chars[next]) || chars[next] == '-' || chars[next] == '>')
                {
                    let prop_start = i;
                    i += 1;
                    while i < chars.len()
                        && (is_sym_char(chars[i]) || chars[i] == '-' || chars[i] == '>')
                    {
                        i += 1;
                    }
                    push_span(&mut spans, &chars, prop_start, i, Tok::Property);
                } else {
                    break;
                }
            }
            continue;
        }

        if c == '.' {
            push_span(&mut spans, &chars, i, i + 1, Tok::Operator);
            i += 1;
            continue;
        }

        push_span(&mut spans, &chars, i, i + 1, Tok::Symbol);
        i += 1;
    }

    annotate_context(&mut spans);
    spans
}

fn read_string_literal(spans: &mut Vec<Span>, chars: &[char], i: &mut usize) {
    let mut segment_start = *i;
    *i += 1;

    while *i < chars.len() {
        if chars[*i] == '\\' && *i + 1 < chars.len() {
            push_span(spans, chars, segment_start, *i, Tok::String);
            push_span(spans, chars, *i, *i + 2, Tok::Escape);
            *i += 2;
            segment_start = *i;
        } else if chars[*i] == '"' {
            *i += 1;
            push_span(spans, chars, segment_start, *i, Tok::String);
            return;
        } else {
            *i += 1;
        }
    }

    push_span(spans, chars, segment_start, *i, Tok::String);
}

fn read_number_body(chars: &[char], i: &mut usize) {
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

fn annotate_context(spans: &mut [Span]) {
    let mut previous_sig: Option<String> = None;
    let mut pending_param_list = false;
    let mut param_square_depth: Option<usize> = None;
    let mut square_depth = 0usize;

    for i in 0..spans.len() {
        if is_ws_span(&spans[i]) {
            continue;
        }

        let text = spans[i].text.clone();
        let key = context_key(&spans[i]);

        match text.as_str() {
            "[" | "#[" => {
                if pending_param_list && text == "[" {
                    param_square_depth = Some(square_depth + 1);
                    pending_param_list = false;
                }
                square_depth += 1;
                previous_sig = Some(key);
                continue;
            }
            "]" => {
                if param_square_depth == Some(square_depth) {
                    param_square_depth = None;
                }
                square_depth = square_depth.saturating_sub(1);
                previous_sig = Some(key);
                continue;
            }
            _ => {}
        }

        if is_symbol_like(spans[i].tok) {
            let prev = previous_sig.as_deref();
            if key == "_" {
                spans[i].tok = Tok::Constant;
            } else if param_square_depth.is_some() {
                spans[i].tok = if key == "&" {
                    Tok::Special
                } else {
                    Tok::Binding
                };
            } else if matches!(prev, Some("const" | "defmacro" | "defrule")) {
                spans[i].tok = Tok::Definition;
                if prev == Some("defmacro") {
                    pending_param_list = true;
                }
            } else if prev == Some("fn") {
                spans[i].tok = Tok::Binding;
                pending_param_list = false;
            } else if matches!(prev, Some("let" | "set!" | "catch")) {
                spans[i].tok = Tok::Binding;
            } else if prev == Some("(") && spans[i].tok == Tok::Symbol {
                spans[i].tok = Tok::Call;
            }
        }

        if key == "fn" {
            pending_param_list = true;
        }

        previous_sig = Some(key);
    }
}

/// The set of function names the highlighter recognizes, split into game
/// commands (the `setup_glyph_env` layer) and core builtins (`default_env`).
///
/// Populated once at startup from the live Glyph environment via [`set_vocab`],
/// so adding a `reg!` entry colors it automatically — no hand-maintained list.
/// Until populated, `classify_symbol` falls back to plain symbols for anything
/// that is not a special form or operator (both of which live in code, not the
/// env, and so stay as authoritative lists below).
#[derive(Default)]
struct Vocab {
    commands: HashSet<String>,
    builtins: HashSet<String>,
}

static VOCAB: OnceLock<Vocab> = OnceLock::new();

/// Register the function vocabulary used for highlighting. Idempotent: the first
/// caller wins, later calls are ignored (the vocabulary is fixed at startup).
/// Names are stored lowercased to match the case-insensitive Glyph environment.
pub fn set_vocab(commands: &[String], builtins: &[String]) {
    let _ = VOCAB.set(Vocab {
        commands: commands.iter().map(|n| n.to_lowercase()).collect(),
        builtins: builtins.iter().map(|n| n.to_lowercase()).collect(),
    });
}

fn classify_symbol(text: &str) -> Tok {
    // The Glyph env is case-insensitive (Env::lookup lowercases), so the
    // highlighter mirrors that and classifies on the lowercased name.
    let lower = text.to_lowercase();

    if matches!(lower.as_str(), "nil" | "true" | "false" | "_") {
        return Tok::Constant;
    }

    if lower.len() > 2 && lower.starts_with('*') && lower.ends_with('*') {
        return Tok::Binding;
    }

    if is_special_form(&lower) {
        return Tok::Special;
    }

    if is_operator(&lower) {
        return Tok::Operator;
    }

    if let Some(vocab) = VOCAB.get() {
        if vocab.commands.contains(&lower) {
            return Tok::Command;
        }
        if vocab.builtins.contains(&lower) {
            return Tok::Builtin;
        }
    }

    Tok::Symbol
}

fn is_special_form(text: &str) -> bool {
    matches!(
        text,
        "quote"
            | "if"
            | "do"
            | "let"
            | "fn"
            | "const"
            | "defmacro"
            | "set!"
            | "try"
            | "catch"
            | "and"
            | "or"
            | "match"
            | "bind-key"
            | "recur"
            | "defrule"
    )
}

fn is_operator(text: &str) -> bool {
    matches!(
        text,
        "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "."
    )
}

fn is_sym_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '?' | '!' | '+' | '-' | '*' | '/' | '=' | '<' | '>' | '_' | '%' | '&'
        )
}

fn push_span(spans: &mut Vec<Span>, chars: &[char], start: usize, end: usize, tok: Tok) {
    if start < end {
        spans.push(Span {
            text: chars[start..end].iter().collect(),
            tok,
        });
    }
}

fn rgb(r: u8, g: u8, b: u8) -> RGB {
    RGB::from_f32(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

fn is_ws_span(span: &Span) -> bool {
    span.text.chars().all(char::is_whitespace)
}

fn is_symbol_like(tok: Tok) -> bool {
    matches!(
        tok,
        Tok::Symbol | Tok::Call | Tok::Builtin | Tok::Command | Tok::Operator | Tok::Binding
    )
}

fn context_key(span: &Span) -> String {
    span.text.to_lowercase()
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
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].tok, Tok::Keyword);
        assert_eq!(spans[2].tok, Tok::Keyword);
    }

    #[test]
    fn test_string() {
        let spans = highlight(r#""hello" "a\nb""#);
        assert_eq!(spans[0].tok, Tok::String);
        assert!(spans.iter().any(|span| span.tok == Tok::Escape));
        assert_eq!(spans.last().unwrap().tok, Tok::String);
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
        assert_eq!(spans[0].tok, Tok::Paren);
        assert_eq!(spans[1].tok, Tok::Special);
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
        assert_eq!(spans[0].text, "player");
        assert_eq!(spans[0].tok, Tok::Symbol);
        assert_eq!(spans[1].text, ".pos");
        assert_eq!(spans[1].tok, Tok::Property);
    }

    #[test]
    fn test_definitions_and_bindings() {
        let spans = highlight("(const answer (fn [x] (+ x 1)))");
        assert_token(&spans, "const", Tok::Special);
        assert_token(&spans, "answer", Tok::Definition);
        assert_token(&spans, "fn", Tok::Special);
        assert_token(&spans, "x", Tok::Binding);
    }

    #[test]
    fn test_builtin_command_operator_and_call() {
        // Commands and builtins are classified from the vocabulary registered at
        // startup. Seed it here so the unit test does not depend on env setup.
        // `set_vocab` is set-once; ignore the result if another test set it first
        // (the real env contains these same names).
        set_vocab(&["move!".to_string()], &["list".to_string()]);
        let spans = highlight("(move! :north) (list (+ a 1)) (custom-call)");
        assert_token(&spans, "move!", Tok::Command);
        assert_token(&spans, "list", Tok::Builtin);
        assert_token(&spans, "+", Tok::Operator);
        assert_token(&spans, "custom-call", Tok::Call);
    }

    #[test]
    fn test_reader_macros() {
        let spans = highlight("#[:red #{:blue}]");
        assert_eq!(spans[0].tok, Tok::ReaderMacro);
        assert!(spans
            .iter()
            .any(|span| span.text == "#{" && span.tok == Tok::ReaderMacro));
    }

    #[test]
    fn test_incomplete_source_still_highlights() {
        let spans = highlight(r#"(bind-key :z (do "unterminated"#);
        assert_token(&spans, "bind-key", Tok::Special);
        assert_token(&spans, ":z", Tok::Keyword);
        assert!(spans.iter().any(|span| span.tok == Tok::String));
    }

    fn assert_token(spans: &[Span], text: &str, tok: Tok) {
        let span = spans
            .iter()
            .find(|span| span.text == text)
            .unwrap_or_else(|| panic!("missing token {text:?} in {spans:?}"));
        assert_eq!(span.tok, tok);
    }
}

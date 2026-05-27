//! Reader: parses Glyph source text into canonical Value forms.

use std::collections::{BTreeMap, BTreeSet};
use std::iter::Peekable;
use std::str::Chars;

use super::value::*;

pub struct Reader<'a> {
    pub(crate) input: Peekable<Chars<'a>>,
    offset: usize,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str) -> Self {
        Reader {
            input: source.chars().peekable(),
            offset: 0,
        }
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.input.next()?;
        self.offset += c.len_utf8();
        Some(c)
    }

    pub fn read(&mut self) -> ReadResult<Option<Value>> {
        self.skip_ws();
        match self.input.peek() {
            None => Ok(None),
            Some(_) => Ok(Some(self.read_form()?)),
        }
    }

    pub fn read_all(&mut self) -> ReadResult<Vec<Value>> {
        let mut forms = Vec::new();
        while let Some(form) = self.read()? {
            forms.push(form);
        }
        Ok(forms)
    }

    pub fn read_form(&mut self) -> ReadResult<Value> {
        self.skip_ws();
        match self.input.peek() {
            None => Err(ReadError::UnexpectedEof(
                "reading a form".into(),
                self.offset,
            )),
            Some(&c) => match c {
                '(' => self.read_list(),
                ')' => Err(ReadError::UnexpectedChar(
                    ')',
                    "unmatched paren".into(),
                    self.offset,
                )),
                '[' => self.read_infix(),
                ']' => Err(ReadError::UnexpectedChar(
                    ']',
                    "unmatched bracket".into(),
                    self.offset,
                )),
                '#' => {
                    self.bump();
                    match self.input.peek() {
                        Some('[') => self.read_vector(),
                        Some('{') => self.read_set(),
                        Some(c) => Err(ReadError::UnexpectedChar(
                            *c,
                            "expected [ or { after #".into(),
                            self.offset,
                        )),
                        None => Err(ReadError::UnexpectedEof("after #".into(), self.offset)),
                    }
                }
                '{' => self.read_map(),
                '}' => Err(ReadError::UnexpectedChar(
                    '}',
                    "unmatched brace".into(),
                    self.offset,
                )),
                ':' => self.read_keyword(),
                '"' => self.read_string(),
                '\'' => {
                    self.bump();
                    let form = self.read_form()?;
                    Ok(Value::List(vec![super::sym("quote"), form]))
                }
                '.' => {
                    self.bump();
                    Ok(Value::Symbol(Symbol::new(".")))
                }
                ';' => {
                    self.skip_line();
                    self.read_form()
                }
                _ => {
                    let peek = self.input.peek().copied();
                    let start_is_digit = peek.map_or(false, |c| c.is_ascii_digit());
                    let start_is_neg = peek == Some('-')
                        && self
                            .input
                            .clone()
                            .nth(1)
                            .map_or(false, |c| c.is_ascii_digit());
                    if start_is_digit || start_is_neg {
                        self.read_number()
                    } else {
                        self.read_symbol_or_dot()
                    }
                }
            },
        }
    }

    // --- lists, vectors, sets, maps ---

    fn read_list(&mut self) -> ReadResult<Value> {
        self.bump();
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            match self.input.peek() {
                None => return Err(ReadError::UnexpectedEof("reading list".into(), self.offset)),
                Some(&')') => {
                    self.bump();
                    return Ok(Value::List(items));
                }
                Some(&_) => items.push(self.read_form()?),
            }
        }
    }

    fn read_vector(&mut self) -> ReadResult<Value> {
        self.bump(); // consume [
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            match self.input.peek() {
                None => return Err(ReadError::UnexpectedEof("reading list".into(), self.offset)),
                Some(&']') => {
                    self.bump();
                    let mut form = vec![super::sym("list")];
                    form.extend(items);
                    return Ok(Value::List(form));
                }
                Some(&_) => items.push(self.read_form()?),
            }
        }
    }

    fn read_set(&mut self) -> ReadResult<Value> {
        self.bump();
        let mut items = BTreeSet::new();
        loop {
            self.skip_ws();
            match self.input.peek() {
                None => return Err(ReadError::UnexpectedEof("reading set".into(), self.offset)),
                Some(&'}') => {
                    self.bump();
                    return Ok(Value::Set(items));
                }
                Some(&_) => {
                    items.insert(self.read_form()?);
                }
            }
        }
    }

    fn read_map(&mut self) -> ReadResult<Value> {
        self.bump();
        let mut entries = BTreeMap::new();
        loop {
            self.skip_ws();
            match self.input.peek() {
                None => return Err(ReadError::UnexpectedEof("reading map".into(), self.offset)),
                Some(&'}') => {
                    self.bump();
                    return Ok(Value::Map(entries));
                }
                Some(&_) => {
                    let key = self.read_form()?;
                    self.skip_ws();
                    let value = self.read_form()?;
                    entries.insert(key, value);
                }
            }
        }
    }

    // --- keywords, strings, numbers, symbols ---

    fn read_keyword(&mut self) -> ReadResult<Value> {
        self.bump();
        let mut name = String::new();
        loop {
            match self.input.peek() {
                Some(&c) if is_symbol_char(c) || c == '/' => {
                    name.push(c);
                    self.bump();
                }
                _ => break,
            }
        }
        if name.is_empty() {
            return Err(ReadError::UnexpectedChar(
                ':',
                "expected keyword name".into(),
                self.offset,
            ));
        }
        Ok(Value::Keyword(Keyword { name }))
    }

    fn read_string(&mut self) -> ReadResult<Value> {
        self.bump();
        let mut s = String::new();
        loop {
            match self.bump() {
                None => {
                    return Err(ReadError::UnexpectedEof(
                        "reading string".into(),
                        self.offset,
                    ))
                }
                Some('"') => return Ok(Value::String(s)),
                Some('\\') => {
                    let c = self
                        .bump()
                        .ok_or(ReadError::UnexpectedEof("after \\".into(), self.offset))?;
                    match c {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        c => return Err(ReadError::InvalidEscape(format!("\\{}", c), self.offset)),
                    }
                }
                Some(c) => s.push(c),
            }
        }
    }

    fn read_number(&mut self) -> ReadResult<Value> {
        let mut raw = String::new();
        let mut is_float = false;

        if let Some(&'-') = self.input.peek() {
            raw.push('-');
            self.bump();
        }

        // hex/octal/binary
        if raw == "-" || raw.is_empty() {
            if let Some(&'0') = self.input.peek() {
                raw.push('0');
                self.bump();
                match self.input.peek() {
                    Some(&'x') | Some(&'X') => {
                        raw.push('x');
                        self.bump();
                        return self.read_radix(raw, 16);
                    }
                    Some(&'o') | Some(&'O') => {
                        raw.push('o');
                        self.bump();
                        return self.read_radix(raw, 8);
                    }
                    Some(&'b') | Some(&'B') => {
                        raw.push('b');
                        self.bump();
                        return self.read_radix(raw, 2);
                    }
                    _ => {}
                }
            }
        };

        // digits, decimal point, exponent
        loop {
            match self.input.peek() {
                Some(&c) if c.is_ascii_digit() => {
                    raw.push(c);
                    self.bump();
                }
                Some(&'.') if !is_float => {
                    raw.push('.');
                    is_float = true;
                    self.bump();
                    while let Some(&c) = self.input.peek() {
                        if c.is_ascii_digit() {
                            raw.push(c);
                            self.bump();
                        } else {
                            break;
                        }
                    }
                }
                Some(&'e') | Some(&'E') if !is_float => {
                    if raw.len() > if raw.starts_with('-') { 1 } else { 0 } {
                        is_float = true;
                        raw.push('e');
                        self.bump();
                        if let Some(&'+') | Some(&'-') = self.input.peek() {
                            raw.push(self.bump().unwrap());
                        }
                        while let Some(&c) = self.input.peek() {
                            if c.is_ascii_digit() {
                                raw.push(c);
                                self.bump();
                            } else {
                                break;
                            }
                        }
                    }
                    break;
                }
                _ => break,
            }
        }

        if raw.is_empty() || raw == "-" {
            return Err(ReadError::InvalidNumber(raw, self.offset));
        }

        if is_float {
            raw.parse::<f64>()
                .map(Value::F64)
                .map_err(|_| ReadError::InvalidNumber(raw, self.offset))
        } else {
            raw.parse::<i64>()
                .map(Value::I64)
                .map_err(|_| ReadError::InvalidNumber(raw, self.offset))
        }
    }

    fn read_radix(&mut self, mut raw: String, radix: u32) -> ReadResult<Value> {
        loop {
            match self.input.peek() {
                Some(&c) if c.is_ascii_alphanumeric() => {
                    raw.push(c);
                    self.bump();
                }
                _ => break,
            }
        }
        let (neg, start) = if raw.starts_with('-') {
            (true, 3)
        } else {
            (false, 2)
        };
        let num_str = &raw[start..];
        if num_str.is_empty() {
            return Err(ReadError::InvalidNumber(raw, self.offset));
        }
        let actual = if neg {
            format!("-{}", num_str)
        } else {
            num_str.to_string()
        };
        i64::from_str_radix(&actual, radix)
            .map(Value::I64)
            .map_err(|_| ReadError::InvalidNumber(raw, self.offset))
    }

    fn read_symbol_or_dot(&mut self) -> ReadResult<Value> {
        let mut raw = String::new();
        loop {
            match self.input.peek() {
                Some(&c) if is_symbol_char(c) => {
                    raw.push(c);
                    self.bump();
                }
                _ => break,
            }
        }
        if raw.is_empty() {
            return Err(ReadError::UnexpectedChar(
                self.bump().unwrap_or('\0'),
                "reading symbol".into(),
                self.offset,
            ));
        }

        // Dotted access: a.b.c
        if self.input.peek() == Some(&'.') {
            let mut chain = vec![raw];
            loop {
                match self.input.peek() {
                    Some(&'.') => {
                        self.bump();
                        let mut part = String::new();
                        loop {
                            match self.input.peek() {
                                Some(&c) if is_symbol_char(c) || c == '-' || c == '>' => {
                                    part.push(c);
                                    self.bump();
                                }
                                _ => break,
                            }
                        }
                        if part.is_empty() {
                            return Err(ReadError::UnexpectedChar(
                                '.',
                                "expected name after .".into(),
                                self.offset,
                            ));
                        }
                        chain.push(part);
                    }
                    _ => break,
                }
            }
            let mut result = Value::Symbol(Symbol::new(&chain[0]));
            for part in &chain[1..] {
                result = Value::List(vec![
                    super::sym("."),
                    result,
                    Value::Keyword(Keyword { name: part.clone() }),
                ]);
            }
            return Ok(result);
        }

        Ok(match raw.as_str() {
            "nil" => Value::Nil,
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            _ => Value::Symbol(Symbol::new(&raw)),
        })
    }

    // --- infix ---

    fn op_precedence(op: &str) -> u32 {
        match op {
            "or" => 1,
            "and" => 2,
            "==" | "!=" | "<" | ">" | "<=" | ">=" => 3,
            "+" | "-" => 4,
            "*" | "/" | "%" => 5,
            _ => 0,
        }
    }

    /// Read a `[...]` block. If forms[1] is a known binary operator, parse infix.
    /// Otherwise produce a List (for param lists, let bindings, etc.).
    fn read_infix(&mut self) -> ReadResult<Value> {
        self.bump();
        let forms = self.read_inner_forms(b']')?;
        if forms.is_empty() {
            return Ok(Value::List(vec![]));
        }
        if forms.len() == 1 {
            return Ok(forms.into_iter().next().unwrap());
        }
        let is_infix = forms.get(1).map_or(false, |v| {
            if let Value::Symbol(s) = v {
                Self::op_precedence(&s.name) > 0
            } else {
                false
            }
        });
        if is_infix {
            let mut tokens = forms.into_iter().peekable();
            self.parse_prec(&mut tokens, 0)
        } else {
            Ok(Value::List(forms))
        }
    }

    /// Classic precedence climbing.
    pub fn parse_prec(
        &mut self,
        tokens: &mut Peekable<std::vec::IntoIter<Value>>,
        min_prec: u32,
    ) -> ReadResult<Value> {
        let mut lhs = tokens.next().expect("at least one token");
        loop {
            let (op, prec) = match tokens.peek() {
                Some(Value::Symbol(s)) => {
                    let p = Self::op_precedence(&s.name);
                    if p == 0 || p < min_prec {
                        break;
                    }
                    (s.name.clone(), p)
                }
                _ => break,
            };
            tokens.next();
            let rhs = self.parse_prec(tokens, prec + 1)?;
            lhs = Value::List(vec![Value::Symbol(Symbol::new(&op)), lhs, rhs]);
        }
        Ok(lhs)
    }

    pub fn read_inner_forms(&mut self, delim: u8) -> ReadResult<Vec<Value>> {
        let mut forms = Vec::new();
        loop {
            self.skip_ws();
            match self.input.peek() {
                None => {
                    return Err(ReadError::UnexpectedEof(
                        "reading infix".into(),
                        self.offset,
                    ))
                }
                Some(&c) if c as u8 == delim => {
                    self.bump();
                    break;
                }
                Some(_) => forms.push(self.read_form()?),
            }
        }
        Ok(forms)
    }

    // --- whitespace ---

    fn skip_ws(&mut self) {
        loop {
            match self.input.peek() {
                Some(&c) if c.is_whitespace() => {
                    self.bump();
                }
                Some(&';') => {
                    self.skip_line();
                }
                _ => break,
            }
        }
    }

    fn skip_line(&mut self) {
        self.bump();
        loop {
            match self.bump() {
                Some('\n') | None => return,
                _ => continue,
            }
        }
    }
}

fn is_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || c == '?'
        || c == '!'
        || c == '+'
        || c == '-'
        || c == '*'
        || c == '/'
        || c == '='
        || c == '<'
        || c == '>'
        || c == '_'
        || c == '%'
        || c == '&'
}

/// Parse a string into a list of canonical forms.
pub fn read_string(source: &str) -> ReadResult<Vec<Value>> {
    let mut reader = Reader::new(source);
    reader.read_all()
}

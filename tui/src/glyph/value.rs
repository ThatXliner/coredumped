//! Core types for Glyph: Value, Symbol, Keyword, errors.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

// ============================================================================
// Virtual File System
// ============================================================================

/// Host-provided filesystem abstraction. When set on `SandboxOptions`,
/// I/O built-ins (`slurp`, etc.) read through this instead of the real
/// filesystem.
pub trait VirtualFileSystem: Send + Sync {
    fn read_to_string(&self, path: &str) -> Result<String, String>;
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone)]
pub enum ReadError {
    UnexpectedEof(String, usize),
    UnexpectedChar(char, String, usize),
    InvalidNumber(String, usize),
    InvalidEscape(String, usize),
}

impl ReadError {
    pub fn offset(&self) -> usize {
        match self {
            ReadError::UnexpectedEof(_, o)
            | ReadError::UnexpectedChar(_, _, o)
            | ReadError::InvalidNumber(_, o)
            | ReadError::InvalidEscape(_, o) => *o,
        }
    }

    pub fn report(&self, source: &str) -> String {
        use ariadne::{Color, Config, Label, Report, ReportKind, Source};
        let offset = self.offset();
        let msg = self.to_string();
        let span = offset..offset + 1;
        let report = Report::build(ReportKind::Error, "glyph", offset)
            .with_config(Config::default().with_color(false))
            .with_message("syntax error")
            .with_label(
                Label::new(("glyph", span))
                    .with_message(&msg)
                    .with_color(Color::Red),
            )
            .finish();
        let mut out = Vec::<u8>::new();
        report
            .write(("glyph", Source::from(source)), &mut out)
            .unwrap();
        String::from_utf8_lossy(&out).to_string()
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::UnexpectedEof(ctx, _) => {
                write!(f, "unexpected end of input while {}", ctx)
            }
            ReadError::UnexpectedChar(c, ctx, _) => {
                write!(f, "unexpected character '{}' while {}", c, ctx)
            }
            ReadError::InvalidNumber(s, _) => write!(f, "invalid number: {}", s),
            ReadError::InvalidEscape(s, _) => write!(f, "invalid escape sequence: {}", s),
        }
    }
}

pub type ReadResult<T> = Result<T, ReadError>;

#[derive(Debug, Clone)]
pub enum EvalError {
    UnboundSymbol(String),
    NotCallable(String),
    WrongArgCount { expected: usize, got: usize },
    NotInList(String, String),
    DivisionByZero,
    RecursionLimit,
    TypeError { expected: &'static str, got: String },
    PatternMatchFailed(String),
    Custom(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UnboundSymbol(s) => write!(f, "unbound symbol: {}", s),
            EvalError::NotCallable(v) => write!(f, "not callable: {}", v),
            EvalError::WrongArgCount { expected, got } => {
                write!(
                    f,
                    "wrong argument count: expected {}, got {}",
                    expected, got
                )
            }
            EvalError::NotInList(v, ctx) => write!(f, "{} is not in list: {}", v, ctx),
            EvalError::DivisionByZero => write!(f, "division by zero"),
            EvalError::RecursionLimit => write!(f, "recursion limit exceeded"),
            EvalError::TypeError { expected, got } => {
                write!(f, "type error: expected {}, got {}", expected, got)
            }
            EvalError::PatternMatchFailed(v) => write!(f, "no pattern matched: {}", v),
            EvalError::Custom(s) => write!(f, "error: {}", s),
        }
    }
}

pub type EvalResult<T> = Result<T, EvalError>;

/// Configuration for the evaluator's safety boundaries.
/// Passed by value; each recursive call decrements `depth`.
#[derive(Clone)]
pub struct SandboxOptions {
    /// Maximum recursion depth before RecursionLimit error.
    pub max_depth: usize,
    /// Remaining recursion budget (counts down from max_depth).
    pub depth: usize,
    /// Optional virtual filesystem for I/O sandboxing.
    /// When set, I/O built-ins read through this instead of the real filesystem.
    pub vfs: Option<Arc<dyn VirtualFileSystem>>,
}

impl SandboxOptions {
    pub fn new(max_depth: usize) -> Self {
        SandboxOptions {
            max_depth,
            depth: max_depth,
            vfs: None,
        }
    }

    /// Descend one level. Returns None if budget exhausted.
    pub(crate) fn descend(&self) -> Option<SandboxOptions> {
        if self.depth == 0 {
            None
        } else {
            Some(SandboxOptions {
                depth: self.depth - 1,
                vfs: self.vfs.clone(),
                ..*self
            })
        }
    }
}

impl Default for SandboxOptions {
    fn default() -> Self {
        SandboxOptions::new(1024)
    }
}

impl fmt::Debug for SandboxOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SandboxOptions")
            .field("max_depth", &self.max_depth)
            .field("depth", &self.depth)
            .field("vfs", &self.vfs.is_some())
            .finish()
    }
}

// ============================================================================
// Core Data Types
// ============================================================================

/// Case-insensitive symbol (interned via lowercasing).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol {
    pub name: String,
}

impl Symbol {
    pub fn new(name: &str) -> Self {
        Symbol {
            name: name.to_lowercase(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keyword {
    pub name: String,
}

/// A built-in function implemented in Rust.
///
/// The `world` parameter gives safe mutable access to the game state
/// (see [`crate::world::World`]). Builtins that don't need it simply
/// ignore the parameter.
#[derive(Clone)]
pub struct BuiltinFn {
    pub name: &'static str,
    pub func: fn(
        &[Value],
        &super::env::Env,
        &SandboxOptions,
        &mut crate::world::World,
    ) -> EvalResult<Value>,
}

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltinFn")
            .field("name", &self.name)
            .finish()
    }
}

impl PartialEq for BuiltinFn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// User-defined function with captured environment.
#[derive(Debug, Clone)]
pub struct ClosureData {
    pub params: Vec<String>,
    pub body: Vec<Value>,
    pub env: super::env::Env,
}

/// Macro definition.
#[derive(Debug, Clone)]
pub struct MacroData {
    pub params: Vec<String>,
    pub body: Vec<Value>,
    pub env: super::env::Env,
}

/// The core data type of Glyph.
#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Symbol(Symbol),
    Keyword(Keyword),
    List(Vec<Value>),
    Vector(Vec<Value>),
    Map(BTreeMap<Value, Value>),
    Set(BTreeSet<Value>),
    Builtin(BuiltinFn),
    Closure(ClosureData),
    Macro(MacroData),
}

// --- PartialEq, Eq, Ord ---

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(a), Bool(b)) => a == b,
            (I64(a), I64(b)) => a == b,
            (F64(a), F64(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Symbol(a), Symbol(b)) => a == b,
            (Keyword(a), Keyword(b)) => a == b,
            (List(a), List(b)) => a == b,
            (Vector(a), Vector(b)) => a == b,
            (Map(a), Map(b)) => a == b,
            (Set(a), Set(b)) => a == b,
            (Builtin(a), Builtin(b)) => a == b,
            (Closure(a), Closure(b)) => a.params == b.params && a.body == b.body,
            (Macro(a), Macro(b)) => a.params == b.params && a.body == b.body,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        fn tag(v: &Value) -> u8 {
            use Value::*;
            match v {
                Nil => 0,
                Bool(_) => 1,
                I64(_) => 2,
                F64(_) => 3,
                String(_) => 4,
                Symbol(_) => 5,
                Keyword(_) => 6,
                List(_) => 7,
                Vector(_) => 8,
                Map(_) => 9,
                Set(_) => 10,
                Builtin(_) => 11,
                Closure(_) => 12,
                Macro(_) => 13,
            }
        }
        let t1 = tag(self);
        let t2 = tag(other);
        if t1 != t2 {
            return t1.cmp(&t2);
        }
        match (self, other) {
            (Value::Nil, Value::Nil) => Equal,
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::I64(a), Value::I64(b)) => a.cmp(b),
            (Value::F64(a), Value::F64(b)) => a.partial_cmp(b).unwrap_or(Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Symbol(a), Value::Symbol(b)) => a.name.cmp(&b.name),
            (Value::Keyword(a), Value::Keyword(b)) => a.name.cmp(&b.name),
            (Value::List(a), Value::List(b)) => a.cmp(b),
            (Value::Vector(a), Value::Vector(b)) => a.cmp(b),
            (Value::Map(a), Value::Map(b)) => a.cmp(b),
            (Value::Set(a), Value::Set(b)) => a.cmp(b),
            (Value::Builtin(a), Value::Builtin(b)) => a.name.cmp(b.name),
            (Value::Closure(a), Value::Closure(b)) => a.params.cmp(&b.params),
            (Value::Macro(a), Value::Macro(b)) => a.params.cmp(&b.params),
            _ => Equal,
        }
    }
}

// --- Display ---

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::I64(n) => write!(f, "{}", n),
            Value::F64(n) => {
                if n.is_infinite() {
                    write!(f, "{}inf", if n.is_sign_negative() { "-" } else { "" })
                } else if n.is_nan() {
                    write!(f, "nan")
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Symbol(s) => write!(f, "{}", s.name),
            Value::Keyword(k) => write!(f, ":{}", k.name),
            Value::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            Value::Vector(items) => {
                write!(f, "#[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Map(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{} {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(items) => {
                write!(f, "#{{")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "}}")
            }
            Value::Builtin(b) => write!(f, "#<builtin {}>", b.name),
            Value::Closure(c) => write!(f, "#<closure (fn [{}] ...)>", c.params.join(" ")),
            Value::Macro(m) => write!(f, "#<macro {}>", m.params.join(" ")),
        }
    }
}

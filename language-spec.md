# Glyph Language Specification

Version 0.1 — documents what is implemented. Planned features are marked **[planned]**.

Glyph is a homoiconic Lisp dialect. Code is ordinary data: every source file reads into the same values the language operates on. The canonical form is the authoritative representation for evaluation, transformation, serialization, and diffing.

---

## 1. Values

Every expression evaluates to one of these types. Types are disjoint — a value belongs to exactly one.

### 1.1 Nil

`nil` is the empty value. It is the only falsy value besides `false`.

```lisp
nil
```

### 1.2 Booleans

`true` and `false`. In conditionals, only `nil` and `false` are falsy; everything else is truthy (including `0`, `""`, and empty collections).

```lisp
true
false
```

### 1.3 Integers

64-bit signed integers. Decimal, hex (`0x`), octal (`0o`), and binary (`0b`) literals are supported. Underscores are not digit separators.

```lisp
42
-17
0xff     ; 255
0o77     ; 63
0b1010   ; 10
```

### 1.4 Floats

64-bit IEEE 754 floats. A decimal point or exponent suffix makes a literal a float. Leading digits before the point are required.

```lisp
3.14
-0.5
1e10
2.5e-3
```

### 1.5 Strings

UTF-8 strings. Double-quoted. Escape sequences: `\n`, `\t`, `\r`, `\"`, `\\`.

```lisp
"hello"
"line one\nline two"
```

### 1.6 Symbols

Identifiers that look up values in the environment. Symbols are **case-insensitive**: the reader lowercases them, so `Foo`, `foo`, and `FOO` name the same binding.

Legal characters: letters, digits, and `? ! + - * / = < > _ % &`.

```lisp
x
my-var
empty?
set!
+
```

### 1.7 Keywords

Keywords begin with `:` and evaluate to themselves. They are primarily used as map keys and enum-like tags.

Unlike symbols, keywords are **case-sensitive**: `:OK`, `:ok`, and `:Ok` are three distinct keywords.

```lisp
:ok
:error
:module/name
```

### 1.8 Lists

Ordered sequences, delimited by parentheses. Lists evaluate as calls (see §3). A list in the operator position invokes evaluation — it is code, not just data. To use a list as data without calling it, quote it: `'(1 2 3)`.

```lisp
(1 2 3)
(+ 2 2)
```

### 1.9 Vectors

Ordered sequences, delimited by `#[` and `]`. Vectors evaluate to themselves — each element is evaluated, but the vector itself is not callable. Vectors are **data**, not code; unlike lists, they never trigger evaluation as a call.

```lisp
#[:red :green :blue]
#[1 2 3]
```

A bare `[]` is an **infix expression** (see §2.7), not a vector. Use `#[]` for the empty vector.

### 1.10 Maps

Key-value collections, delimited by braces. Keys and values alternate. Evaluation evaluates each key and value. Maps evaluate to themselves.

Maps are a **first-class value type**, not reader sugar. Unlike infix `[...]` or dot-access `a.b` (which are rewritten at read time), `{...}` produces `Value::Map` directly in the reader.

```lisp
{:name "example"
 :enabled true
 :tags #{:alpha :public}}
```

### 1.11 Sets

Unordered unique collections, delimited by `#{` and `}`. Elements are evaluated. Sets evaluate to themselves.

```lisp
#{:read :write}
```

---

## 2. Reader

The reader converts source text into canonical forms. Sugar lowers at read time — the evaluator never sees surface syntax.

### 2.1 Quoting

`'form` is reader sugar for `(quote form)`.

```lisp
'x          ; => (quote x)
'(1 2 3)    ; => (quote (1 2 3))
```

### 2.2 Comments

`;` starts a line comment. Everything from `;` to end-of-line is discarded. Comments do not survive into canonical form.

```lisp
; this is a comment
(+ 2 2)  ; returns 4
```

### 2.3 Dot Access

`a.b.c` desugars into nested property access:

```lisp
object.position.x
;; lowers to: (. (. object :position) :x)
```

The `.` built-in looks up a keyword key in a map. This works on any map-typed value.

### 2.4 Infix Expressions

Square brackets (no `#` prefix) denote infix expressions. The reader rewrites them into prefix calls using operator precedence.

```lisp
[2 + 2]              ; => (+ 2 2)
[object.count <= 0]  ; => (<= (. object :count) 0)
```

**Precedence table** (lowest to highest):

| Level | Operators |
|-------|-----------|
| 1     | `or` |
| 2     | `and` |
| 3     | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| 4     | `+`, `-` |
| 5     | `*`, `/`, `%` |

Operators at the same level are left-associative:

```lisp
[a - b - c]  ; => (- (- a b) c)
[a + b * c]  ; => (+ a (* b c))
```

If a bracketed form has one element, it returns that element directly. If it has zero elements, it becomes an empty list. If the second element is not a recognised operator, the whole form becomes a vector.

### 2.5 Symbol Matching

Symbol characters: `[a-zA-Z0-9 ? ! + - * / = < > _ % &]`. Symbols may not start with a digit or `:`.

---

## 3. Evaluation

The evaluator takes a canonical form and an environment and produces a value.

**Self-evaluating forms** return themselves: `nil`, booleans, integers, floats, strings, keywords, vectors, maps, sets, built-in functions, closures, and macros.

**Symbols** are looked up in the current lexical environment. If not found, evaluation signals an `UnboundSymbol` error.

**Lists** are calls. The first element determines how the list is evaluated:

1. If it names a **special form**, the evaluator dispatches to that form's rule (arguments may or may not be evaluated).
2. If it names a **macro**, the macro is expanded and the result is re-evaluated.
3. Otherwise, the operator and all arguments are evaluated left-to-right, and the operator (which must be callable) is invoked with the evaluated arguments.

### 3.1 Recursion limit

Evaluation is bounded by a recursion depth limit (default 1024). This prevents infinite recursion from blowing the Rust stack. The limit can be configured via `SandboxOptions` in the host.

---

## 4. Special Forms

Special forms control evaluation order. The evaluator recognises them by name in the operator position — you cannot shadow a special form with a local binding.

### 4.1 `quote`

```lisp
(quote form)
```

Returns `form` unevaluated. The `'` reader sugar is the idiomatic way to write quote.

`quote` is the fundamental mechanism for treating code as data. Its primary use is in **macros**, where it lets you construct a form that will be evaluated later by the caller. Without `quote`, you cannot return code from a macro — because the evaluator would evaluate it before the macro returns.

```lisp
; Without quote: evaluator tries to call (if ...) before returning
(defmacro when [test & body]
  (list 'if test (cons 'do body) nil))

; Each ' stops evaluation so the symbol/form passes through as data
```

### 4.2 `if`

```lisp
(if test then)
(if test then else)
```

Evaluates `test`. If the result is truthy (not `nil` and not `false`), evaluates and returns `then`. Otherwise evaluates and returns `else` (or `nil` if no else branch).

### 4.3 `do`

```lisp
(do form*)
```

Evaluates each form in order. Returns the value of the last form. Useful for sequencing side-effects.

### 4.4 `let`

```lisp
(let name value body*)
```

Evaluates `value`, binds the result to `name` in a new scope, then evaluates `body*` in that scope. Returns the last body value. The binding is visible only within the body.

```lisp
(let x 5
  [x * x])  ; => 25
```

### 4.5 `fn`

```lisp
(fn [param*] body*)
```

Creates a closure that captures the current lexical environment. Parameters are bare symbols (no destructuring).

```lisp
(fn [x] [x * x])

; variadic: & rest collects remaining args into a list
(fn [a & rest] (println! a rest))
```

When called, the closure checks that the argument count matches the parameter count. If `& rest` is present, it accepts at least that many arguments.

### 4.6 `const`

```lisp
(const name value)
```

Binds `name` to the evaluated `value` in the current environment. The binding cannot be redefined — attempting to `const` the same name again signals an error.

```lisp
(const pi 3.14159)
(const square (fn [x] [x * x]))
```

### 4.7 `defmacro`

```lisp
(defmacro name [param*] body*)
```

Defines a macro in the current environment. Macros receive their arguments **unevaluated** and must return a form that will be evaluated. Macros expand at call sites before evaluation.

```lisp
(defmacro when [test & body]
  (list 'if test
        (cons 'do body)
        nil))

(when [x > 0]
  (println! "positive"))
```

### 4.8 `set!`

```lisp
(set! place value)
```

Mutates an existing binding. `place` is either:
- A symbol: updates the nearest scope containing that name.
- A `.` call like `(. map :key)`: updates the value at that key in a mutable map.

```lisp
(const counter 0)
(set! counter [counter + 1])

(const data {:count 0})
(set! (. data :count) 10)
```

The `!` suffix is convention, not syntax — it signals a destructive operation.

### 4.9 `and`

```lisp
(and form*)
```

Evaluates forms left-to-right. Short-circuits on the first falsy value (returns `nil` or `false`). If all forms are truthy, returns the value of the last form.

```lisp
(and [x > 0] [x < 10])  ; => true if x in (0, 10)
```

### 4.10 `or`

```lisp
(or form*)
```

Evaluates forms left-to-right. Short-circuits on the first truthy value and returns it. If all forms are falsy, returns `nil`.

```lisp
(or (lookup :cache) (lookup :database))
```

### 4.11 `match`

```lisp
(match expr
  pattern1 body1*
  pattern2 body2*
  ...)
```

Evaluates `expr`, then tries each clause in order. The first pattern that matches has its body evaluated. If no pattern matches, a `PatternMatchFailed` error is signalled.

**Patterns** support:
- Literal values: `nil`, `true`, `false`, integers, floats, strings, keywords — match if `=` holds.
- `_` — matches anything (wildcard).

```lisp
(match status
  :ok    (println! "success")
  :error (println! "failed")
  _      (println! "unknown"))
```

**[Planned]** Nested structural patterns (lists, vectors) and pattern variables (`?name`).

### 4.12 `try` / `catch`

```lisp
(try body* (catch pattern catch-body*))
```

Evaluates `body*`. If evaluation completes without error, returns the last body value. If an error occurs, matches it against `pattern` using the same pattern rules as `match`. If the pattern matches, evaluates `catch-body*`; otherwise the error propagates.

```lisp
(try
  (/ 1 0)
  (catch :divide-by-zero 0))
```

---

## 5. Functions

Functions are first-class values created by `fn`. They close over their definition environment (lexical scoping).

### 5.1 Calling conventions

```lisp
((fn [x] [x * x]) 5)  ; => 25
```

The operator position can be any expression that evaluates to a callable:

```lisp
(const ops {:square (fn [x] [x * x])
            :cube   (fn [x] [x * x * x])})
((. ops :square) 3)  ; => 9
```

### 5.2 Variadic functions

A `&` before the last parameter collects remaining arguments into a list:

```lisp
(const log-all (fn [level & messages]
  (println! "[" level "]" messages)))

(log-all :info "connected" "ready")
```

### 5.3 Built-in functions

Built-in functions are implemented in the host (Rust). They are callable but have no accessible source.

Built-in categories:

**Arithmetic** — `+`, `-`, `*`, `/`, `%`
- Variadic except `%` (exactly 2 args).
- Mixing integers and floats promotes to float.
- `/` does integer division on two integers, float division otherwise.
- Division by zero signals a `:divide-by-zero` error.

**Comparison** — `=`, `!=`, `<`, `>`, `<=`, `>=`
- Variadic. Equality (`=`/`!=`) works across all value types.
- Ordered comparison works on numbers; chaining is supported: `(< 1 x 10)`.

**Collections**
| Function | Signature | Behaviour |
|----------|-----------|-----------|
| `list` | `(list a*)` | Create a list from evaluated args |
| `vector` | `(vector a*)` | Create a vector from evaluated args |
| `cons` | `(cons x xs)` | Prepend `x` to list `xs` |
| `first` | `(first xs)` | First element of a list; errors if empty |
| `rest` | `(rest xs)` | All but the first element |
| `empty?` | `(empty? x)` | True if `x` is an empty list, vector, or string |
| `.` | `(. map :key)` | Look up keyword key in a map |
| `map` | `(map f xs)` | Apply `f` to each element of list `xs`, return list of results |

**I/O**
| Function | Signature | Behaviour |
|----------|-----------|-----------|
| `print!` | `(print! a*)` | Print values to stdout (no newline) |
| `println!` | `(println! a*)` | Print values to stdout with trailing newline |
| `slurp` | `(slurp path)` | Read file at `path` into a string |

**Introspection**
| Function | Signature | Behaviour |
|----------|-----------|-----------|
| `type` | `(type v)` | Return a keyword naming the type (`:nil`, `:int`, `:list`, etc.) |
| `str` | `(str a*)` | Concatenate display representations of all args into a string |

**Meta**
| Function | Signature | Behaviour |
|----------|-----------|-----------|
| `eval` | `(eval form)` | Evaluate a form in the current environment |
| `apply` | `(apply f args)` | Call `f` with list `args` spread as arguments |

---

## 6. Macros

Macros are functions that run at expansion time. They receive **unevaluated** arguments and return a form that replaces the macro call.

Because macros return code that must not be evaluated until the call site, they rely on `quote` (`'`) to construct templates without triggering premature evaluation (see §4.1).

### 6.1 Defining macros

```lisp
(defmacro unless [test & body]
  (list 'if test nil (cons 'do body)))
```

### 6.2 Macro expansion

Macro expansion is recursive: the result of expanding a macro is itself scanned for further macros. The public API is `macroexpand_all`:

```lisp
(macroexpand_all '(unless false (println! "hi")))
; => (if false nil (do (println! "hi")))
```

**[Planned]** `macroexpand` as a built-in accessible from within Glyph.

### 6.3 Quasiquote **[planned]**

```lisp
`(list ~x ~@rest)
; => (list 1 2 3)  ; when x=1, rest=(2 3)
```

Quasiquote with `~` (unquote) and `~@` (splice-unquote) is specified but not yet implemented. The current workaround is manual `list`/`cons` construction.

---

## 7. Environment Model

Glyph uses lexical scoping with parent-linked environments.

- `const` and `defmacro` bind into the current environment.
- `let` creates a new child environment for its body.
- Lookup walks the chain upward to the root.
- `set!` mutates the nearest binding (walking upward).

The root environment is initialised by the host with all built-in functions and special forms.

---

## 8. Sandbox & Host Integration

Glyph is designed to run embedded in a host application (currently: the Xlyph roguelike). The host controls what capabilities Glyph code has.

### 8.1 Recursion limiting

`SandboxOptions` carries a `max_depth` (default 1024). Each nested evaluation call consumes one unit. When exhausted, evaluation signals `RecursionLimit`.

### 8.2 Virtual filesystem

`SandboxOptions` accepts an optional `vfs: Option<Arc<dyn VirtualFileSystem>>`. When set, I/O built-ins (`slurp`) read through the virtual filesystem instead of touching the real filesystem. This lets the host sandbox file access or inject synthetic file contents for testing.

```rust
// Host-side (Rust)
let mut opts = SandboxOptions::default();
opts.vfs = Some(Arc::new(MyVirtualFS));
eval_with_opts(&form, &env, opts);
```

### 8.3 I/O gating

Built-in I/O functions (`slurp`, `print!`, `println!`) exist in the default environment but a host may omit or replace them. The host can construct any environment before evaluation.

### 8.4 Capabilities **[planned]**

The spec envisions explicit capability sets:

```lisp
{:caps #{:read :write :trace}
 :form '(set! object.count 0)}
```

Attempting an operation without the required capability would yield a structured fault. This is not yet enforced by the runtime.

### 8.5 Determinism **[planned]**

The spec recommends that hosts make randomness and time explicit rather than ambient:

```lisp
(rand rng)
(choice rng options)
```

These built-ins are not yet implemented.

---

## 9. Error Model

Errors are represented as values during evaluation.

### 9.1 EvalError variants

| Variant | Condition |
|---------|-----------|
| `UnboundSymbol` | Symbol lookup found nothing |
| `NotCallable` | Operator position did not evaluate to a function/macro |
| `WrongArgCount` | Function called with wrong number of arguments |
| `NotInList` | Variadic arg binding failed |
| `DivisionByZero` | Integer division by zero |
| `RecursionLimit` | Exceeded `max_depth` |
| `TypeError` | Operation on incompatible types |
| `PatternMatchFailed` | No `match` clause matched |
| `Custom` | User/host-defined error |

### 9.2 ReadError variants

| Variant | Condition |
|---------|-----------|
| `UnexpectedEof` | Input ended mid-form |
| `UnexpectedChar` | Illegal character in context |
| `InvalidNumber` | Malformed numeric literal |
| `InvalidEscape` | Unknown string escape sequence |

**[Planned]** Structured fault values (`{:fault :divide-by-zero :form '(...)}`) as an alternative return convention.

---

## 10. Round-Tripping

Glyph's canonical printer (`Display` on `Value`) produces stable prefix-form output suitable for diffs, hashing, and serialization.

**[Planned]** A surface-mode printer that reconstructs infix expressions and dot-access for human editing, without obscuring the underlying tree shape.

---

## 11. Modules **[planned]**

```lisp
(module example/core
  {:version 1
   :exports [run stop configure]})
```

The module system is specified but not implemented. Modules would provide namespacing, export control, and metadata attachment. Currently, all bindings share a single flat environment.

---

## 12. Structural Rewriting **[planned]**

```lisp
(rewrite form
  (replace '(call ?x)
           '(trace (call ?x))))
```

Planned operations: `replace`, `insert-before`, `insert-after`, `wrap`, `remove`, `rename`, `specialize`. These operate on canonical data and do not imply permission to mutate the host.

---

## 13. Style Conventions

These are not enforced by the language but are used throughout the standard environment and recommended for user code:

- `name?` — predicate functions returning booleans (`empty?`, `ready?`)
- `name!` — destructive or effectful operations (`set!`, `print!`, `println!`)

Note: `slurp` is a pure query (reads a file, returns contents) and intentionally lacks the `!` suffix.
- `foo.bar` — property access (lowers to `(. foo :bar)`)
- `kebab-case` — multi-word identifiers
- Keywords for enum-like tags: `:ok`, `:error`, `:missing-capability`

---

## Appendix A: Complete Grammar

```text
form      := literal | collection | symbol | keyword | quote | infix | dot-access
literal   := nil | true | false | integer | float | string
integer   := [-](digit+) | 0x(hex-digit+) | 0o(oct-digit+) | 0b(bin-digit+)
float     := [-](digit+).(digit+)[(e|E)[+|-](digit+)]
string    := "(char*)"
symbol    := [a-zA-Z?%&*+/<=>!_|-][a-zA-Z0-9?%&*+/<=>!_|-]*
keyword   := :symbol
collection:= list | vector | map | set
list      := ( form* )
vector    := #[ form* ]
map       := { form form* }    ; even number of forms
set       := #{ form* }
quote     := 'form
infix     := [ form op form* ]  ; op must be a known infix operator
dot-access:= symbol(.symbol)+
comment   := ;.*$
```

## Appendix B: Quick Reference

| Category | Forms |
|----------|-------|
| Self-evaluating | `nil`, `true`, `false`, numbers, strings, keywords, vectors, maps, sets |
| Special forms | `quote`, `if`, `do`, `let`, `fn`, `const`, `defmacro`, `set!`, `and`, `or`, `match`, `try`/`catch` |
| Arithmetic | `+`, `-`, `*`, `/`, `%` |
| Comparison | `=`, `!=`, `<`, `>`, `<=`, `>=` |
| Collections | `list`, `vector`, `cons`, `first`, `rest`, `empty?`, `.`, `map` |
| I/O | `print!`, `println!`, `slurp` |
| Introspection | `type`, `str` |
| Meta | `eval`, `apply` |
| Reader sugar | `'form` → `(quote form)`, `a.b` → `(. a :b)`, `[a + b]` → `(+ a b)` |

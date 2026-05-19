# Glyph Language Reference

A guided tour of Glyph, the Lisp dialect embedded in Xlyph. If you want the formal grammar and semantics, see `language-spec.md`. This document is meant to be read cover-to-cover.

## Hello, World

Fire up a REPL (or the in-game console) and type:

```lisp
(println "hello, world")
```

`println` is a built-in function. Like everything in Glyph, it's called by putting it at the front of a list: `(function arg1 arg2 ...)`.

## Values at a Glance

Glyph has 13 types. Here are the ones you'll use every day:

```lisp
nil                 ; the empty value — one of two falsy things
true  false         ; booleans — false is the other falsy thing
42  -17  0xff       ; 64-bit integers (decimal, hex, octal, binary)
3.14  -0.5  1e10    ; 64-bit floats
"hello\nworld"      ; UTF-8 strings
my-var  empty?  +   ; symbols — case-insensitive, look up values
:ok  :error         ; keywords — case-sensitive, evaluate to themselves
#[1 2 3]             ; lists — the backbone of code and data. Syntax sugar for (list 1 2 3)
{:x 10 :y 20}       ; maps — key-value pairs
#{"a" "b"}          ; sets — unique unordered collections
```

Everything that isn't `nil` or `false` is truthy — including `0`, `""`, and empty collections.

## Calling Functions

A list whose first element is a callable gets evaluated as a call:

```lisp
(+ 1 2 3)           ; => 6
(str "turn " 42)    ; => "turn 42"
(first (list 1 2))  ; => 1
```

Arguments are evaluated left-to-right before the function runs.

## Infix Expressions

Deeply-nested arithmetic is easier to read with square brackets:

```lisp
[1 + 2 * 3]         ; => 7 — reads as (+ 1 (* 2 3))
[x > 0 and x < 10]  ; => (and (> x 0) (< x 10))
```

Operator precedence follows standard math rules: `* / %` bind tighter than `+ -`, which bind tighter than comparisons, which bind tighter than `and`, which binds tighter than `or`. Same-precedence operators are left-associative.

A bare `[...]` with no operators is just a list — useful for parameter vectors:

```lisp
(fn [x y] (+ x y))  ; [x y] is the parameter list
```

## Quote and Quoting Shortcuts

A `'` in front of any form prevents evaluation:

```lisp
'(1 2 3)            ; => the list (1 2 3), not a function call
'my-symbol          ; => the symbol my-symbol, not its value
```

This is syntax sugar — `'x` reads as `(quote x)`.

## Dot Notation for Map Access

A chain of dotted names desugars into nested `.` calls:

```lisp
player.hp            ; reads as (. player :hp)
a.b.c                ; reads as (. (. a :b) :c)
```

This only works with symbol-like chains — you can't dot through arbitrary expressions.

## Comments

```lisp
; everything from a semicolon to end of line is a comment
(+ 1 2)  ; => 3
```

## Binding Names

### `const` — Define Once

`const` binds a name in the current scope and refuses to *redefine* it (no re-`const` on the same name). The binding itself can still be mutated with `set!` — `const` is "bind once," not "immutable value."

```lisp
(const answer 42)
(const add1 (fn [x] (+ x 1)))
```

### `let` — Local Bindings

`let` creates a child scope, binds one name, and evaluates a body:

```lisp
(let x 10
  (let y 20
    (+ x y)))        ; => 30
; x and y are not visible out here
```

The value is evaluated *before* entering the new scope, so you can't reference other let-bindings in the same chain. Nest them instead.

## Functions

### Creating Functions with `fn`

`fn` creates a closure, capturing the current lexical environment:

```lisp
;; Single arity
(fn [x] (+ x 1))

;; Shorthand for single-parameter functions — drop the brackets:
(fn x (+ x 1))

;; Variadic — & collects remaining args into a list:
(fn [x & rest] (cons x rest))

;; Multi-arity — each clause is its own list:
(fn ([x] x)
    ([x y] (+ x y))
    ([x y z] (* x y z)))
```

`fn` produces an anonymous function. Use it inline or give it a name:

```lisp
;; Anonymous, called immediately:
((fn [x] (* x 2)) 21)    ; => 42

;; Named:
(const double (fn [x] (* x 2)))
```

There is no `defn` — `const` + `fn` is the idiomatic pattern.

### Tail Recursion with `recur`

`recur` jumps back to the top of the current function without growing the stack. It must appear in tail position (as the last thing the function does):

```lisp
(const countdown (fn [n acc]
  (if (= n 0)
      acc
      (recur (- n 1) (+ acc n)))))

(countdown 1000 0)   ; => 500500 — no stack overflow
```

### The `lambda` Macro

The prelude provides `lambda` as a friendlier alias for `fn`:

```lisp
(lambda [x] (+ x 1))          ; same as (fn [x] (+ x 1))
(lambda body)                 ; zero-arg function returning body
```

## Control Flow

### `if`

```lisp
(if condition
    then-expression
    else-expression)    ; else is optional — defaults to nil
```

### `and` and `or`

Short-circuiting logical operators:

```lisp
(and (positive? x) (< x 100))   ; stops at first falsy value
(or a b c)                       ; stops at first truthy value
```

### `do`

Evaluate expressions in sequence, return the last:

```lisp
(do
  (println "starting...")
  (do-something)
  (println "done"))
```

### `match`

Pattern-match against literals and wildcards:

```lisp
(match status
  :ok    (println "success")
  :error (println "failed")
  _      (println "unknown"))   ; _ matches anything
```

## Error Handling

```lisp
(try
  (risky-operation)
  (catch DivisionByZero
    (println "oops")))
```

The catch clause matches on error type tags. If the body succeeds, `try` returns its value. If it throws, the matching catch clause runs.

## Mutation

`set!` mutates an existing binding — including one created by `const`:

```lisp
(const counter 0)
(set! counter 5)              ; mutates the binding to 5

(const player {:hp 10})
(set! (. player :hp) 5)       ; mutates the :hp entry in the map
```

You can only `set!` a name that already exists. For map entries, use the `.` accessor as the place.

## Collections

### Lists

```lisp
(list 1 2 3)          ; construct a list
(cons 0 (list 1 2))   ; prepend => (0 1 2)
(first my-list)        ; first element
(rest my-list)         ; everything but the first
(empty? my-list)      ; true if empty
```

### Maps

```lisp
{:name "goblin" :hp 5}          ; literal map
(. entity :hp)                   ; access by keyword => 5
```

### Sets

```lisp
#{"a" "b" "c"}         ; literal set
```

## The Standard Prelude

These are written in Glyph itself and loaded at startup:

| Function | What it does |
|----------|-------------|
| `(second lst)` | Second element |
| `(nth lst n)` | Zero-indexed element access |
| `(filter pred lst)` | Keep elements matching predicate |
| `(reduce f init lst)` | Left fold |
| `(some pred lst)` | First truthy result of predicate, or nil |
| `(every pred lst)` | True if all match predicate |
| `(take n lst)` | First n elements |
| `(drop n lst)` | All but first n elements |
| `(append lst x)` | Add element to end |
| `(concat a b)` | Concatenate two lists |
| `(reverse lst)` | Reverse a list |
| `(range end)` / `(range start end)` / `(range start end step)` | Numeric range |

### `repeat` Macro

```lisp
(repeat 3 (println "hi"))    ; prints "hi" three times
```

## Built-in Function Reference

### Arithmetic

| Function | Description |
|----------|-------------|
| `(+ a b ...)` | Sum. Mixed int/float promotes to float. |
| `(- a b ...)` | Subtraction |
| `(* a b ...)` | Product |
| `(/ a b)` | Integer division for two ints, float division otherwise. Signals `DivisionByZero`. |
| `(% a b)` | Integer remainder |

### Comparison

| Function | Description |
|----------|-------------|
| `(= a b ...)` | Structural equality. Chaining: `(= 1 1 1)` is true. |
| `(!= a b)` | Not equal |
| `(< a b ...)` | Less than. Chaining: `(< 1 x 5)` is true if 1 < x < 5. |
| `(> a b ...)` | Greater than |
| `(<= a b ...)` | Less than or equal |
| `(>= a b ...)` | Greater than or equal |

### Introspection

| Function | Description |
|----------|-------------|
| `(type x)` | Returns a keyword: `:nil`, `:bool`, `:int`, `:float`, `:string`, `:symbol`, `:keyword`, `:list`, `:map`, `:set`, `:builtin`, `:fn`, `:macro` |
| `(str a b ...)` | Concatenate display representations |

### I/O

| Function | Description |
|----------|-------------|
| `(print a b ...)` | Print to stdout (no newline) |
| `(println a b ...)` | Print with trailing newline |
| `(slurp path)` | Read file as string. Sandboxed by VFS. |

### Meta

| Function | Description |
|----------|-------------|
| `(eval form)` | Evaluate a form in the current environment |
| `(apply f a b [c d])` | Call f with args `[a b c d]`. Last arg must be a list. |
| `(map f lst)` | Apply function to each element, return list of results |

## Macros

Macros receive their arguments unevaluated and return a form that gets evaluated in the caller's environment:

```lisp
(defmacro unless [test & body]
  (list 'if test nil (cons 'do body)))

(unless (empty? items)
  (println "got items"))
```

The `defmacro` form looks like `fn` but its body runs at compile time and must produce code.

## Game-Specific Features

### `bind-key`

Bind a single-character keyword key to a Glyph expression that re-evaluates on each keypress:

```lisp
(bind-key :f (println "you pressed f"))
```

The expression is stored as source text — it is not evaluated at bind time.

### AI Builtins

Available in enemy AI rules (not in the REPL):

| Function | Description |
|----------|-------------|
| `(adjacent? entity-id target-id)` | Manhattan distance == 1? |
| `(attack! attacker-id target-id damage)` | Deal damage |
| `(step-toward! entity-id target-id)` | A* path one step toward target |
| `(random-step! entity-id)` | Move to random adjacent walkable tile |
| `(flee-step! entity-id threat-id)` | Move one step away from threat |
| `(roll-odds? entity-id probability)` | Deterministic chance check |
| `(hp entity-id)` | Current HP |

## Idioms and Patterns

**Threading state through recursion:**

```lisp
(const sum (fn [lst]
  (if (empty? lst)
      0
      [(first lst) + (sum (rest lst))])))
```

**Building a list with cons and reverse:**

```lisp
(const map (fn [f lst]
  (reverse (reduce (fn [acc x] (cons (f x) acc)) (list) lst))))
```

**Multi-arity for default arguments:**

```lisp
(const greet (fn ([name] (greet name "hello"))
                  ([name greeting] (println greeting ", " name))))
```

**Using match for dispatching:**

```lisp
(const describe (fn [x]
  (match (type x)
    :int    "a number"
    :string "some text"
    :list   "a list"
    _       "something else")))
```

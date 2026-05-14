# Glyph Language Specification

Status: concept draft

Glyph is a homoiconic, Lisp-derived language with a friendlier surface syntax. It is designed for live systems where code can be represented, inspected, transformed, serialized, diffed, and evaluated as ordinary data.

The language does not define a game, a spell system, a level progression model, or a patch economy. Those are host-application concerns. Glyph only defines the syntax, canonical data model, macro system, evaluator expectations, and optional runtime primitives that a host can choose to expose.

## Design Goals

- Preserve homoiconicity: code is ordinary data after reading.
- Keep the canonical form small, regular, and easy to transform.
- Support both prefix Lisp forms and lightweight infix syntax.
- Make syntax sugar lower into obvious canonical trees.
- Support macros as structural transformations over canonical forms.
- Support deterministic execution when the host provides deterministic effects.
- Keep host capabilities explicit rather than implicit.

## Non-Goals

- Teaching programming concepts.
- Defining any particular game mechanic.
- Defining progression, unlocks, balance, or content generation.
- Providing unrestricted host, filesystem, network, or OS access.
- Making natural language part of the language runtime.

## Core Model

Every source file is read into a canonical abstract syntax tree. The AST is built from ordinary values:

- nil
- booleans
- numbers
- strings
- symbols
- keywords
- lists
- vectors
- maps
- sets

The evaluator only evaluates canonical forms. Surface syntax is sugar.

```lisp
[2 + 2]
```

lowers to:

```lisp
(+ 2 2)
```

The canonical tree, not the original source text, is the authoritative form used for evaluation, transformation, serialization, hashing, and diffing.

## Reader Pipeline

```text
source text
  -> reader/parser
  -> canonical forms
  -> macro expansion
  -> optional host validation
  -> evaluation or compilation
```

The reader is responsible for syntax sugar. Macros operate only on canonical forms.

## Syntax Overview

### Prefix Calls

Prefix forms are always valid.

```lisp
(print "hello")
(+ 2 2)
(if [x <= 0] :non-positive :positive)
```

### Infix Forms

Square brackets denote infix expressions.

```lisp
[2 + 2]
[object.count <= 0]
[ready? object and object.kind != :sealed]
```

These lower to:

```lisp
(+ 2 2)
(<= (. object :count) 0)
(and (ready? object) (!= (. object :kind) :sealed))
```

Infix forms are expression-only. They do not introduce a separate data type.

### Property Access

Dot syntax lowers into canonical property access.

```lisp
object.position.x
```

lowers to:

```lisp
(. (. object :position) :x)
```

Property assignment is explicit:

```lisp
(set! object.count [object.count - 1])
```

lowers to:

```lisp
(set! (. object :count) (- (. object :count) 1))
```

### Keywords

Keywords begin with `:`.

```lisp
:ok
:error
:module/name
```

Keywords evaluate to themselves.

### Vectors

Vectors use `#[]`.

```lisp
#[:red :green :blue]
```

Square brackets without `#` are reserved for infix expressions.

### Maps

Maps use braces.

```lisp
{:name "example"
 :enabled true
 :tags #{:alpha :public}}
```

### Sets

Sets use `#{}`.

```lisp
#{:read :write}
```

### Strings

Strings are UTF-8.

```lisp
"hello"
```

### Comments

Line comments begin with `;`.

```lisp
; This comment is source-only and is not preserved in canonical form.
```

Doc metadata should be represented explicitly if it must survive round-tripping.

## Canonical Special Forms

The minimum special forms are:

```lisp
(quote form)
(if test then else)
(do form*)
(let [binding*] body*)
(fn [param*] body*)
(def name value)
(defmacro name [param*] body*)
(set! place value)
(try body catch-form)
```

Reader shorthand:

```lisp
'form
```

lowers to:

```lisp
(quote form)
```

## Functions

Functions are first-class values.

```lisp
(def square
  (fn [x] [x * x]))
```

Function calls evaluate the operator and arguments left to right unless the operator is a special form or macro.

```lisp
(square 9)
```

## Macros

Macros receive unevaluated canonical forms and return canonical forms.

```lisp
(defmacro when [test & body]
  `(if ~test
     (do ~@body)
     nil))
```

Macros transform trees. They do not automatically gain access to host effects. A host may validate macro output before compilation or evaluation.

## Quasiquote

The language supports quasiquote for structural code generation.

```lisp
`(call ~target ~argument)
```

Splice unquote is allowed inside list and vector contexts.

```lisp
`(do ~@forms)
```

## Pattern Matching

Pattern variables begin with `?`.

```lisp
(match form
  ['(call ?target ?argument)
   {:target ?target :argument ?argument}]

  [_ nil])
```

Patterns match data structures, not source text.

## Structural Rewrite Forms

Glyph may provide generic structural rewrite helpers. These operate on data and do not imply permission to mutate any host system.

```lisp
(rewrite form
  (replace '(call ?x)
           '(trace (call ?x))))
```

Recommended rewrite operations:

- `replace`
- `insert-before`
- `insert-after`
- `wrap`
- `remove`
- `rename`
- `specialize`

Whether a rewritten form can be installed anywhere is a host concern.

## Capabilities

Glyph can model effects through capability sets, but the language does not define a fixed capability taxonomy. Hosts define their own capability names and enforcement rules.

Example:

```lisp
{:caps #{:read :write :trace}
 :form '(set! object.count 0)}
```

Attempting an operation without a required capability should yield a structured fault when the host chooses capability enforcement.

```lisp
{:fault :missing-capability
 :required :write
 :form '(set! object.count 0)}
```

## Determinism

Glyph itself should not require nondeterminism. Deterministic execution depends on host-provided effects.

Recommended host rules:

- make randomness explicit
- make time explicit
- serialize accepted code as canonical data
- avoid hidden ambient effects
- keep validation deterministic

Randomness, if exposed, should use explicit handles.

```lisp
(rand rng)
(choice rng options)
```

## Errors And Faults

Errors should be representable as values.

```lisp
{:fault :divide-by-zero
 :form '(/ 1 0)}
```

A host may decide whether a fault:

- returns nil
- aborts the current evaluation
- rolls back a transaction
- disables a loaded module
- escalates to a fatal error

## Introspection

Glyph should expose structural introspection primitives.

```lisp
(source symbol-or-value)
(macroexpand '(when ready (run task)))
(caps value)
(trace function argument)
(diff old-form new-form)
```

Hosts may restrict introspection. The language only defines how inspectable data is represented once access is granted.

## Modules

Modules are namespaces plus metadata.

```lisp
(module example/core
  {:version 1
   :exports [run stop configure]})
```

A host may attach additional metadata such as signatures, provenance, visibility, trust level, or validation status.

## Round-Tripping

Canonical forms should be printable in two modes:

- `canonical`: stable prefix form for diffs, hashes, saves, and debugging.
- `surface`: readable mixed syntax for human editing.

The canonical printer is lossless. The surface printer may choose friendly syntax, but it must never obscure the tree shape.

## Example

Surface:

```lisp
(defn clamp [x low high]
  (if [x < low]
    low
    (if [x > high]
      high
      x)))
```

Canonical:

```lisp
(defn clamp
  [x low high]
  (if (< x low)
    low
    (if (> x high)
      high
      x)))
```

Structural rewrite:

```lisp
(rewrite '(if (< x low) low x)
  (wrap '(< ?a ?b)
        '(trace ~form)))
```

The same program can be displayed as source, transformed as data, validated by a host, evaluated, serialized, diffed, and replayed.


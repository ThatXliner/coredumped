//! Glyph — a Lisp for live systems.
//!
//! This module implements the Glyph language: reader, evaluator, core types,
//! macros, and built-in functions.

pub mod env;
pub mod eval;
pub mod highlight;
#[cfg(feature = "prelude")]
pub mod prelude;
pub mod reader;
pub mod value;

pub use env::Env;
pub use eval::{default_env, eval, eval_with_opts, macroexpand_all};
pub use reader::read_string;
pub use value::{
    BuiltinFn, ClosureData, EvalError, EvalResult, Keyword, MacroData, ReadError, ReadResult,
    SandboxOptions, Symbol, Value,
};

/// Create a symbol value.
pub fn sym(name: &str) -> Value {
    Value::Symbol(Symbol::new(name))
}

/// Create a keyword value.
pub fn kw(name: &str) -> Value {
    Value::Keyword(Keyword {
        name: name.to_string(),
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::value::*;
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn eval_str(s: &str, env: &Env) -> EvalResult<Value> {
        let forms = read_string(s).unwrap();
        let mut world = crate::world::World::minimal();
        let mut result = Value::Nil;
        for form in forms {
            result = eval(&form, env, &mut world)?;
        }
        Ok(result)
    }

    mod reader {
        use super::*;

        #[test]
        fn test_nil() {
            let f = read_string("nil").unwrap();
            assert_eq!(f, vec![Value::Nil]);
        }
        #[test]
        fn test_bools() {
            let f = read_string("true false").unwrap();
            assert_eq!(f, vec![Value::Bool(true), Value::Bool(false)]);
        }

        #[test]
        fn test_ints() {
            let f = read_string("0 42 -17 0xff 0o77 0b1010").unwrap();
            assert_eq!(
                f,
                vec![
                    Value::I64(0),
                    Value::I64(42),
                    Value::I64(-17),
                    Value::I64(255),
                    Value::I64(63),
                    Value::I64(10)
                ]
            );
        }

        #[test]
        fn test_floats() {
            let f = read_string("3.14 -0.5 1e10").unwrap();
            assert_eq!(
                f,
                vec![Value::F64(3.14), Value::F64(-0.5), Value::F64(1e10)]
            );
        }

        #[test]
        fn test_strings() {
            let f = read_string(r#""hello" "a\nb""#).unwrap();
            assert_eq!(f[0], Value::String("hello".into()));
            assert_eq!(f[1], Value::String("a\nb".into()));
        }
        #[test]
        fn test_keywords() {
            let f = read_string(":ok :module/name").unwrap();
            assert_eq!(f[0], kw("ok"));
            assert_eq!(f[1], kw("module/name"));
        }

        #[test]
        fn test_symbols() {
            let f = read_string("hello world? set! + <=> &rest").unwrap();
            assert_eq!(f[0], sym("hello"));
            assert_eq!(f[1], sym("world?"));
            assert_eq!(f[2], sym("set!"));
            assert_eq!(f[3], sym("+"));
            assert_eq!(f[4], sym("<=>"));
            assert_eq!(f[5], sym("&rest"));
        }

        #[test]
        fn test_case_insensitive() {
            let f = read_string("Foo FOO foo").unwrap();
            for x in f {
                assert_eq!(x, sym("foo"));
            }
        }

        #[test]
        fn test_lists() {
            let f = read_string("(+ 1 (* 2 3))").unwrap();
            assert_eq!(
                f[0],
                Value::List(vec![
                    sym("+"),
                    Value::I64(1),
                    Value::List(vec![sym("*"), Value::I64(2), Value::I64(3)])
                ])
            );
        }

        #[test]
        fn test_vector() {
            let f = read_string("#[1 2 3]").unwrap();
            assert_eq!(
                f[0],
                Value::Vector(vec![Value::I64(1), Value::I64(2), Value::I64(3)])
            );
        }

        #[test]
        fn test_map() {
            let f = read_string("{:a 1 :b 2}").unwrap();
            let mut m = BTreeMap::new();
            m.insert(kw("a"), Value::I64(1));
            m.insert(kw("b"), Value::I64(2));
            assert_eq!(f[0], Value::Map(m));
        }

        #[test]
        fn test_set() {
            let f = read_string("#{:a :b}").unwrap();
            let mut s = BTreeSet::new();
            s.insert(kw("a"));
            s.insert(kw("b"));
            assert_eq!(f[0], Value::Set(s));
        }

        #[test]
        fn test_quote() {
            let f = read_string("'(1 2 3)").unwrap();
            assert_eq!(
                f[0],
                Value::List(vec![
                    sym("quote"),
                    Value::List(vec![Value::I64(1), Value::I64(2), Value::I64(3)])
                ])
            );
        }

        #[test]
        fn test_dotted_access() {
            let f = read_string("a.b.c").unwrap();
            assert_eq!(
                f[0],
                Value::List(vec![
                    sym("."),
                    Value::List(vec![sym("."), sym("a"), kw("b")]),
                    kw("c")
                ])
            );
        }

        #[test]
        fn test_infix_basic() {
            let f = read_string("[2 + 2]").unwrap();
            assert_eq!(
                f[0],
                Value::List(vec![sym("+"), Value::I64(2), Value::I64(2)])
            );
        }
        #[test]
        fn test_infix_precedence() {
            let f = read_string("[a + b * c]").unwrap();
            assert_eq!(
                f[0],
                Value::List(vec![
                    sym("+"),
                    sym("a"),
                    Value::List(vec![sym("*"), sym("b"), sym("c")])
                ])
            );
        }
        #[test]
        fn test_infix_left_assoc() {
            let f = read_string("[a - b - c]").unwrap();
            assert_eq!(
                f[0],
                Value::List(vec![
                    sym("-"),
                    Value::List(vec![sym("-"), sym("a"), sym("b")]),
                    sym("c")
                ])
            );
        }
        #[test]
        fn test_comment() {
            let f = read_string("(+ 1 2) ; comment\n (+ 3 4)").unwrap();
            assert_eq!(f.len(), 2);
        }
        #[test]
        fn test_multiple_forms() {
            let f = read_string("(a) (b) (c)").unwrap();
            assert_eq!(f.len(), 3);
        }
    }

    mod eval_tests {
        use super::*;

        #[test]
        fn test_self_evaluating() {
            let env = default_env();
            assert_eq!(eval_str("nil", &env).unwrap(), Value::Nil);
            assert_eq!(eval_str("true", &env).unwrap(), Value::Bool(true));
            assert_eq!(eval_str("42", &env).unwrap(), Value::I64(42));
            assert_eq!(
                eval_str("\"hi\"", &env).unwrap(),
                Value::String("hi".into())
            );
            assert_eq!(eval_str(":ok", &env).unwrap(), kw("ok"));
        }

        #[test]
        fn test_quote() {
            let env = default_env();
            assert_eq!(
                eval_str("'(1 2 3)", &env).unwrap(),
                Value::List(vec![Value::I64(1), Value::I64(2), Value::I64(3)])
            );
        }
        #[test]
        fn test_arithmetic() {
            let env = default_env();
            assert_eq!(eval_str("(+ 1 2)", &env).unwrap(), Value::I64(3));
            assert_eq!(eval_str("(* 2 3)", &env).unwrap(), Value::I64(6));
            assert_eq!(eval_str("(- 10 4)", &env).unwrap(), Value::I64(6));
            assert_eq!(eval_str("(/ 10 3)", &env).unwrap(), Value::I64(3));
            assert_eq!(eval_str("(+ 1 2 3)", &env).unwrap(), Value::I64(6));
        }
        #[test]
        fn test_float_arithmetic() {
            let env = default_env();
            assert_eq!(eval_str("(/ 7.0 2)", &env).unwrap(), Value::F64(3.5));
            assert_eq!(eval_str("(/ 7 2)", &env).unwrap(), Value::I64(3));
        }
        #[test]
        fn test_nested_infix() {
            let env = default_env();
            assert_eq!(eval_str("[2 + 2]", &env).unwrap(), Value::I64(4));
            assert_eq!(eval_str("[2 + 3 * 4]", &env).unwrap(), Value::I64(14));
            assert_eq!(eval_str("[[1 + 2] * 3]", &env).unwrap(), Value::I64(9));
        }
        #[test]
        fn test_comparison() {
            let env = default_env();
            assert_eq!(eval_str("(= 1 1)", &env).unwrap(), Value::Bool(true));
            assert_eq!(eval_str("(= 1 2)", &env).unwrap(), Value::Bool(false));
            assert_eq!(eval_str("(< 1 2 3)", &env).unwrap(), Value::Bool(true));
            assert_eq!(eval_str("(<= 1 1 2)", &env).unwrap(), Value::Bool(true));
        }
        #[test]
        fn test_if() {
            let env = default_env();
            assert_eq!(eval_str("(if true :ok :no)", &env).unwrap(), kw("ok"));
            assert_eq!(eval_str("(if false :ok :no)", &env).unwrap(), kw("no"));
            assert_eq!(eval_str("(if nil :ok)", &env).unwrap(), Value::Nil);
        }
        #[test]
        fn test_do() {
            let env = default_env();
            assert_eq!(eval_str("(do 1 2 3)", &env).unwrap(), Value::I64(3));
        }
        #[test]
        fn test_let() {
            let env = default_env();
            assert_eq!(eval_str("(let x 10 x)", &env).unwrap(), Value::I64(10));
            assert_eq!(
                eval_str("(let x 10 (let y [x + 2] [x + y]))", &env).unwrap(),
                Value::I64(22)
            );
        }
        #[test]
        fn test_fn() {
            let env = default_env();
            assert_eq!(
                eval_str("((fn [x] [x + 1]) 5)", &env).unwrap(),
                Value::I64(6)
            );
        }
        #[test]
        fn test_closure() {
            let env = default_env();
            assert_eq!(
                eval_str("(let add5 (fn [x] [x + 5]) (add5 3))", &env).unwrap(),
                Value::I64(8)
            );
        }
        #[test]
        fn test_const() {
            let env = default_env();
            eval_str("(const pi 3.14)", &env).unwrap();
            assert_eq!(eval_str("pi", &env).unwrap(), Value::F64(3.14));
        }
        #[test]
        fn test_const_no_redefine() {
            let env = default_env();
            eval_str("(const x 1)", &env).unwrap();
            assert!(eval_str("(const x 2)", &env).is_err());
        }
        #[test]
        fn test_variadic_fn() {
            let env = default_env();
            assert_eq!(
                eval_str("((fn [& args] args) 1 2 3)", &env).unwrap(),
                Value::List(vec![Value::I64(1), Value::I64(2), Value::I64(3)])
            );
        }
        #[test]
        fn test_and_or() {
            let env = default_env();
            assert_eq!(
                eval_str("(and true true)", &env).unwrap(),
                Value::Bool(true)
            );
            assert_eq!(
                eval_str("(and true false)", &env).unwrap(),
                Value::Bool(false)
            );
            assert_eq!(eval_str("(or false nil :ok)", &env).unwrap(), kw("ok"));
            assert_eq!(eval_str("(or false nil)", &env).unwrap(), Value::Nil);
        }
        #[test]
        fn test_map_access() {
            let env = default_env();
            assert_eq!(eval_str("(. {:a 1} :a)", &env).unwrap(), Value::I64(1));
        }
        #[test]
        fn test_dotted_access_eval() {
            let env = default_env();
            assert_eq!(
                eval_str("(. (. {:a {:b 3}} :a) :b)", &env).unwrap(),
                Value::I64(3)
            );
        }
        #[test]
        fn test_match_literal() {
            let env = default_env();
            assert_eq!(
                eval_str("(match 1 [1 :one] [_ :other])", &env).unwrap(),
                kw("one")
            );
            assert_eq!(
                eval_str("(match 2 [1 :one] [_ :other])", &env).unwrap(),
                kw("other")
            );
        }
        #[test]
        fn test_match_no_match_errors() {
            let env = default_env();
            assert!(eval_str("(match 1)", &env).is_err());
        }
        #[test]
        fn test_set_on_map() {
            let env = default_env();
            assert_eq!(
                eval_str("(let m {:count 0} (set! (. m :count) 5) m)", &env).unwrap(),
                Value::Map(BTreeMap::from([(kw("count"), Value::I64(5))]))
            );
        }
        #[test]
        fn test_try_success() {
            let env = default_env();
            assert_eq!(
                eval_str("(try (+ 1 2) (catch _ :err))", &env).unwrap(),
                Value::I64(3)
            );
        }
        #[test]
        fn test_first_rest_cons() {
            let env = default_env();
            assert_eq!(eval_str("(first '(1 2 3))", &env).unwrap(), Value::I64(1));
            assert_eq!(
                eval_str("(rest '(1 2 3))", &env).unwrap(),
                Value::List(vec![Value::I64(2), Value::I64(3)])
            );
        }
        #[test]
        fn test_empty_list_errors() {
            let env = default_env();
            assert!(eval_str("()", &env).is_err());
        }
        #[test]
        fn test_unbound_symbol() {
            let env = default_env();
            assert!(eval_str("no-such-symbol", &env).is_err());
        }

        #[test]
        fn test_macro_unless() {
            let env = default_env();
            eval_str(
                "(defmacro unless [test & body] (list (quote if) test nil (cons (quote do) body)))",
                &env,
            )
            .unwrap();
            assert_eq!(eval_str("(unless false 42)", &env).unwrap(), Value::I64(42));
            assert_eq!(eval_str("(unless true 99)", &env).unwrap(), Value::Nil);
        }

        #[test]
        fn test_macro_identity() {
            let env = default_env();
            eval_str("(defmacro identity [x] x)", &env).unwrap();
            assert_eq!(eval_str("(identity (+ 1 2))", &env).unwrap(), Value::I64(3));
        }

        #[test]
        fn test_macro_unevaluated_args() {
            let env = default_env();
            eval_str("(defmacro my-list [& args] (cons (quote list) args))", &env).unwrap();
            let r = eval_str("(my-list 1 2 (+ 1 2))", &env).unwrap();
            assert_eq!(
                r,
                Value::List(vec![Value::I64(1), Value::I64(2), Value::I64(3)])
            );
        }

        #[test]
        fn test_range() {
            let env = default_env();
            assert_eq!(
                eval_str("(range 5)", &env).unwrap(),
                Value::List(vec![
                    Value::I64(0),
                    Value::I64(1),
                    Value::I64(2),
                    Value::I64(3),
                    Value::I64(4)
                ])
            );
            assert_eq!(
                eval_str("(range 3 7)", &env).unwrap(),
                Value::List(vec![
                    Value::I64(3),
                    Value::I64(4),
                    Value::I64(5),
                    Value::I64(6)
                ])
            );
            assert_eq!(
                eval_str("(range 0 10 2)", &env).unwrap(),
                Value::List(vec![
                    Value::I64(0),
                    Value::I64(2),
                    Value::I64(4),
                    Value::I64(6),
                    Value::I64(8)
                ])
            );
            assert_eq!(
                eval_str("(range 5 0 -1)", &env).unwrap(),
                Value::List(vec![
                    Value::I64(5),
                    Value::I64(4),
                    Value::I64(3),
                    Value::I64(2),
                    Value::I64(1)
                ])
            );
        }
    }
}

//! Evaluator for Glyph: special forms, built-ins, default environment.

use bracket_lib::prelude::{GREEN, RGB};

use super::env::Env;
use super::value::*;
use crate::world::World;

/// Evaluate a form with default sandbox options.
pub fn eval(form: &Value, env: &Env, world: &mut World) -> EvalResult<Value> {
    eval_with_opts(form, env, SandboxOptions::default(), world)
}

/// Evaluate a form with custom sandbox options.
pub fn eval_with_opts(
    form: &Value,
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    match opts.descend() {
        Some(next) => eval_inner(form, env, next, world),
        None => Err(EvalError::RecursionLimit),
    }
}

fn eval_inner(
    form: &Value,
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    match form {
        Value::Nil
        | Value::Bool(_)
        | Value::I64(_)
        | Value::F64(_)
        | Value::String(_)
        | Value::Keyword(_)
        | Value::Vector(_)
        | Value::Map(_)
        | Value::Set(_)
        | Value::Builtin(_)
        | Value::Closure(_)
        | Value::Macro(_) => Ok(form.clone()),

        Value::Symbol(s) => env
            .lookup(&s.name)
            .ok_or_else(|| EvalError::UnboundSymbol(s.name.clone())),

        Value::List(items) => {
            if items.is_empty() {
                return Err(EvalError::Custom("empty list".into()));
            }
            if let Some(expanded) = try_macroexpand_inner(
                form,
                env,
                opts.descend().ok_or(EvalError::RecursionLimit)?,
                world,
            )? {
                return eval_inner(
                    &expanded,
                    env,
                    opts.descend().ok_or(EvalError::RecursionLimit)?,
                    world,
                );
            }
            if let Value::Symbol(s) = &items[0] {
                match s.name.as_str() {
                    "quote" => return eval_quote(&items[1..], env),
                    "if" => {
                        return eval_if_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "do" => {
                        return eval_do_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "let" => {
                        return eval_let_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "fn" => return eval_fn(&items[1..], env),
                    "const" => {
                        return eval_const_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "defmacro" => return eval_defmacro(&items[1..], env),
                    "set!" => {
                        return eval_set_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "try" => {
                        return eval_try_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "and" => {
                        return eval_and_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "or" => {
                        return eval_or_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "match" => {
                        return eval_match_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "bind-key" => {
                        return eval_bind_key_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    "recur" => {
                        return eval_recur_inner(
                            &items[1..],
                            env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )
                    }
                    _ => {}
                }
            }
            eval_call_inner(
                form,
                env,
                opts.descend().ok_or(EvalError::RecursionLimit)?,
                world,
            )
        }
    }
}

// --- Macro expansion ---

fn try_macroexpand_inner(
    form: &Value,
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Option<Value>> {
    let items = match form {
        Value::List(items) => items,
        _ => return Ok(None),
    };
    if items.is_empty() {
        return Ok(None);
    }
    let op = match &items[0] {
        Value::Symbol(s) => s,
        _ => return Ok(None),
    };
    match env.lookup(&op.name) {
        Some(Value::Macro(m)) => {
            let macro_env = Env::extend(&m.env);
            let args = &items[1..];
            bind_params(&m.params, args, &macro_env)?;
            let mut expanded = Value::Nil;
            for expr in &m.body {
                expanded = eval_inner(
                    expr,
                    &macro_env,
                    opts.descend().ok_or(EvalError::RecursionLimit)?,
                    world,
                )?;
            }
            match try_macroexpand_inner(
                &expanded,
                env,
                opts.descend().ok_or(EvalError::RecursionLimit)?,
                world,
            )? {
                Some(further) => Ok(Some(further)),
                None => Ok(Some(expanded)),
            }
        }
        _ => Ok(None),
    }
}

/// Expand all macros in a form (public helper).
pub fn macroexpand_all(form: &Value, env: &Env, world: &mut World) -> EvalResult<Value> {
    let opts = SandboxOptions::default();
    match try_macroexpand_inner(
        form,
        env,
        opts.descend().ok_or(EvalError::RecursionLimit)?,
        world,
    )? {
        Some(expanded) => match &expanded {
            Value::List(items) => {
                let new: EvalResult<Vec<Value>> = items
                    .iter()
                    .map(|x| macroexpand_all(x, env, world))
                    .collect();
                Ok(Value::List(new?))
            }
            _ => Ok(expanded),
        },
        None => Ok(form.clone()),
    }
}

// --- Function application ---

fn eval_call_inner(
    form: &Value,
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    let items = match form {
        Value::List(items) => items,
        _ => unreachable!(),
    };
    let mut evaled = Vec::with_capacity(items.len());
    for item in items {
        evaled.push(eval_inner(
            item,
            env,
            opts.descend().ok_or(EvalError::RecursionLimit)?,
            world,
        )?);
    }
    let callee = evaled.remove(0);
    apply_inner(
        &callee,
        &evaled,
        env,
        opts.descend().ok_or(EvalError::RecursionLimit)?,
        world,
    )
}

fn find_arity<'a>(arities: &'a [Arity], nargs: usize) -> EvalResult<&'a Arity> {
    for arity in arities {
        let has_rest = arity.params.iter().any(|p| p == "&");
        if has_rest {
            let rest_idx = arity.params.iter().position(|p| p == "&").unwrap();
            if nargs >= rest_idx {
                return Ok(arity);
            }
        } else if nargs == arity.params.len() {
            return Ok(arity);
        }
    }
    Err(EvalError::Custom(format!(
        "no matching arity for {} argument{}",
        nargs,
        if nargs == 1 { "" } else { "s" }
    )))
}

fn apply_inner(
    callee: &Value,
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    match callee {
        Value::Builtin(b) => (b.func)(args, env, &opts, world),
        Value::Closure(c) => {
            let mut current_args = args.to_vec();
            'tco: loop {
                let arity = find_arity(&c.arities, current_args.len())?;
                let closure_env = Env::extend(&c.env);
                bind_params(&arity.params, &current_args, &closure_env)?;

                // Evaluate body — the last expression is in tail position
                let body = &arity.body;
                let mut result = Value::Nil;
                let mut found_recur = false;
                for (i, expr) in body.iter().enumerate() {
                    let is_last = i == body.len() - 1;
                    if is_last {
                        let sub = opts.descend().ok_or(EvalError::RecursionLimit)?;
                        match eval_inner(expr, &closure_env, sub.with_recur(true), world) {
                            Ok(val) => result = val,
                            Err(EvalError::Recur(recur_args)) => {
                                current_args = recur_args;
                                found_recur = true;
                                break;
                            }
                            Err(e) => return Err(e),
                        }
                    } else {
                        result = eval_inner(
                            expr,
                            &closure_env,
                            opts.descend().ok_or(EvalError::RecursionLimit)?,
                            world,
                        )?;
                    }
                }
                if found_recur {
                    continue 'tco;
                }
                return Ok(result);
            }
        }
        other => Err(EvalError::NotCallable(other.to_string())),
    }
}

fn bind_params(params: &[String], args: &[Value], env: &Env) -> EvalResult<()> {
    let mut idx = 0;
    let mut piter = params.iter();
    while let Some(p) = piter.next() {
        if p == "&" {
            let rest = piter
                .next()
                .ok_or_else(|| EvalError::Custom("expected name after &".into()))?;
            env.bind(rest, Value::List(args[idx..].to_vec()));
            return Ok(());
        }
        if idx >= args.len() {
            return Err(EvalError::WrongArgCount {
                expected: params.len(),
                got: args.len(),
            });
        }
        env.bind(p, args[idx].clone());
        idx += 1;
    }
    if idx != args.len() {
        return Err(EvalError::WrongArgCount {
            expected: params.len(),
            got: args.len(),
        });
    }
    Ok(())
}

// --- Special Forms ---

fn eval_quote(args: &[Value], _env: &Env) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    Ok(args[0].clone())
}

fn eval_if_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let test = eval_inner(
        &args[0],
        env,
        opts.descend()
            .ok_or(EvalError::RecursionLimit)?
            .with_recur(false),
        world,
    )?;
    let truthy = !matches!(test, Value::Nil | Value::Bool(false));
    if truthy {
        eval_inner(
            &args[1],
            env,
            opts.descend().ok_or(EvalError::RecursionLimit)?,
            world,
        )
    } else if args.len() == 3 {
        eval_inner(
            &args[2],
            env,
            opts.descend().ok_or(EvalError::RecursionLimit)?,
            world,
        )
    } else {
        Ok(Value::Nil)
    }
}

fn eval_do_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    let mut result = Value::Nil;
    for (i, arg) in args.iter().enumerate() {
        let is_last = i == args.len() - 1;
        let sub_opts = opts
            .descend()
            .ok_or(EvalError::RecursionLimit)?
            .with_recur(is_last);
        result = eval_inner(arg, env, sub_opts, world)?;
    }
    Ok(result)
}

fn eval_let_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() < 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let name = match &args[0] {
        Value::Symbol(s) => s.name.clone(),
        other => {
            return Err(EvalError::TypeError {
                expected: "symbol",
                got: other.to_string(),
            })
        }
    };
    let value = eval_inner(
        &args[1],
        env,
        opts.descend()
            .ok_or(EvalError::RecursionLimit)?
            .with_recur(false),
        world,
    )?;
    let let_env = Env::extend(env);
    let_env.bind(&name, value);
    let body = &args[2..];
    let mut result = Value::Nil;
    for (i, expr) in body.iter().enumerate() {
        let is_last = i == body.len() - 1;
        let sub_opts = opts
            .descend()
            .ok_or(EvalError::RecursionLimit)?
            .with_recur(is_last);
        result = eval_inner(expr, &let_env, sub_opts, world)?;
    }
    Ok(result)
}

fn eval_fn(args: &[Value], env: &Env) -> EvalResult<Value> {
    if args.is_empty() {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: 0,
        });
    }
    // Multi-arity: (fn ([params] body...) ([params] body...)*)
    // Each clause is a list whose first element is a param spec (vec or symbol).
    if args.len() > 1 && args.iter().all(|a| matches!(a, Value::List(_))) {
        let mut arities = Vec::new();
        for clause in args {
            let items = match clause {
                Value::List(items) => items,
                _ => unreachable!(),
            };
            if items.is_empty() {
                return Err(EvalError::Custom("empty arity clause in fn".into()));
            }
            let params = parse_param_vec(&items[0])?;
            let body = items[1..].to_vec();
            arities.push(Arity { params, body });
        }
        if arities.is_empty() {
            return Err(EvalError::Custom("fn requires at least one arity".into()));
        }
        Ok(Value::Closure(ClosureData {
            arities,
            env: env.clone(),
        }))
    } else {
        // Single arity: (fn [params] body...) or (fn sym body...)
        let params = parse_param_vec(&args[0])?;
        let body: Vec<Value> = args[1..].to_vec();
        Ok(Value::Closure(ClosureData {
            arities: vec![Arity { params, body }],
            env: env.clone(),
        }))
    }
}

fn eval_const_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let name = match &args[0] {
        Value::Symbol(s) => s.name.clone(),
        other => {
            return Err(EvalError::TypeError {
                expected: "symbol",
                got: other.to_string(),
            })
        }
    };
    if env.exists(&name) {
        return Err(EvalError::Custom(format!("cannot redefine: {}", name)));
    }
    let value = eval_inner(
        &args[1],
        env,
        opts.descend().ok_or(EvalError::RecursionLimit)?,
        world,
    )?;
    env.bind(&name, value);
    Ok(Value::Symbol(Symbol::new(&name)))
}

fn eval_defmacro(args: &[Value], env: &Env) -> EvalResult<Value> {
    if args.len() < 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let name = match &args[0] {
        Value::Symbol(s) => s.name.clone(),
        other => {
            return Err(EvalError::TypeError {
                expected: "symbol",
                got: other.to_string(),
            })
        }
    };
    let params = parse_param_vec(&args[1])?;
    let body: Vec<Value> = args[2..].to_vec();
    env.bind(
        &name,
        Value::Macro(MacroData {
            params,
            body,
            env: env.clone(),
        }),
    );
    Ok(Value::Symbol(Symbol::new(&name)))
}

fn parse_param_vec(v: &Value) -> EvalResult<Vec<String>> {
    let items: &[Value] = match v {
        Value::Vector(items) => items.as_slice(),
        Value::Symbol(s) => return Ok(vec![s.name.clone()]),
        other => {
            return Err(EvalError::TypeError {
                expected: "vector or symbol for params",
                got: other.to_string(),
            })
        }
    };
    let mut params = Vec::new();
    let mut iter = items.iter();
    while let Some(p) = iter.next() {
        match p {
            Value::Symbol(s) if s.name == "&" => {
                let rest = iter
                    .next()
                    .ok_or_else(|| EvalError::Custom("expected name after &".into()))?;
                match rest {
                    Value::Symbol(s) => {
                        params.push("&".to_string());
                        params.push(s.name.clone());
                    }
                    other => {
                        return Err(EvalError::TypeError {
                            expected: "symbol after &",
                            got: other.to_string(),
                        })
                    }
                }
            }
            Value::Symbol(s) => params.push(s.name.clone()),
            other => {
                return Err(EvalError::TypeError {
                    expected: "symbol in param list",
                    got: other.to_string(),
                })
            }
        }
    }
    Ok(params)
}

fn eval_set_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let value = eval_inner(
        &args[1],
        env,
        opts.descend().ok_or(EvalError::RecursionLimit)?,
        world,
    )?;
    match &args[0] {
        Value::Symbol(s) => {
            env.set(&s.name, value.clone())?;
            Ok(value)
        }
        Value::List(items)
            if items.len() == 3 && matches!(&items[0], Value::Symbol(s) if s.name == ".") =>
        {
            let key = match &items[2] {
                Value::Keyword(k) => k.clone(),
                other => {
                    return Err(EvalError::TypeError {
                        expected: "keyword",
                        got: other.to_string(),
                    })
                }
            };
            if let Value::Symbol(sym) = &items[1] {
                let current = env
                    .lookup(&sym.name)
                    .ok_or_else(|| EvalError::UnboundSymbol(sym.name.clone()))?;
                match current {
                    Value::Map(mut map) => {
                        map.insert(Value::Keyword(key), value);
                        env.set(&sym.name, Value::Map(map))?;
                        return Ok(Value::Nil);
                    }
                    other => {
                        return Err(EvalError::Custom(format!(
                            "cannot set property on {}",
                            other
                        )))
                    }
                }
            }
            let obj = eval_inner(
                &items[1],
                env,
                opts.descend().ok_or(EvalError::RecursionLimit)?,
                world,
            )?;
            match obj {
                Value::Map(mut map) => {
                    map.insert(Value::Keyword(key), value);
                    Ok(Value::Map(map))
                }
                other => Err(EvalError::Custom(format!(
                    "cannot set property on {}",
                    other
                ))),
            }
        }
        other => Err(EvalError::Custom(format!("invalid set! place: {}", other))),
    }
}

fn eval_try_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.is_empty() {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: 0,
        });
    }
    let mut catch_start = args.len();
    for (i, arg) in args.iter().enumerate() {
        if let Value::List(items) = arg {
            if items.len() >= 2 && matches!(&items[0], Value::Symbol(s) if s.name == "catch") {
                catch_start = i;
                break;
            }
        }
    }
    let body = &args[..catch_start];
    let catches = &args[catch_start..];
    match eval_do_inner(
        body,
        env,
        opts.descend().ok_or(EvalError::RecursionLimit)?,
        world,
    ) {
        Ok(val) => Ok(val),
        Err(err) => {
            for clause in catches {
                if let Value::List(items) = clause {
                    if items.len() >= 2
                        && matches!(&items[0], Value::Symbol(s) if s.name == "catch")
                    {
                        let pattern = &items[1];
                        if pattern_matches(pattern, &Value::String(err.to_string())) {
                            let catch_env = Env::extend(env);
                            if let Value::Symbol(s) = pattern {
                                catch_env.bind(&s.name, Value::String(err.to_string()));
                            }
                            return eval_do_inner(
                                &items[2..],
                                &catch_env,
                                opts.descend().ok_or(EvalError::RecursionLimit)?,
                                world,
                            );
                        }
                    }
                }
            }
            Err(err)
        }
    }
}

fn pattern_matches(pattern: &Value, expr: &Value) -> bool {
    match pattern {
        Value::Symbol(s) if s.name == "_" => true,
        Value::Nil => matches!(expr, Value::Nil),
        Value::Bool(b) => matches!(expr, Value::Bool(e) if e == b),
        Value::I64(n) => matches!(expr, Value::I64(e) if e == n),
        Value::F64(n) => matches!(expr, Value::F64(e) if e == n),
        Value::String(s) => matches!(expr, Value::String(e) if e == s),
        Value::Keyword(k) => matches!(expr, Value::Keyword(e) if e == k),
        _ => false,
    }
}

fn eval_and_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        let is_last = i == args.len() - 1;
        let sub_opts = opts
            .descend()
            .ok_or(EvalError::RecursionLimit)?
            .with_recur(is_last);
        let val = eval_inner(arg, env, sub_opts, world)?;
        if matches!(val, Value::Nil | Value::Bool(false)) {
            return Ok(val);
        }
    }
    if args.is_empty() {
        return Ok(Value::Bool(true));
    }
    Ok(Value::Bool(true))
}

fn eval_or_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        let is_last = i == args.len() - 1;
        let sub_opts = opts
            .descend()
            .ok_or(EvalError::RecursionLimit)?
            .with_recur(is_last);
        let val = eval_inner(arg, env, sub_opts, world)?;
        if !matches!(val, Value::Nil | Value::Bool(false)) {
            return Ok(val);
        }
    }
    Ok(Value::Nil)
}

fn eval_match_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() < 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let expr = eval_inner(
        &args[0],
        env,
        opts.descend()
            .ok_or(EvalError::RecursionLimit)?
            .with_recur(false),
        world,
    )?;
    for clause in &args[1..] {
        let (pattern, body) = match clause {
            Value::Vector(v) if v.len() == 2 => (&v[0], &v[1]),
            Value::List(v) if v.len() == 2 => (&v[0], &v[1]),
            _ => {
                return Err(EvalError::Custom(format!(
                    "invalid match clause: {}",
                    clause
                )))
            }
        };
        if pattern_matches(pattern, &expr) {
            let match_env = Env::extend(env);
            if let Value::Symbol(s) = pattern {
                match_env.bind(&s.name, expr.clone());
            }
            return eval_inner(
                body,
                &match_env,
                opts.descend().ok_or(EvalError::RecursionLimit)?,
                world,
            );
        }
    }
    Err(EvalError::PatternMatchFailed(expr.to_string()))
}

fn eval_recur_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if !opts.recur_allowed {
        return Err(EvalError::Custom("recur outside function body".into()));
    }
    let mut recur_args = Vec::with_capacity(args.len());
    for arg in args {
        recur_args.push(eval_inner(
            arg,
            env,
            opts.descend().ok_or(EvalError::RecursionLimit)?,
            world,
        )?);
    }
    Err(EvalError::Recur(recur_args))
}

fn eval_bind_key_inner(
    args: &[Value],
    env: &Env,
    opts: SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let key_val = eval_inner(
        &args[0],
        env,
        opts.descend().ok_or(EvalError::RecursionLimit)?,
        world,
    )?;
    let key = match &key_val {
        Value::Keyword(kw) => kw.name.clone(),
        _ => {
            return Err(EvalError::TypeError {
                expected: "keyword (e.g. :z, :x)",
                got: key_val.to_string(),
            })
        }
    };
    if key.is_empty() || key.len() != 1 {
        return Err(EvalError::TypeError {
            expected: "single-character keyword (e.g. :z, :x)",
            got: format!(":{}", key),
        });
    }
    // Second arg is NOT evaluated — store its source text so the full
    // expression is re-evaluated when the key is pressed later.
    let source = args[1].to_string();
    world.bindings.insert(key.clone(), source.clone());
    world.event_log.push_colored(
        format!("Bound key '{}' to: {}", key, source),
        RGB::named(GREEN),
    );
    // Return the source as a value so it shows in console output
    Ok(args[1].clone())
}

// --- Built-in functions ---

fn int_op<F>(a: &Value, b: &Value, f: F) -> Option<Value>
where
    F: Fn(i64, i64) -> i64,
{
    match (a, b) {
        (Value::I64(a), Value::I64(b)) => Some(Value::I64(f(*a, *b))),
        _ => None,
    }
}

fn float_op<F>(a: &Value, b: &Value, f: F) -> EvalResult<Value>
where
    F: Fn(f64, f64) -> f64,
{
    Ok(Value::F64(f(as_float(a)?, as_float(b)?)))
}

fn as_float(v: &Value) -> EvalResult<f64> {
    match v {
        Value::I64(n) => Ok(*n as f64),
        Value::F64(n) => Ok(*n),
        other => Err(EvalError::TypeError {
            expected: "number",
            got: other.to_string(),
        }),
    }
}

#[cfg(not(feature = "prelude"))]
fn as_int(v: &Value) -> EvalResult<i64> {
    match v {
        Value::I64(n) => Ok(*n),
        other => Err(EvalError::TypeError {
            expected: "int",
            got: other.to_string(),
        }),
    }
}

macro_rules! arith_binop {
    ($name:ident, $op:tt) => {
        fn $name(
            args: &[Value],
            _env: &Env,
            _opts: &SandboxOptions,
            _world: &mut World,
        ) -> EvalResult<Value> {
            if args.is_empty() { return Ok(Value::I64(0)); }
            let mut acc = args[0].clone();
            for arg in &args[1..] {
                if let Some(result) = int_op(&acc, arg, |a, b| a $op b) { acc = result; }
                else { acc = float_op(&acc, arg, |a, b| a $op b)?; }
            }
            Ok(acc)
        }
    };
}

arith_binop!(builtin_add, +);
arith_binop!(builtin_sub, -);
arith_binop!(builtin_mul, *);

fn builtin_div(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.is_empty() {
        return Ok(Value::I64(0));
    }
    for arg in &args[1..] {
        match arg {
            Value::I64(0) => return Err(EvalError::DivisionByZero),
            Value::F64(n) if *n == 0.0 => return Err(EvalError::DivisionByZero),
            _ => {}
        }
    }
    let mut acc = args[0].clone();
    for arg in &args[1..] {
        match (&acc, arg) {
            (Value::I64(a), Value::I64(b)) => acc = Value::I64(a / b),
            _ => acc = float_op(&acc, arg, |a, b| a / b)?,
        }
    }
    Ok(acc)
}

fn builtin_eq(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    Ok(Value::Bool(args.windows(2).all(|w| w[0] == w[1])))
}

fn builtin_neq(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    Ok(Value::Bool(args.len() == 2 && args[0] != args[1]))
}

macro_rules! cmp_binop {
    ($name:ident, $op:tt) => {
        fn $name(
            args: &[Value],
            _env: &Env,
            _opts: &SandboxOptions,
            _world: &mut World,
        ) -> EvalResult<Value> {
            let vals: Vec<f64> = args
                .iter()
                .map(|a| as_float(a))
                .collect::<EvalResult<_>>()?;
            Ok(Value::Bool(vals.windows(2).all(|w| w[0] $op w[1])))
        }
    };
}

cmp_binop!(builtin_lt, <);
cmp_binop!(builtin_gt, >);
cmp_binop!(builtin_lte, <=);
cmp_binop!(builtin_gte, >=);

fn builtin_dot(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let key = match &args[1] {
        Value::Keyword(k) => k,
        other => {
            return Err(EvalError::TypeError {
                expected: "keyword",
                got: other.to_string(),
            })
        }
    };
    match &args[0] {
        Value::Map(m) => m
            .get(&Value::Keyword(key.clone()))
            .cloned()
            .ok_or_else(|| EvalError::NotInList(key.name.clone(), "key not in map".into())),
        other => Err(EvalError::TypeError {
            expected: "map",
            got: other.to_string(),
        }),
    }
}

fn builtin_list(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    Ok(Value::List(args.to_vec()))
}

fn builtin_vector(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    Ok(Value::Vector(args.to_vec()))
}

fn builtin_print(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            world.console_output.push(' ');
        }
        world.console_output.push_str(&print_value(arg));
    }
    Ok(Value::Nil)
}

fn builtin_println(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            world.console_output.push(' ');
        }
        world.console_output.push_str(&print_value(arg));
    }
    world.console_output.push('\n');
    Ok(Value::Nil)
}

fn print_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn builtin_type_of(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    Ok(super::kw(match &args[0] {
        Value::Nil => "nil",
        Value::Bool(_) => "bool",
        Value::I64(_) => "int",
        Value::F64(_) => "float",
        Value::String(_) => "string",
        Value::Symbol(_) => "symbol",
        Value::Keyword(_) => "keyword",
        Value::List(_) => "list",
        Value::Vector(_) => "vector",
        Value::Map(_) => "map",
        Value::Set(_) => "set",
        Value::Builtin(_) => "builtin",
        Value::Closure(_) => "fn",
        Value::Macro(_) => "macro",
    }))
}

fn builtin_cons(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    match &args[1] {
        Value::List(items) => {
            let mut new = vec![args[0].clone()];
            new.extend(items.iter().cloned());
            Ok(Value::List(new))
        }
        other => Err(EvalError::TypeError {
            expected: "list",
            got: other.to_string(),
        }),
    }
}

fn builtin_first(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    match &args[0] {
        Value::List(items) => items
            .first()
            .cloned()
            .ok_or_else(|| EvalError::Custom("empty list".into())),
        other => Err(EvalError::TypeError {
            expected: "list",
            got: other.to_string(),
        }),
    }
}

fn builtin_rest(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    match &args[0] {
        Value::List(items) => Ok(Value::List(items.iter().skip(1).cloned().collect())),
        other => Err(EvalError::TypeError {
            expected: "list",
            got: other.to_string(),
        }),
    }
}

fn builtin_emptyq(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    match &args[0] {
        Value::List(items) => Ok(Value::Bool(items.is_empty())),
        Value::Vector(items) => Ok(Value::Bool(items.is_empty())),
        Value::String(s) => Ok(Value::Bool(s.is_empty())),
        other => Err(EvalError::TypeError {
            expected: "list, vector, or string",
            got: other.to_string(),
        }),
    }
}

fn builtin_str(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    let mut out = String::new();
    for arg in args {
        out.push_str(&arg.to_string());
    }
    Ok(Value::String(out))
}

fn builtin_slurp(
    args: &[Value],
    _env: &Env,
    opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    let path = match &args[0] {
        Value::String(s) => s.clone(),
        other => {
            return Err(EvalError::TypeError {
                expected: "string (path)",
                got: other.to_string(),
            })
        }
    };
    if let Some(vfs) = &opts.vfs {
        match vfs.read_to_string(&path) {
            Ok(contents) => return Ok(Value::String(contents)),
            Err(e) => return Err(EvalError::Custom(format!("cannot read '{}': {}", path, e))),
        }
    }
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(Value::String(contents)),
        Err(e) => Err(EvalError::Custom(format!("cannot read '{}': {}", path, e))),
    }
}

fn builtin_eval(
    args: &[Value],
    env: &Env,
    opts: &SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    eval_with_opts(&args[0], env, opts.clone(), world)
}

fn builtin_apply(
    args: &[Value],
    env: &Env,
    opts: &SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() < 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let callee = &args[0];
    let mut call_args: Vec<Value> = args[1..args.len() - 1].to_vec();
    match &args[args.len() - 1] {
        Value::List(items) => call_args.extend(items.iter().cloned()),
        other => {
            return Err(EvalError::TypeError {
                expected: "list (last arg)",
                got: other.to_string(),
            })
        }
    }
    apply_inner(
        callee,
        &call_args,
        env,
        opts.descend().ok_or(EvalError::RecursionLimit)?,
        world,
    )
}

fn builtin_map_fn(
    args: &[Value],
    env: &Env,
    opts: &SandboxOptions,
    world: &mut World,
) -> EvalResult<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let list = match &args[1] {
        Value::List(items) => items.clone(),
        other => {
            return Err(EvalError::TypeError {
                expected: "list",
                got: other.to_string(),
            })
        }
    };
    let mut result = Vec::with_capacity(list.len());
    let inner_opts = opts.descend().ok_or(EvalError::RecursionLimit)?;
    for item in &list {
        result.push(apply_inner(
            &args[0],
            &[item.clone()],
            env,
            inner_opts.clone(),
            world,
        )?);
    }
    Ok(Value::List(result))
}

#[cfg(not(feature = "prelude"))]
fn builtin_range(
    args: &[Value],
    _env: &Env,
    _opts: &SandboxOptions,
    _world: &mut World,
) -> EvalResult<Value> {
    let (start, end, step) =
        match args.len() {
            1 => {
                let e = as_int(&args[0])?;
                (0i64, e, 1i64)
            }
            2 => {
                let s = as_int(&args[0])?;
                let e = as_int(&args[1])?;
                (s, e, 1i64)
            }
            3 => {
                let s = as_int(&args[0])?;
                let e = as_int(&args[1])?;
                let st = as_int(&args[2])?;
                if st == 0 {
                    return Err(EvalError::Custom("range step cannot be zero".into()));
                }
                (s, e, st)
            }
            _ => return Err(EvalError::Custom(
                "range expects 1-3 args: (range end), (range start end), or (range start end step)"
                    .into(),
            )),
        };
    let mut items = Vec::new();
    if step > 0 {
        let mut i = start;
        while i < end {
            items.push(Value::I64(i));
            i += step;
        }
    } else {
        let mut i = start;
        while i > end {
            items.push(Value::I64(i));
            i += step;
        }
    }
    Ok(Value::List(items))
}

// --- Environment Setup ---

fn builtin_fn(
    name: &'static str,
    func: fn(&[Value], &Env, &SandboxOptions, &mut World) -> EvalResult<Value>,
) -> Value {
    Value::Builtin(BuiltinFn { name, func })
}

/// Create the default global environment with all built-ins.
pub fn default_env() -> Env {
    let env = Env::new();

    env.bind("+", builtin_fn("+", builtin_add));
    env.bind("-", builtin_fn("-", builtin_sub));
    env.bind("*", builtin_fn("*", builtin_mul));
    env.bind("/", builtin_fn("/", builtin_div));
    env.bind(
        "%",
        builtin_fn("%", |args, _env, _opts, _world| {
            if args.len() != 2 {
                return Err(EvalError::WrongArgCount {
                    expected: 2,
                    got: args.len(),
                });
            }
            let a = match &args[0] {
                Value::I64(n) => *n,
                other => {
                    return Err(EvalError::TypeError {
                        expected: "integer",
                        got: other.to_string(),
                    })
                }
            };
            let b = match &args[1] {
                Value::I64(n) => *n,
                other => {
                    return Err(EvalError::TypeError {
                        expected: "integer",
                        got: other.to_string(),
                    })
                }
            };
            if b == 0 {
                return Err(EvalError::DivisionByZero);
            }
            Ok(Value::I64(a % b))
        }),
    );

    env.bind("=", builtin_fn("=", builtin_eq));
    env.bind("!=", builtin_fn("!=", builtin_neq));
    env.bind("<", builtin_fn("<", builtin_lt));
    env.bind(">", builtin_fn(">", builtin_gt));
    env.bind("<=", builtin_fn("<=", builtin_lte));
    env.bind(">=", builtin_fn(">=", builtin_gte));

    env.bind(".", builtin_fn(".", builtin_dot));

    env.bind("list", builtin_fn("list", builtin_list));
    env.bind("vector", builtin_fn("vector", builtin_vector));
    env.bind("cons", builtin_fn("cons", builtin_cons));
    env.bind("first", builtin_fn("first", builtin_first));
    env.bind("rest", builtin_fn("rest", builtin_rest));
    env.bind("empty?", builtin_fn("empty?", builtin_emptyq));
    env.bind("map", builtin_fn("map", builtin_map_fn));
    #[cfg(not(feature = "prelude"))]
    env.bind("range", builtin_fn("range", builtin_range));

    env.bind("print", builtin_fn("print", builtin_print));
    env.bind("println", builtin_fn("println", builtin_println));
    env.bind("print!", builtin_fn("print!", builtin_print));
    env.bind("println!", builtin_fn("println!", builtin_println));
    env.bind("slurp", builtin_fn("slurp", builtin_slurp));

    env.bind("type", builtin_fn("type", builtin_type_of));
    env.bind("str", builtin_fn("str", builtin_str));

    env.bind("eval", builtin_fn("eval", builtin_eval));
    env.bind("apply", builtin_fn("apply", builtin_apply));

    env
}

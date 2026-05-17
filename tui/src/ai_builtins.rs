//! AI builtins for Glyph rule evaluation.
//!
//! Builtins that enemy AI rules call — `adjacent?`, `attack!`,
//! `step-toward!`, etc. They access the active [`World`] through a
//! thread-local pointer set by [`advance_enemies`](crate::game::World::advance_enemies).

use std::cell::RefCell;

use crate::{
    entity::{EntityId, Position},
    game::World,
    glyph::{self, Env, Value},
};

thread_local! {
    pub(crate) static ACTIVE_WORLD: RefCell<*mut World> =
        const { RefCell::new(std::ptr::null_mut()) };
}

/// Access the active World from within an AI builtin.
///
/// Returns `None` when called outside of `advance_enemies`.
///
/// # Safety
/// The caller must ensure no other mutable borrow of `World` exists
/// for the duration of `f`.
pub(crate) unsafe fn with_active_world<R>(f: impl FnOnce(&mut World) -> R) -> Option<R> {
    ACTIVE_WORLD.with(|cell| {
        let ptr = *cell.borrow();
        if ptr.is_null() {
            None
        } else {
            Some(f(&mut *ptr))
        }
    })
}

fn entity_id_from_value(v: &Value) -> glyph::EvalResult<EntityId> {
    match v {
        Value::I64(n) => Ok(EntityId::new(*n as usize)),
        other => Err(glyph::EvalError::TypeError {
            expected: "entity id (int)",
            got: other.to_string(),
        }),
    }
}

fn i64_from_value(v: &Value) -> glyph::EvalResult<i64> {
    match v {
        Value::I64(n) => Ok(*n),
        other => Err(glyph::EvalError::TypeError {
            expected: "int",
            got: other.to_string(),
        }),
    }
}

fn f64_from_value(v: &Value) -> glyph::EvalResult<f64> {
    match v {
        Value::F64(n) => Ok(*n),
        Value::I64(n) => Ok(*n as f64),
        other => Err(glyph::EvalError::TypeError {
            expected: "number",
            got: other.to_string(),
        }),
    }
}

// ---------------------------------------------------------------------------
// AI builtins
// ---------------------------------------------------------------------------

fn builtin_adjacentq(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let a = entity_id_from_value(&args[0])?;
    let b = entity_id_from_value(&args[1])?;
    unsafe {
        with_active_world(|w| {
            let pa = w.ecs.position(a);
            let pb = w.ecs.position(b);
            Value::Bool(match (pa, pb) {
                (Some(pa), Some(pb)) => pa.manhattan_distance(pb) == 1,
                _ => false,
            })
        })
        .ok_or_else(|| glyph::EvalError::Custom("AI builtin called without active world".into()))
    }
}

fn builtin_ai_attack(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    if args.len() != 3 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 3,
            got: args.len(),
        });
    }
    let attacker = entity_id_from_value(&args[0])?;
    let target = entity_id_from_value(&args[1])?;
    let dmg = i64_from_value(&args[2])? as i32;

    unsafe {
        with_active_world(|w| {
            if !w.ecs.is_alive(target) || !w.ecs.is_alive(attacker) {
                return Value::Nil;
            }
            if target == w.player_id && w.blocking {
                w.event_log
                    .push(format!("You block the {}'s attack.", w.ecs.name(attacker)));
            } else {
                w.ecs.damage(target, dmg);
                let attacker_name = w.ecs.name(attacker);
                if target == w.player_id {
                    w.event_log.push(format!(
                        "The {} attacks you for {} damage.",
                        attacker_name, dmg
                    ));
                } else {
                    w.event_log.push(format!(
                        "The {} attacks the {} for {} damage.",
                        attacker_name,
                        w.ecs.name(target),
                        dmg
                    ));
                }
            }
            Value::Nil
        })
        .ok_or_else(|| glyph::EvalError::Custom("AI builtin called without active world".into()))
    }
}

fn builtin_step_toward(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;
    let target = entity_id_from_value(&args[1])?;

    unsafe {
        with_active_world(|w| {
            let target_pos = match w.ecs.position(target) {
                Some(p) => p,
                None => return Value::Bool(false),
            };
            let path = w.enemy_ai_path(entity);
            if !path.success || path.steps.len() < 2 {
                return Value::Bool(false);
            }
            let next_pos = w.map.position_for_idx(path.steps[1]);
            if next_pos == target_pos
                || !w.map.is_walkable(next_pos)
                || w.ecs.entity_at_except(next_pos, entity).is_some()
            {
                return Value::Bool(false);
            }
            w.ecs.set_position(entity, next_pos);
            Value::Bool(true)
        })
        .ok_or_else(|| glyph::EvalError::Custom("AI builtin called without active world".into()))
    }
}

fn builtin_random_step(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    if args.len() != 1 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;

    unsafe {
        with_active_world(|w| {
            let pos = match w.ecs.position(entity) {
                Some(p) => p,
                None => return Value::Bool(false),
            };
            let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];
            let idx = (pos
                .x
                .wrapping_mul(7)
                .wrapping_add(pos.y.wrapping_mul(3))
                .wrapping_add(w.turn as i32)) as usize;
            let player_pos = w.player_pos();
            for i in 0..4 {
                let (dx, dy) = dirs[(idx + i) % 4];
                let candidate = Position::new(pos.x + dx, pos.y + dy);
                if w.map.is_walkable(candidate)
                    && candidate != player_pos
                    && w.ecs.entity_at(candidate).is_none()
                {
                    w.ecs.set_position(entity, candidate);
                    return Value::Bool(true);
                }
            }
            Value::Bool(false)
        })
        .ok_or_else(|| glyph::EvalError::Custom("AI builtin called without active world".into()))
    }
}

fn builtin_flee_step(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;
    let threat = entity_id_from_value(&args[1])?;

    unsafe {
        with_active_world(|w| {
            let pos = match w.ecs.position(entity) {
                Some(p) => p,
                None => return Value::Bool(false),
            };
            let threat_pos = match w.ecs.position(threat) {
                Some(p) => p,
                None => return Value::Bool(false),
            };
            let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];
            let mut best: Option<Position> = None;
            let mut best_dist = pos.manhattan_distance(threat_pos);
            let player_pos = w.player_pos();
            for (dx, dy) in &dirs {
                let candidate = Position::new(pos.x + dx, pos.y + dy);
                if w.map.is_walkable(candidate)
                    && candidate != player_pos
                    && w.ecs.entity_at(candidate).is_none()
                {
                    let dist = candidate.manhattan_distance(threat_pos);
                    if dist > best_dist {
                        best_dist = dist;
                        best = Some(candidate);
                    }
                }
            }
            if let Some(next) = best {
                w.ecs.set_position(entity, next);
                Value::Bool(true)
            } else {
                Value::Bool(false)
            }
        })
        .ok_or_else(|| glyph::EvalError::Custom("AI builtin called without active world".into()))
    }
}

fn builtin_roll_oddsq(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;
    let prob = f64_from_value(&args[1])?;

    unsafe {
        with_active_world(|w| {
            let pos = match w.ecs.position(entity) {
                Some(p) => p,
                None => return Value::Bool(false),
            };
            let hash = (pos.x as u64)
                .wrapping_mul(13)
                .wrapping_add((pos.y as u64).wrapping_mul(7))
                .wrapping_add(w.turn);
            let threshold = (prob * 100.0) as u64;
            Value::Bool(hash % 100 < threshold)
        })
        .ok_or_else(|| glyph::EvalError::Custom("AI builtin called without active world".into()))
    }
}

fn builtin_ai_hp(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
) -> glyph::EvalResult<Value> {
    if args.len() != 1 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;

    unsafe {
        with_active_world(|w| {
            let hp = w.ecs.hp(entity).map(|h| h.current).unwrap_or(0);
            Value::I64(hp as i64)
        })
        .ok_or_else(|| glyph::EvalError::Custom("AI builtin called without active world".into()))
    }
}

/// Register all AI builtins into the given Glyph environment.
pub(crate) fn register_all(env: &Env) {
    env.bind(
        "adjacent?",
        Value::Builtin(glyph::BuiltinFn {
            name: "adjacent?",
            func: builtin_adjacentq,
        }),
    );
    env.bind(
        "attack!",
        Value::Builtin(glyph::BuiltinFn {
            name: "attack!",
            func: builtin_ai_attack,
        }),
    );
    env.bind(
        "step-toward!",
        Value::Builtin(glyph::BuiltinFn {
            name: "step-toward!",
            func: builtin_step_toward,
        }),
    );
    env.bind(
        "random-step!",
        Value::Builtin(glyph::BuiltinFn {
            name: "random-step!",
            func: builtin_random_step,
        }),
    );
    env.bind(
        "flee-step!",
        Value::Builtin(glyph::BuiltinFn {
            name: "flee-step!",
            func: builtin_flee_step,
        }),
    );
    env.bind(
        "roll-odds?",
        Value::Builtin(glyph::BuiltinFn {
            name: "roll-odds?",
            func: builtin_roll_oddsq,
        }),
    );
    env.bind(
        "hp",
        Value::Builtin(glyph::BuiltinFn {
            name: "hp",
            func: builtin_ai_hp,
        }),
    );
}

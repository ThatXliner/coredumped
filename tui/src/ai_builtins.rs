//! AI builtins for Glyph rule evaluation.
//!
//! Builtins that enemy AI rules call — `adjacent?`, `attack!`,
//! `step-toward!`, etc. They receive `&mut World` directly through the
//! eval chain's context parameter.

use bracket_lib::prelude::{ORANGE, RED, RGB};

use crate::{
    entity::{EntityId, Position},
    glyph::{self, Env, Value},
    world::World,
};

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
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let a = entity_id_from_value(&args[0])?;
    let b = entity_id_from_value(&args[1])?;
    let pa = world.ecs.position(a);
    let pb = world.ecs.position(b);
    Ok(Value::Bool(match (pa, pb) {
        (Some(pa), Some(pb)) => pa.manhattan_distance(pb) == 1,
        _ => false,
    }))
}

fn builtin_ai_attack(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
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

    if !world.ecs.is_alive(target) || !world.ecs.is_alive(attacker) {
        return Ok(Value::Nil);
    }
    if target == world.player_id && world.blocking {
        world.event_log.push_colored(
            format!("You block the {}'s attack.", world.ecs.name(attacker)),
            RGB::named(ORANGE),
        );
    } else {
        world.ecs.damage(target, dmg);
        let attacker_name = world.ecs.name(attacker);
        if target == world.player_id {
            world.event_log.push_colored(
                format!("The {} attacks you for {} damage.", attacker_name, dmg),
                RGB::named(RED),
            );
        } else {
            world.event_log.push_colored(
                format!(
                    "The {} attacks the {} for {} damage.",
                    attacker_name,
                    world.ecs.name(target),
                    dmg
                ),
                RGB::named(RED),
            );
        }
    }
    Ok(Value::Nil)
}

fn builtin_step_toward(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;
    let target = entity_id_from_value(&args[1])?;

    let target_pos = match world.ecs.position(target) {
        Some(p) => p,
        None => return Ok(Value::Bool(false)),
    };
    let path = world.enemy_ai_path(entity);
    if !path.success || path.steps.len() < 2 {
        return Ok(Value::Bool(false));
    }
    let next_pos = world.map.position_for_idx(path.steps[1]);
    if next_pos == target_pos
        || !world.map.is_walkable(next_pos)
        || world.ecs.entity_at_except(next_pos, entity).is_some()
    {
        return Ok(Value::Bool(false));
    }
    world.ecs.set_position(entity, next_pos);
    Ok(Value::Bool(true))
}

fn builtin_random_step(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 1 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;

    let pos = match world.ecs.position(entity) {
        Some(p) => p,
        None => return Ok(Value::Bool(false)),
    };
    let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let idx = (pos
        .x
        .wrapping_mul(7)
        .wrapping_add(pos.y.wrapping_mul(3))
        .wrapping_add(world.turn as i32)) as usize;
    let player_pos = world.player_pos();
    for i in 0..4 {
        let (dx, dy) = dirs[(idx + i) % 4];
        let candidate = Position::new(pos.x + dx, pos.y + dy);
        if world.map.is_walkable(candidate)
            && candidate != player_pos
            && world.ecs.entity_at(candidate).is_none()
        {
            world.ecs.set_position(entity, candidate);
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}

fn builtin_flee_step(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;
    let threat = entity_id_from_value(&args[1])?;

    let pos = match world.ecs.position(entity) {
        Some(p) => p,
        None => return Ok(Value::Bool(false)),
    };
    let threat_pos = match world.ecs.position(threat) {
        Some(p) => p,
        None => return Ok(Value::Bool(false)),
    };
    let dirs = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let mut best: Option<Position> = None;
    let mut best_dist = pos.manhattan_distance(threat_pos);
    let player_pos = world.player_pos();
    for (dx, dy) in &dirs {
        let candidate = Position::new(pos.x + dx, pos.y + dy);
        if world.map.is_walkable(candidate)
            && candidate != player_pos
            && world.ecs.entity_at(candidate).is_none()
        {
            let dist = candidate.manhattan_distance(threat_pos);
            if dist > best_dist {
                best_dist = dist;
                best = Some(candidate);
            }
        }
    }
    if let Some(next) = best {
        world.ecs.set_position(entity, next);
        Ok(Value::Bool(true))
    } else {
        Ok(Value::Bool(false))
    }
}

fn builtin_roll_oddsq(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 2 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;
    let prob = f64_from_value(&args[1])?;

    let pos = match world.ecs.position(entity) {
        Some(p) => p,
        None => return Ok(Value::Bool(false)),
    };
    let hash = (pos.x as u64)
        .wrapping_mul(13)
        .wrapping_add((pos.y as u64).wrapping_mul(7))
        .wrapping_add(world.turn);
    let threshold = (prob * 100.0) as u64;
    Ok(Value::Bool(hash % 100 < threshold))
}

fn builtin_ai_hp(
    args: &[Value],
    _env: &Env,
    _opts: &glyph::SandboxOptions,
    world: &mut World,
) -> glyph::EvalResult<Value> {
    if args.len() != 1 {
        return Err(glyph::EvalError::WrongArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    let entity = entity_id_from_value(&args[0])?;
    let hp = world.ecs.hp(entity).map(|h| h.current).unwrap_or(0);
    Ok(Value::I64(hp as i64))
}

/// Register all AI builtins into the given Glyph environment.
pub(crate) fn register_all(env: &Env) {
    env.bind(
        "adjacent?",
        Value::Builtin(glyph::BuiltinFn {
            name: "adjacent?",
            doc: "check if two entities are adjacent",
            func: builtin_adjacentq,
        }),
    );
    env.bind(
        "attack!",
        Value::Builtin(glyph::BuiltinFn {
            name: "attack!",
            doc: "attack a target entity: (attack! target-id)",
            func: builtin_ai_attack,
        }),
    );
    env.bind(
        "step-toward!",
        Value::Builtin(glyph::BuiltinFn {
            name: "step-toward!",
            doc: "move one step toward a target entity",
            func: builtin_step_toward,
        }),
    );
    env.bind(
        "random-step!",
        Value::Builtin(glyph::BuiltinFn {
            name: "random-step!",
            doc: "take a random step to an adjacent walkable tile",
            func: builtin_random_step,
        }),
    );
    env.bind(
        "flee-step!",
        Value::Builtin(glyph::BuiltinFn {
            name: "flee-step!",
            doc: "move one step away from a target entity",
            func: builtin_flee_step,
        }),
    );
    env.bind(
        "roll-odds?",
        Value::Builtin(glyph::BuiltinFn {
            name: "roll-odds?",
            doc: "roll a chance: (roll-odds? numerator denominator)",
            func: builtin_roll_oddsq,
        }),
    );
    env.bind(
        "hp",
        Value::Builtin(glyph::BuiltinFn {
            name: "hp",
            doc: "get the HP of an entity: (hp entity-id)",
            func: builtin_ai_hp,
        }),
    );
}

//! AI builtins for Glyph rule evaluation.
//!
//! Builtins that enemy AI rules call — `adjacent?`, `attack!`,
//! `step-toward!`, etc. They receive `&mut World` directly through the
//! eval chain's context parameter.

use bracket_color::prelude::{ORANGE, RED, RGB};

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
    let entity_pos = match world.ecs.position(entity) {
        Some(p) => p,
        None => return Ok(Value::Bool(false)),
    };

    if entity_pos == target_pos {
        return Ok(Value::Bool(false));
    }

    let next_pos = match world.dijkstra_best_step(entity_pos, target_pos) {
        Some(pos) => pos,
        None => return Ok(Value::Bool(false)),
    };

    if next_pos == target_pos || !world.map.is_walkable(next_pos) {
        return Ok(Value::Bool(false));
    }

    if let Some(blocker) = world.ecs.entity_at_except(next_pos, entity) {
        log::warn!(
            target: "xlyph::ai",
            "move blocked turn={} depth={} actor={}#{} from=({},{}) to=({},{}) occupied_by={}#{}",
            world.turn,
            world.depth,
            world.ecs.name(entity),
            entity.raw(),
            entity_pos.x,
            entity_pos.y,
            next_pos.x,
            next_pos.y,
            world.ecs.name(blocker),
            blocker.raw()
        );
        return Ok(Value::Bool(false));
    }

    let moved = world.ecs.set_position(entity, next_pos);
    if moved {
        log::debug!(
            target: "xlyph::ai",
            "step-toward turn={} depth={} actor={}#{} from=({},{}) to=({},{}) target=({},{})",
            world.turn,
            world.depth,
            world.ecs.name(entity),
            entity.raw(),
            entity_pos.x,
            entity_pos.y,
            next_pos.x,
            next_pos.y,
            target_pos.x,
            target_pos.y
        );
    }
    Ok(Value::Bool(moved))
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
            let moved = world.ecs.set_position(entity, candidate);
            if moved {
                log::debug!(
                    target: "xlyph::ai",
                    "random-step turn={} depth={} actor={}#{} from=({},{}) to=({},{})",
                    world.turn,
                    world.depth,
                    world.ecs.name(entity),
                    entity.raw(),
                    pos.x,
                    pos.y,
                    candidate.x,
                    candidate.y
                );
            }
            return Ok(Value::Bool(moved));
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
        let moved = world.ecs.set_position(entity, next);
        if moved {
            log::debug!(
                target: "xlyph::ai",
                "flee-step turn={} depth={} actor={}#{} from=({},{}) to=({},{}) threat=({},{})",
                world.turn,
                world.depth,
                world.ecs.name(entity),
                entity.raw(),
                pos.x,
                pos.y,
                next.x,
                next.y,
                threat_pos.x,
                threat_pos.y
            );
        }
        Ok(Value::Bool(moved))
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

fn builtin_manhattan(
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
    match (pa, pb) {
        (Some(pa), Some(pb)) => Ok(Value::I64(pa.manhattan_distance(pb) as i64)),
        _ => Ok(Value::I64(999)),
    }
}

/// Register all AI builtins into the given Glyph environment.
pub(crate) fn register_all(env: &Env) {
    env.bind(
        "adjacent?",
        Value::Builtin(glyph::BuiltinFn {
            name: "adjacent?",
            doc: "check if two entities are adjacent\n\nSYNOPSIS\n  (adjacent? entity-a entity-b)\n\nARGUMENTS\n  entity-a  Integer entity ID.\n  entity-b  Integer entity ID.\n\nDESCRIPTION\n  Returns true if the two entities are exactly 1 tile apart\n  (Manhattan distance == 1). Returns false if either entity\n  has no position or they are further apart.\n\nRETURN VALUE\n  true or false.\n\nEXAMPLES\n  (if (adjacent? self player-id) (attack! self player-id 1))",
            func: builtin_adjacentq,
        }),
    );
    env.bind(
        "attack!",
        Value::Builtin(glyph::BuiltinFn {
            name: "attack!",
            doc: "attack a target entity (AI)\n\nSYNOPSIS\n  (attack! attacker-id target-id damage)\n\nARGUMENTS\n  attacker-id  Integer entity ID of the attacker.\n  target-id    Integer entity ID of the target.\n  damage       Integer damage to deal.\n\nDESCRIPTION\n  An AI combat action. The attacker deals the specified damage\n  to the target. If the target is the player and blocking is\n  active, the attack is negated with an \"You block\" message.\n  If either entity is dead, the attack is silently skipped.\n  Logs the attack to the event log in red.\n\nRETURN VALUE\n  nil\n\nERRORS\n  Wrong arg count (expects exactly 3).\n\nSEE ALSO\n  adjacent?, step-toward!, do-attack",
            func: builtin_ai_attack,
        }),
    );
    env.bind(
        "step-toward!",
        Value::Builtin(glyph::BuiltinFn {
            name: "step-toward!",
            doc: "move one step toward a target entity\n\nSYNOPSIS\n  (step-toward! entity-id target-id)\n\nARGUMENTS\n  entity-id  Integer entity ID of the mover.\n  target-id  Integer entity ID of the destination entity.\n\nDESCRIPTION\n  Moves the entity one tile closer to the target using Dijkstra\n  pathfinding. Will not step onto the target's tile, through\n  walls, or onto occupied tiles. Returns false if no valid step\n  exists.\n\nRETURN VALUE\n  true if the entity moved, false otherwise.\n\nERRORS\n  Wrong arg count (expects exactly 2).\n\nSEE ALSO\n  flee-step!, random-step!, adjacent?",
            func: builtin_step_toward,
        }),
    );
    env.bind(
        "random-step!",
        Value::Builtin(glyph::BuiltinFn {
            name: "random-step!",
            doc: "take a random step to an adjacent walkable tile\n\nSYNOPSIS\n  (random-step! entity-id)\n\nARGUMENTS\n  entity-id  Integer entity ID of the mover.\n\nDESCRIPTION\n  Moves the entity to a random adjacent walkable tile that is\n  not occupied by another entity or the player. The \"random\"\n  direction is deterministic — derived from position and turn\n  number — so replays are reproducible. Tries all 4 cardinal\n  directions before giving up.\n\nRETURN VALUE\n  true if the entity moved, false if all adjacent tiles are\n  blocked.\n\nERRORS\n  Wrong arg count (expects exactly 1).\n\nSEE ALSO\n  step-toward!, flee-step!",
            func: builtin_random_step,
        }),
    );
    env.bind(
        "flee-step!",
        Value::Builtin(glyph::BuiltinFn {
            name: "flee-step!",
            doc: "move one step away from a target entity\n\nSYNOPSIS\n  (flee-step! entity-id threat-id)\n\nARGUMENTS\n  entity-id  Integer entity ID of the fleeing entity.\n  threat-id  Integer entity ID of the entity to flee from.\n\nDESCRIPTION\n  Moves the entity one tile in the cardinal direction that\n  maximizes Manhattan distance from the threat. Only considers\n  walkable, unoccupied tiles that are not the player's position.\n  Returns false if no retreat path exists.\n\nRETURN VALUE\n  true if the entity moved, false otherwise.\n\nERRORS\n  Wrong arg count (expects exactly 2).\n\nSEE ALSO\n  step-toward!, random-step!",
            func: builtin_flee_step,
        }),
    );
    env.bind(
        "roll-odds?",
        Value::Builtin(glyph::BuiltinFn {
            name: "roll-odds?",
            doc: "roll a deterministic chance check\n\nSYNOPSIS\n  (roll-odds? entity-id probability)\n\nARGUMENTS\n  entity-id    Integer entity ID (used for positional hash seed).\n  probability  A float between 0.0 and 1.0 (e.g., 0.5 for 50%).\n\nDESCRIPTION\n  Returns true with approximately the given probability. The\n  result is deterministic — derived from the entity's position\n  and the current turn number — so identical game states always\n  produce the same outcome. Not truly random; designed for\n  reproducible AI behavior.\n\nRETURN VALUE\n  true or false.\n\nERRORS\n  Wrong arg count (expects exactly 2).\n\nEXAMPLES\n  (if (roll-odds? self 0.3) (random-step! self))  ; 30% chance to wander",
            func: builtin_roll_oddsq,
        }),
    );
    env.bind(
        "manhattan",
        Value::Builtin(glyph::BuiltinFn {
            name: "manhattan",
            doc: "Manhattan distance between two entities\n\nSYNOPSIS\n  (manhattan entity-a entity-b)\n\nARGUMENTS\n  entity-a  Integer entity ID.\n  entity-b  Integer entity ID.\n\nDESCRIPTION\n  Returns the Manhattan distance (|x1-x2| + |y1-y2|) between\n  two entities. Returns 999 if either entity has no position.\n\nRETURN VALUE\n  An integer distance.\n\nERRORS\n  Wrong arg count (expects exactly 2).\n\nEXAMPLES\n  (if (< (manhattan self player-id) 5) (step-toward! self player-id))",
            func: builtin_manhattan,
        }),
    );
    env.bind(
        "hp",
        Value::Builtin(glyph::BuiltinFn {
            name: "hp",
            doc: "get the current HP of an entity\n\nSYNOPSIS\n  (hp entity-id)\n\nARGUMENTS\n  entity-id  Integer entity ID.\n\nDESCRIPTION\n  Returns the current hit points of the given entity. Returns 0\n  if the entity has no HP component (e.g., a decoration).\n\nRETURN VALUE\n  An integer (current HP).\n\nERRORS\n  Wrong arg count (expects exactly 1).\n\nEXAMPLES\n  (if (< (hp self) 3) (flee-step! self player-id))",
            func: builtin_ai_hp,
        }),
    );
}

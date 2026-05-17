//! Bracket-lib drawing code for the prototype.
//!
//! Rendering is deliberately a read-only projection of `World`: it paints the
//! map, warm flashlight cone, entities, event log, inspector text, and console
//! overlay without changing simulation state.

use std::collections::HashSet;

use bracket_lib::prelude::*;

use crate::{
    entity::{Entity, EntityKind, Position},
    game::{Mode, World},
    map::{TileType, FLASHLIGHT_RADIUS, MAP_HEIGHT, MAP_WIDTH},
    rules::{ENEMY_AI_SOURCE, FLASHLIGHT_SOURCE},
};

const SCREEN_HEIGHT: i32 = 50;
const MAP_X: i32 = 1;
const MAP_Y: i32 = 1;
const PANEL_X: i32 = 57;
const PANEL_Y: i32 = 1;
const PANEL_WIDTH: i32 = 22;
const PANEL_HEIGHT: i32 = 31;
const LOG_X: i32 = 1;
const LOG_Y: i32 = 33;
const LOG_WIDTH: i32 = 78;
const LOG_HEIGHT: i32 = 16;

pub fn render(ctx: &mut BTerm, world: &World) {
    let lit_tiles = world
        .map
        .flashlight_tiles(world.player.pos, world.player_facing);

    render_map(ctx, world, &lit_tiles);
    render_side_panel(ctx, world);
    render_event_log(ctx, world);

    if world.mode == Mode::Console {
        render_console(ctx, world);
    }
}

fn render_map(ctx: &mut BTerm, world: &World, lit_tiles: &HashSet<Position>) {
    draw_box(ctx, 0, 0, MAP_WIDTH + 2, MAP_HEIGHT + 2, " dungeon ");

    for y in 0..world.map.height {
        for x in 0..world.map.width {
            let pos = Position::new(x, y);
            let lit = lit_tiles.contains(&pos);
            let (glyph, fg) = match (world.map.tile(pos), lit) {
                (TileType::Floor, true) => ('.', RGB::named(GOLD)),
                (TileType::Wall, true) => ('#', RGB::named(LIGHT_YELLOW)),
                (TileType::Floor, false) => ('.', RGB::named(DARK_GRAY)),
                (TileType::Wall, false) => ('#', RGB::named(GRAY)),
            };
            ctx.set(MAP_X + x, MAP_Y + y, fg, RGB::named(BLACK), to_cp437(glyph));
        }
    }

    for enemy in world.living_enemies() {
        draw_entity(ctx, enemy, lit_tiles);
    }
    draw_entity(ctx, &world.player, lit_tiles);
}

fn draw_entity(ctx: &mut BTerm, entity: &Entity, lit_tiles: &HashSet<Position>) {
    let lit = lit_tiles.contains(&entity.pos) || entity.kind == EntityKind::Player;
    let color = match (entity.kind, lit) {
        (EntityKind::Player, _) => RGB::named(YELLOW),
        (EntityKind::Slime, true) => RGB::named(ORANGE),
        (EntityKind::Slime, false) => RGB::named(DARK_GREEN),
    };

    ctx.set(
        MAP_X + entity.pos.x,
        MAP_Y + entity.pos.y,
        color,
        RGB::named(BLACK),
        to_cp437(entity.glyph()),
    );
}

fn render_side_panel(ctx: &mut BTerm, world: &World) {
    draw_box(
        ctx,
        PANEL_X,
        PANEL_Y,
        PANEL_WIDTH,
        PANEL_HEIGHT,
        " runtime ",
    );

    let mut y = PANEL_Y + 2;
    print_clipped(ctx, PANEL_X + 2, y, PANEL_WIDTH - 4, "Xlyph prototype");
    y += 2;
    print_clipped(
        ctx,
        PANEL_X + 2,
        y,
        PANEL_WIDTH - 4,
        &format!("turn: {}", world.turn),
    );
    y += 1;
    print_clipped(
        ctx,
        PANEL_X + 2,
        y,
        PANEL_WIDTH - 4,
        &format!("mode: {:?}", world.mode),
    );
    y += 1;
    print_clipped(
        ctx,
        PANEL_X + 2,
        y,
        PANEL_WIDTH - 4,
        &format!("hp: {}/{}", world.player.hp.current, world.player.hp.max),
    );
    y += 1;
    print_clipped(
        ctx,
        PANEL_X + 2,
        y,
        PANEL_WIDTH - 4,
        &format!("lamp: {:?} r{}", world.player_facing, FLASHLIGHT_RADIUS),
    );
    y += 2;

    print_clipped(ctx, PANEL_X + 2, y, PANEL_WIDTH - 4, "controls");
    y += 1;
    for line in [
        "hjkl/arrows move",
        ". waits",
        "i inspector",
        "` console",
        "esc/q quit",
    ] {
        print_clipped(ctx, PANEL_X + 2, y, PANEL_WIDTH - 4, line);
        y += 1;
    }

    y += 1;
    print_clipped(ctx, PANEL_X + 2, y, PANEL_WIDTH - 4, "inspect: rules");
    y += 1;

    let visible_lines = (PANEL_Y + PANEL_HEIGHT - 2 - y).max(0) as usize;
    let rule_lines = ENEMY_AI_SOURCE
        .iter()
        .chain([""].iter())
        .chain(FLASHLIGHT_SOURCE.iter());

    for line in rule_lines.skip(world.inspector_scroll).take(visible_lines) {
        print_clipped(ctx, PANEL_X + 2, y, PANEL_WIDTH - 4, line);
        y += 1;
    }
}

fn render_event_log(ctx: &mut BTerm, world: &World) {
    draw_box(
        ctx,
        LOG_X - 1,
        LOG_Y - 1,
        LOG_WIDTH,
        LOG_HEIGHT,
        " event log ",
    );

    let visible_lines = (LOG_HEIGHT - 2) as usize;
    let entries = world.event_log.entries();
    let start = entries.len().saturating_sub(visible_lines);

    for (line_index, entry) in entries[start..].iter().enumerate() {
        print_clipped(
            ctx,
            LOG_X + 1,
            LOG_Y + line_index as i32,
            LOG_WIDTH - 4,
            entry,
        );
    }
}

fn render_console(ctx: &mut BTerm, world: &World) {
    let x = 8;
    let y = 20;
    let width = 64;
    let height = 7;
    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " forbidden console ");

    print_clipped(
        ctx,
        x + 2,
        y + 2,
        width - 4,
        "Read-only query shell. Enter logs a placeholder result.",
    );
    print_clipped(
        ctx,
        x + 2,
        y + 4,
        width - 4,
        &format!("> {}", world.console_buffer),
    );
}

fn draw_box(ctx: &mut BTerm, x: i32, y: i32, width: i32, height: i32, title: &str) {
    let fg = RGB::named(WHITE);
    let bg = RGB::named(BLACK);

    for dx in 0..width {
        ctx.set(x + dx, y, fg, bg, to_cp437('-'));
        ctx.set(x + dx, y + height - 1, fg, bg, to_cp437('-'));
    }

    for dy in 0..height {
        ctx.set(x, y + dy, fg, bg, to_cp437('|'));
        ctx.set(x + width - 1, y + dy, fg, bg, to_cp437('|'));
    }

    ctx.set(x, y, fg, bg, to_cp437('+'));
    ctx.set(x + width - 1, y, fg, bg, to_cp437('+'));
    ctx.set(x, y + height - 1, fg, bg, to_cp437('+'));
    ctx.set(x + width - 1, y + height - 1, fg, bg, to_cp437('+'));
    print_clipped(ctx, x + 2, y, width - 4, title);
}

fn fill_rect(ctx: &mut BTerm, x: i32, y: i32, width: i32, height: i32, color: RGB) {
    for dy in 0..height {
        for dx in 0..width {
            ctx.set(x + dx, y + dy, RGB::named(WHITE), color, to_cp437(' '));
        }
    }
}

fn print_clipped(ctx: &mut BTerm, x: i32, y: i32, max_width: i32, text: &str) {
    if max_width <= 0 || y < 0 || y >= SCREEN_HEIGHT {
        return;
    }

    let clipped: String = text.chars().take(max_width as usize).collect();
    ctx.print(x, y, clipped);
}

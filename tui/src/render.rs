//! Bracket-lib drawing code for the prototype.
//!
//! Rendering is deliberately a read-only projection of `World`: it paints the
//! map, warm flashlight cone, entities, event log, inspector text, and console
//! overlay without changing simulation state. Mouse hover over the map shows a
//! quick entity info tooltip.

use std::collections::HashSet;

use bracket_lib::prelude::*;

use crate::{
    entity::{EntityKind, EntityView, Position},
    game::{Mode, World},
    glyph::highlight::{self, Span},
    map::{TileType, FLASHLIGHT_RADIUS, MAP_HEIGHT, MAP_WIDTH},
};

const SCREEN_HEIGHT: i32 = 41;
const MAP_X: i32 = 1;
const MAP_Y: i32 = 1;
const PANEL_X: i32 = 42;
const PANEL_Y: i32 = 1;
const PANEL_WIDTH: i32 = 24;
const PANEL_HEIGHT: i32 = 27;
const LOG_X: i32 = 1;
const LOG_Y: i32 = 27;
const LOG_WIDTH: i32 = 66;
const LOG_HEIGHT: i32 = 14;

pub fn render(ctx: &mut BTerm, world: &World) {
    let lit_tiles = world
        .map
        .flashlight_tiles(world.player_pos(), world.player_facing);

    render_map(ctx, world, &lit_tiles);
    render_side_panel(ctx, world);
    render_event_log(ctx, world);

    if world.mode == Mode::Normal {
        render_entity_tooltip(ctx, world);
    }

    if world.mode == Mode::Inspector || world.mode == Mode::Console {
        render_overlay_backdrop(ctx);
    }

    if world.mode == Mode::Inspector {
        render_inspector(ctx, world);
    }

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
                (TileType::StairsDown, _) => ('>', RGB::named(CYAN)),
                (TileType::StairsUp, _) => ('<', RGB::named(MAGENTA)),
                (TileType::Floor, true) => ('.', RGB::named(GOLD)),
                (TileType::Wall, true) => ('#', RGB::named(LIGHT_YELLOW)),
                (TileType::Floor, false) => ('.', RGB::named(DARK_GRAY)),
                (TileType::Wall, false) => ('#', RGB::named(GRAY)),
            };
            ctx.set(MAP_X + x, MAP_Y + y, fg, RGB::named(BLACK), to_cp437(glyph));
        }
    }

    for entity in world.renderable_entities() {
        draw_entity(ctx, entity, lit_tiles);
    }
}

fn draw_entity(ctx: &mut BTerm, entity: EntityView, lit_tiles: &HashSet<Position>) {
    let lit = lit_tiles.contains(&entity.pos) || entity.kind == EntityKind::Player;
    let color = match (entity.kind, lit) {
        (EntityKind::Player, _) => RGB::named(YELLOW),
        (EntityKind::Slime, true) => RGB::named(ORANGE),
        (EntityKind::Slime, false) => RGB::named(DARK_GREEN),
        (EntityKind::Goblin, true) => RGB::named(RED),
        (EntityKind::Goblin, false) => RGB::named(DARK_RED),
        (EntityKind::Bat, true) => RGB::named(WHITE),
        (EntityKind::Bat, false) => RGB::named(GRAY),
        (EntityKind::Ogre, true) => RGB::named(MAGENTA),
        (EntityKind::Ogre, false) => RGB::named(PURPLE),
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

    // --- Stats (two-column) ---
    let c1 = PANEL_X + 2;
    let c2 = PANEL_X + 13;
    let w = PANEL_WIDTH - 4;

    print_clipped(ctx, c1, y, w, &format!("turn  {}", world.turn));
    print_clipped(ctx, c2, y, 10, &format!("depth {}", world.depth));
    y += 1;
    print_clipped(
        ctx,
        c1,
        y,
        10,
        &format!(
            "hp    {}/{}",
            world.player_hp().current,
            world.player_hp().max
        ),
    );
    print_clipped(ctx, c2, y, 10, &format!("mode  {:?}", world.mode));
    y += 1;
    print_clipped(
        ctx,
        c1,
        y,
        w,
        &format!("lamp  {:?} r{}", world.player_facing, FLASHLIGHT_RADIUS),
    );
    if world.blocking {
        ctx.print_color(c2, y, RGB::named(CYAN), RGB::named(BLACK), "guarding");
    }
    y += 2;

    // --- Keys section ---
    print_section_header(ctx, c1, y, w, "keys");
    y += 1;
    let controls: &[(&str, &str)] = &[
        ("move/atk", "hjkl/arrows"),
        ("wait", "."),
        ("block", "b"),
        ("descend", "shift+."),
        ("ascend", "shift+,"),
        ("inspect", "i"),
        ("console", "`"),
        ("quit", "esc/q"),
    ];
    for (label, key) in controls {
        print_clipped(ctx, c1, y, 8, label);
        print_clipped(ctx, c1 + 9, y, w - 9, key);
        y += 1;
    }
}

fn render_overlay_backdrop(ctx: &mut BTerm) {
    let bg = RGB::named(BLACK);
    for y in 0..SCREEN_HEIGHT {
        for x in 0..78i32.min(SCREEN_HEIGHT + 37) {
            ctx.set(x, y, RGB::named(BLACK), bg, to_cp437(' '));
        }
    }
    // Re-render is handled by the overlay draw that follows
}

fn render_inspector(ctx: &mut BTerm, world: &World) {
    let x = 2;
    let y = 1;
    let width = 62;
    let height = 38;

    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " rules ");

    let mut line_y = y + 2;
    let inner_w = width - 4;
    let rules = world.registry.iter().collect::<Vec<_>>();
    let selected = world.inspector_selection.min(rules.len().saturating_sub(1));

    if rules.is_empty() {
        ctx.print_color(
            x + 2,
            line_y,
            RGB::named(GRAY),
            RGB::named(BLACK),
            "(no rules loaded)",
        );
    }

    for (i, rule) in rules.iter().enumerate() {
        let expanded = i == selected;
        let prefix = if expanded { "v" } else { ">" };
        let hl = if expanded {
            RGB::named(YELLOW)
        } else {
            RGB::named(GRAY)
        };

        let header = format!("{prefix} {}", rule.name);
        ctx.print_color(x + 2, line_y, hl, RGB::named(BLACK), &header);
        line_y += 1;

        let meta = format!("   {:?} - {:?}", rule.phase, rule.cost);
        ctx.print_color(
            x + 2,
            line_y,
            RGB::named(DARK_GRAY),
            RGB::named(BLACK),
            &meta,
        );
        line_y += 1;

        if expanded {
            for src in rule.source_lines {
                let src_line = format!("   {src}");
                let spans = highlight::highlight(&src_line);
                print_highlighted(ctx, x + 2, line_y, inner_w, &spans);
                line_y += 1;
            }
        }

        line_y += 1;
    }

    let nav = format!(
        "j/k select  i/esc close  {}/{} rules",
        selected.saturating_add(1),
        rules.len()
    );
    print_clipped(ctx, x + 2, y + height - 2, inner_w, &nav);
}

fn print_section_header(ctx: &mut BTerm, x: i32, y: i32, max_width: i32, title: &str) {
    let header = format!("-- {title} ");
    let fill = "-".repeat((max_width as usize).saturating_sub(header.len()));
    let line = format!("{header}{fill}");
    ctx.print_color(x, y, RGB::named(CYAN), RGB::named(BLACK), &line);
}

/// Show a tooltip with entity info when the mouse hovers over a map position
/// that contains a living entity.
fn render_entity_tooltip(ctx: &mut BTerm, world: &World) {
    let (mx, my) = ctx.mouse_pos();
    let map_x = mx - MAP_X;
    let map_y = my - MAP_Y;

    if map_x < 0 || map_x >= world.map.width || map_y < 0 || map_y >= world.map.height {
        return;
    }

    let pos = Position::new(map_x, map_y);
    let Some(entity) = world.entity_at(pos) else {
        return;
    };

    let tip_lines = [
        format!("#{} {}", entity.id.raw(), entity.name()),
        format!("hp: {}/{}", entity.hp.current, entity.hp.max),
        format!("pos: ({}, {})", entity.pos.x, entity.pos.y),
    ];

    let tooltip_x = (mx + 2).min(78 - 24);
    let tooltip_y = my.min(LOG_Y - 4 - tip_lines.len() as i32);
    let tooltip_w = 24;
    let tooltip_h = tip_lines.len() as i32 + 2;

    let fg = RGB::named(WHITE);
    let bg = RGB::named(BLACK);

    for dx in 0..tooltip_w {
        ctx.set(tooltip_x + dx, tooltip_y, fg, bg, to_cp437('-'));
        ctx.set(
            tooltip_x + dx,
            tooltip_y + tooltip_h - 1,
            fg,
            bg,
            to_cp437('-'),
        );
    }

    for dy in 0..tooltip_h {
        ctx.set(tooltip_x, tooltip_y + dy, fg, bg, to_cp437('|'));
        ctx.set(
            tooltip_x + tooltip_w - 1,
            tooltip_y + dy,
            fg,
            bg,
            to_cp437('|'),
        );
    }

    ctx.set(tooltip_x, tooltip_y, fg, bg, to_cp437('+'));
    ctx.set(tooltip_x + tooltip_w - 1, tooltip_y, fg, bg, to_cp437('+'));
    ctx.set(tooltip_x, tooltip_y + tooltip_h - 1, fg, bg, to_cp437('+'));
    ctx.set(
        tooltip_x + tooltip_w - 1,
        tooltip_y + tooltip_h - 1,
        fg,
        bg,
        to_cp437('+'),
    );

    for (i, line) in tip_lines.iter().enumerate() {
        let clipped: String = line.chars().take(tooltip_w as usize - 2).collect();
        ctx.print(tooltip_x + 1, tooltip_y + 1 + i as i32, clipped);
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
    let x = 3;
    let y = 12;
    let width = 60;
    let height = 13;
    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " glyph console ");

    print_clipped(
        ctx,
        x + 2,
        y + 2,
        width - 4,
        "Glyph REPL. Try (help). Enter (quit) to exit.",
    );

    if !world.console_output.is_empty() {
        let output: String = world
            .console_output
            .chars()
            .take((width - 4) as usize)
            .collect();
        print_clipped(ctx, x + 2, y + 4, width - 4, &output);
    }

    print_clipped(
        ctx,
        x + 2,
        y + 6,
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

fn print_highlighted(ctx: &mut BTerm, x: i32, y: i32, max_width: i32, spans: &[Span]) {
    let mut cx = x;
    for span in spans {
        let fg = span.color();
        for ch in span.text.chars() {
            if cx < x + max_width {
                ctx.set(cx, y, fg, RGB::named(BLACK), to_cp437(ch));
                cx += 1;
            }
        }
    }
}

fn print_clipped(ctx: &mut BTerm, x: i32, y: i32, max_width: i32, text: &str) {
    if max_width <= 0 || !(0..SCREEN_HEIGHT).contains(&y) {
        return;
    }

    let clipped: String = text.chars().take(max_width as usize).collect();
    ctx.print(x, y, clipped);
}

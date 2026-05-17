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

const MIN_SCREEN_WIDTH: i32 = 60;
const MIN_SCREEN_HEIGHT: i32 = 18;
const MAX_PANEL_WIDTH: i32 = 42;
const MIN_PANEL_WIDTH: i32 = 24;
const SECTION_GAP: i32 = 1;

#[derive(Clone, Copy, Debug)]
struct Layout {
    screen_width: i32,
    screen_height: i32,
    map_x: i32,
    map_y: i32,
    map_view_width: i32,
    map_view_height: i32,
    panel_x: i32,
    panel_y: i32,
    panel_width: i32,
    panel_height: i32,
    log_x: i32,
    log_y: i32,
    log_width: i32,
    log_height: i32,
}

impl Layout {
    fn from_context(ctx: &BTerm) -> Option<Self> {
        let (width, height) = ctx.get_char_size();
        Self::from_size(width as i32, height as i32)
    }

    fn from_size(screen_width: i32, screen_height: i32) -> Option<Self> {
        if screen_width < MIN_SCREEN_WIDTH || screen_height < MIN_SCREEN_HEIGHT {
            return None;
        }

        let log_height = (screen_height / 5).clamp(4, 8);
        let top_height = screen_height - log_height;
        let preferred_panel_width = (screen_width / 3).clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
        let map_view_width = (screen_width - preferred_panel_width - 4).clamp(1, MAP_WIDTH);
        let panel_width = (screen_width - map_view_width - 4)
            .clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH)
            .min(screen_width - map_view_width - 4);
        let map_view_height = (top_height - 2).clamp(1, MAP_HEIGHT);
        let content_width = map_view_width + 2 + SECTION_GAP + panel_width;
        let content_x = ((screen_width - content_width) / 2).max(0);
        let panel_x = content_x + map_view_width + 2 + SECTION_GAP;

        Some(Self {
            screen_width,
            screen_height,
            map_x: content_x + 1,
            map_y: 1,
            map_view_width,
            map_view_height,
            panel_x,
            panel_y: 1,
            panel_width,
            panel_height: top_height - 1,
            log_x: content_x + 1,
            log_y: top_height,
            log_width: content_width,
            log_height,
        })
    }
}

pub fn render(ctx: &mut BTerm, world: &World) {
    let Some(layout) = Layout::from_context(ctx) else {
        render_too_small(ctx);
        return;
    };

    let lit_tiles = world
        .map
        .flashlight_tiles(world.player.pos, world.player_facing);

    render_map(ctx, world, &lit_tiles, &layout);
    render_side_panel(ctx, world, &layout);
    render_event_log(ctx, world, &layout);

    if world.mode == Mode::Console {
        render_console(ctx, world, &layout);
    }
}

fn render_too_small(ctx: &mut BTerm) {
    ctx.print(1, 1, "Terminal too small for Xlyph.");
    ctx.print(
        1,
        2,
        format!("Need at least {MIN_SCREEN_WIDTH}x{MIN_SCREEN_HEIGHT}."),
    );
}

fn render_map(ctx: &mut BTerm, world: &World, lit_tiles: &HashSet<Position>, layout: &Layout) {
    draw_box(
        ctx,
        layout.map_x - 1,
        0,
        layout.map_view_width + 2,
        layout.map_view_height + 2,
        " dungeon ",
        layout,
    );

    for y in 0..layout.map_view_height.min(world.map.height) {
        for x in 0..layout.map_view_width.min(world.map.width) {
            let pos = Position::new(x, y);
            let lit = lit_tiles.contains(&pos);
            let (glyph, fg) = match (world.map.tile(pos), lit) {
                (TileType::Floor, true) => ('.', RGB::named(GOLD)),
                (TileType::Wall, true) => ('#', RGB::named(LIGHT_YELLOW)),
                (TileType::Floor, false) => ('.', RGB::named(DARK_GRAY)),
                (TileType::Wall, false) => ('#', RGB::named(GRAY)),
            };
            ctx.set(
                layout.map_x + x,
                layout.map_y + y,
                fg,
                RGB::named(BLACK),
                to_cp437(glyph),
            );
        }
    }

    for enemy in world.living_enemies() {
        draw_entity(ctx, enemy, lit_tiles, layout);
    }
    draw_entity(ctx, &world.player, lit_tiles, layout);
}

fn draw_entity(ctx: &mut BTerm, entity: &Entity, lit_tiles: &HashSet<Position>, layout: &Layout) {
    if entity.pos.x >= layout.map_view_width || entity.pos.y >= layout.map_view_height {
        return;
    }

    let lit = lit_tiles.contains(&entity.pos) || entity.kind == EntityKind::Player;
    let color = match (entity.kind, lit) {
        (EntityKind::Player, _) => RGB::named(YELLOW),
        (EntityKind::Slime, true) => RGB::named(ORANGE),
        (EntityKind::Slime, false) => RGB::named(DARK_GREEN),
    };

    ctx.set(
        layout.map_x + entity.pos.x,
        layout.map_y + entity.pos.y,
        color,
        RGB::named(BLACK),
        to_cp437(entity.glyph()),
    );
}

fn render_side_panel(ctx: &mut BTerm, world: &World, layout: &Layout) {
    draw_box(
        ctx,
        layout.panel_x,
        layout.panel_y,
        layout.panel_width,
        layout.panel_height,
        " runtime ",
        layout,
    );

    let mut y = layout.panel_y + 2;
    print_clipped(
        ctx,
        layout.panel_x + 2,
        y,
        layout.panel_width - 4,
        "Xlyph prototype",
        layout,
    );
    y += 2;
    print_clipped(
        ctx,
        layout.panel_x + 2,
        y,
        layout.panel_width - 4,
        &format!("turn: {}", world.turn),
        layout,
    );
    y += 1;
    print_clipped(
        ctx,
        layout.panel_x + 2,
        y,
        layout.panel_width - 4,
        &format!("mode: {:?}", world.mode),
        layout,
    );
    y += 1;
    print_clipped(
        ctx,
        layout.panel_x + 2,
        y,
        layout.panel_width - 4,
        &format!("hp: {}/{}", world.player.hp.current, world.player.hp.max),
        layout,
    );
    y += 1;
    print_clipped(
        ctx,
        layout.panel_x + 2,
        y,
        layout.panel_width - 4,
        &format!("lamp: {:?} r{}", world.player_facing, FLASHLIGHT_RADIUS),
        layout,
    );
    y += 2;

    print_clipped(
        ctx,
        layout.panel_x + 2,
        y,
        layout.panel_width - 4,
        "controls",
        layout,
    );
    y += 1;
    for line in [
        "arrows or hjkl move",
        ". waits",
        "i inspector",
        "` console",
        "esc/q quit",
    ] {
        print_clipped(
            ctx,
            layout.panel_x + 2,
            y,
            layout.panel_width - 4,
            line,
            layout,
        );
        y += 1;
    }

    y += 1;
    print_clipped(
        ctx,
        layout.panel_x + 2,
        y,
        layout.panel_width - 4,
        "inspect: rules",
        layout,
    );
    y += 1;

    let visible_lines = (layout.panel_y + layout.panel_height - 2 - y).max(0) as usize;
    let rule_lines = ENEMY_AI_SOURCE
        .iter()
        .chain([""].iter())
        .chain(FLASHLIGHT_SOURCE.iter());

    for line in rule_lines.skip(world.inspector_scroll).take(visible_lines) {
        print_clipped(
            ctx,
            layout.panel_x + 2,
            y,
            layout.panel_width - 4,
            line,
            layout,
        );
        y += 1;
    }
}

fn render_event_log(ctx: &mut BTerm, world: &World, layout: &Layout) {
    draw_box(
        ctx,
        layout.log_x - 1,
        layout.log_y,
        layout.log_width,
        layout.log_height,
        " event log ",
        layout,
    );

    let visible_lines = (layout.log_height - 2).max(0) as usize;
    let entries = world.event_log.entries();
    let start = entries.len().saturating_sub(visible_lines);

    for (line_index, entry) in entries[start..].iter().enumerate() {
        print_clipped(
            ctx,
            layout.log_x + 1,
            layout.log_y + 1 + line_index as i32,
            layout.log_width - 4,
            entry,
            layout,
        );
    }
}

fn render_console(ctx: &mut BTerm, world: &World, layout: &Layout) {
    let width = (layout.screen_width - 4).min(64);
    let height = 7;
    let x = (layout.screen_width - width) / 2;
    let y = (layout.screen_height - height) / 2;
    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " forbidden console ", layout);

    print_clipped(
        ctx,
        x + 2,
        y + 2,
        width - 4,
        "Read-only query shell. Enter logs a placeholder result.",
        layout,
    );
    print_clipped(
        ctx,
        x + 2,
        y + 4,
        width - 4,
        &format!("> {}", world.console_buffer),
        layout,
    );
}

fn draw_box(
    ctx: &mut BTerm,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    title: &str,
    layout: &Layout,
) {
    if width < 2 || height < 2 {
        return;
    }

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
    print_clipped(ctx, x + 2, y, width - 4, title, layout);
}

fn fill_rect(ctx: &mut BTerm, x: i32, y: i32, width: i32, height: i32, color: RGB) {
    for dy in 0..height {
        for dx in 0..width {
            ctx.set(x + dx, y + dy, RGB::named(WHITE), color, to_cp437(' '));
        }
    }
}

fn print_clipped(ctx: &mut BTerm, x: i32, y: i32, max_width: i32, text: &str, layout: &Layout) {
    if max_width <= 0 || y < 0 || y >= layout.screen_height || x >= layout.screen_width {
        return;
    }

    let available = max_width.min(layout.screen_width - x).max(0) as usize;
    let clipped: String = text.chars().take(available).collect();
    ctx.print(x, y, clipped);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_terminals_keep_the_play_surface_bounded() {
        let layout = Layout::from_size(200, 50).expect("wide terminal should fit");

        assert_eq!(layout.map_view_width, MAP_WIDTH);
        assert_eq!(layout.panel_width, MAX_PANEL_WIDTH);
        assert_eq!(
            layout.log_width,
            MAP_WIDTH + 2 + SECTION_GAP + MAX_PANEL_WIDTH
        );
        assert!(layout.map_x > 1);
        assert!(layout.panel_x + layout.panel_width < layout.screen_width);
    }

    #[test]
    fn minimum_supported_terminal_still_fits_all_boxes() {
        let layout = Layout::from_size(MIN_SCREEN_WIDTH, MIN_SCREEN_HEIGHT)
            .expect("minimum terminal should fit");

        assert!(layout.map_view_width > 0);
        assert!(layout.map_view_height > 0);
        assert!(layout.panel_x + layout.panel_width <= layout.screen_width);
        assert!(layout.log_y + layout.log_height <= layout.screen_height);
    }

    #[test]
    fn too_small_terminals_are_rejected() {
        assert!(Layout::from_size(MIN_SCREEN_WIDTH - 1, MIN_SCREEN_HEIGHT).is_none());
        assert!(Layout::from_size(MIN_SCREEN_WIDTH, MIN_SCREEN_HEIGHT - 1).is_none());
    }
}

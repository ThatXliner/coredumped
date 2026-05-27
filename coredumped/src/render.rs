//! Terminal drawing code for the prototype.
//!
//! Rendering is deliberately a read-only projection of `World`: it paints the
//! map, warm flashlight cone, entities, event log, inspector text, and console
//! overlay without changing simulation state. Mouse hover over the map shows a
//! quick entity info tooltip.

use std::collections::HashSet;

use bracket_color::prelude::*;

use crate::{
    entity::{EntityKind, EntityView, Position},
    event_log::LogEntry,
    game::Mode,
    glyph::highlight::{self, Span},
    map::{TileType, FLASHLIGHT_RADIUS},
    terminal::Frame,
    world::World,
};

const MAP_X: i32 = 1;
const MAP_Y: i32 = 1;
/// Viewport dimensions — the visible portion of the map.
pub const VIEWPORT_WIDTH: i32 = 40;
pub const VIEWPORT_HEIGHT: i32 = 24;
const MAP_BOX_WIDTH: i32 = VIEWPORT_WIDTH + 2;
const MAP_BOX_HEIGHT: i32 = VIEWPORT_HEIGHT + 2;
const PANEL_MIN_WIDTH: i32 = 18;
const MIN_BOTTOM_LOG_HEIGHT: i32 = 10;

pub fn render(ctx: &mut Frame, world: &World) {
    // lit_tiles computed via cache — render takes &World (not &mut) so we can't call
    // world.lit_tiles() here. The cache is populated by mark_visible_entities in tick().
    // Fall back to computing directly if cache is stale.
    let pos = world.player_pos();
    let facing = world.player_facing;
    let lit_tiles =
        if pos == world.cached_flashlight_pos && facing == world.cached_flashlight_facing {
            world.cached_flashlight.clone()
        } else {
            world.map.flashlight_tiles(pos, facing)
        };

    render_map(ctx, world, &lit_tiles);
    render_side_panel(ctx, world);
    if !event_log_uses_side_panel(ctx) {
        render_event_log(ctx, world);
    }

    if world.mode == Mode::Normal {
        render_entity_tooltip(ctx, world);
    }

    if world.mode == Mode::Dead {
        render_overlay_backdrop(ctx);
        render_death_screen(ctx, world);
        return;
    }

    let needs_overlay = matches!(
        world.mode,
        Mode::Inspector | Mode::Keybindings | Mode::Memories
    );
    if needs_overlay {
        render_overlay_backdrop(ctx);
    }

    match world.mode {
        Mode::Inspector => render_inspector(ctx, world),
        Mode::Keybindings => render_keybindings(ctx, world),
        Mode::Memories => render_memories(ctx, world),
        Mode::Console => render_console(ctx, world),
        _ => {}
    }
}

fn render_map(ctx: &mut Frame, world: &World, lit_tiles: &HashSet<Position>) {
    draw_box(ctx, 0, 0, MAP_BOX_WIDTH, MAP_BOX_HEIGHT, " dungeon ");

    let cam_x = world.camera_x;
    let cam_y = world.camera_y;
    let player_pos = world.player_pos();
    let flashlight_radius = FLASHLIGHT_RADIUS as f32;

    for vy in 0..VIEWPORT_HEIGHT {
        for vx in 0..VIEWPORT_WIDTH {
            let map_x = cam_x + vx;
            let map_y = cam_y + vy;
            let pos = Position::new(map_x, map_y);

            if !world.map.contains(pos) {
                ctx.set(MAP_X + vx, MAP_Y + vy, RGB::named(BLACK), RGB::named(BLACK), ' ');
                continue;
            }

            let lit = lit_tiles.contains(&pos);
            let explored = world.explored_tiles.contains(&pos);

            // Fog of war: unexplored tiles are hidden
            if !lit && !explored {
                ctx.set(MAP_X + vx, MAP_Y + vy, RGB::named(BLACK), RGB::named(BLACK), ' ');
                continue;
            }

            // Distance-based fade for lit tiles
            let fade = if lit {
                let dx = (pos.x - player_pos.x) as f32;
                let dy = (pos.y - player_pos.y) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                // Fade from 1.0 (close) to 0.4 (at radius edge)
                1.0 - (dist / flashlight_radius) * 0.6
            } else {
                1.0
            };

            let (glyph, base_fg) = match (world.map.tile(pos), lit) {
                // Lit tiles: full color (will be faded by distance)
                (TileType::StairsDown, true) => ('>', RGB::named(CYAN)),
                (TileType::StairsUp, true) => ('<', RGB::named(MAGENTA)),
                (TileType::Floor, true) => ('.', RGB::named(GOLD)),
                (TileType::Wall, true) => ('#', RGB::named(LIGHT_YELLOW)),
                (TileType::Fire, true) => ('^', RGB::named(RED)),
                (TileType::Lamp, _) => ('#', RGB::named(YELLOW)),
                // Remembered tiles: dimmed but readable
                (TileType::StairsDown, false) => ('>', RGB::from_u8(60, 120, 120)),
                (TileType::StairsUp, false) => ('<', RGB::from_u8(100, 60, 100)),
                (TileType::Floor, false) => ('.', RGB::from_u8(70, 70, 70)),
                (TileType::Wall, false) => ('#', RGB::from_u8(100, 100, 100)),
                (TileType::Fire, false) => ('^', RGB::from_u8(120, 60, 60)),
            };

            let fg = if lit && !matches!(world.map.tile(pos), TileType::Lamp) {
                RGB::from_f32(base_fg.r * fade, base_fg.g * fade, base_fg.b * fade)
            } else {
                base_fg
            };
            ctx.set(MAP_X + vx, MAP_Y + vy, fg, RGB::named(BLACK), glyph);
        }
    }

    for entity in world.renderable_entities() {
        draw_entity(ctx, world, entity, lit_tiles);
    }
}

fn draw_entity(ctx: &mut Frame, world: &World, entity: EntityView, lit_tiles: &HashSet<Position>) {
    let screen_x = entity.pos.x - world.camera_x;
    let screen_y = entity.pos.y - world.camera_y;

    if screen_x < 0 || screen_x >= VIEWPORT_WIDTH || screen_y < 0 || screen_y >= VIEWPORT_HEIGHT {
        return;
    }

    let is_player = entity.kind == EntityKind::Player;
    let lit = lit_tiles.contains(&entity.pos) || is_player;

    // Only show entities in lit areas (player always visible, enemies hidden in fog)
    if !lit && !is_player {
        return;
    }
    let (fg, bg) = match (entity.kind, lit) {
        (EntityKind::Player, _) => (RGB::named(YELLOW), RGB::named(BLACK)),
        (EntityKind::Slime, true) => (RGB::named(ORANGE), RGB::named(BLACK)),
        (EntityKind::Slime, false) => (RGB::named(DARK_GREEN), RGB::named(BLACK)),
        (EntityKind::Goblin, true) => (RGB::named(RED), RGB::named(BLACK)),
        (EntityKind::Goblin, false) => (RGB::named(DARK_RED), RGB::named(BLACK)),
        (EntityKind::Bat, true) => (RGB::named(WHITE), RGB::named(BLACK)),
        (EntityKind::Bat, false) => (RGB::named(GRAY), RGB::named(BLACK)),
        (EntityKind::Ogre, true) => (RGB::named(MAGENTA), RGB::named(BLACK)),
        (EntityKind::Ogre, false) => (RGB::named(PURPLE), RGB::named(BLACK)),
        (EntityKind::Wizard, true) => (RGB::named(CYAN), RGB::from_u8(0, 0, 60)),
        (EntityKind::Wizard, false) => (RGB::named(BLUE), RGB::from_u8(0, 0, 40)),
        (EntityKind::Barrel, true) => (RGB::from_u8(180, 100, 30), RGB::named(BLACK)),
        (EntityKind::Barrel, false) => (RGB::from_u8(80, 50, 20), RGB::named(BLACK)),
        (EntityKind::Sign, true) => (RGB::from_u8(200, 200, 100), RGB::named(BLACK)),
        (EntityKind::Sign, false) => (RGB::from_u8(120, 120, 60), RGB::named(BLACK)),
        (EntityKind::Fragment, true) => (RGB::named(GREEN), RGB::from_u8(0, 30, 0)),
        (EntityKind::Fragment, false) => (RGB::named(DARK_GREEN), RGB::named(BLACK)),
        (EntityKind::Shade, true) => (RGB::named(GRAY), RGB::from_u8(20, 20, 20)),
        (EntityKind::Shade, false) => (RGB::named(DARK_GRAY), RGB::named(BLACK)),
        (EntityKind::Rage, true) => (RGB::named(RED), RGB::from_u8(40, 0, 0)),
        (EntityKind::Rage, false) => (RGB::named(DARK_RED), RGB::named(BLACK)),
        (EntityKind::Sentry, true) => (RGB::named(WHITE), RGB::from_u8(30, 30, 30)),
        (EntityKind::Sentry, false) => (RGB::named(GRAY), RGB::named(BLACK)),
        (EntityKind::ShadeEcho, true) => (RGB::named(CYAN), RGB::from_u8(0, 30, 30)),
        (EntityKind::ShadeEcho, false) => (RGB::named(DARK_GRAY), RGB::named(BLACK)),
        (EntityKind::VaporCanteen, true) => (RGB::named(CYAN), RGB::from_u8(0, 20, 40)),
        (EntityKind::VaporCanteen, false) => (RGB::named(DARK_BLUE), RGB::named(BLACK)),
    };

    let x = MAP_X + screen_x;
    let y = MAP_Y + screen_y;
    ctx.set(x, y, fg, bg, entity.glyph());
}

fn render_side_panel(ctx: &mut Frame, world: &World) {
    let panel_x = MAP_BOX_WIDTH;
    let panel_y = 0;
    let panel_width = (ctx.width() - panel_x).max(0);
    let panel_height = top_panel_height(ctx);
    if panel_width < PANEL_MIN_WIDTH || panel_height < 3 {
        return;
    }

    draw_box(
        ctx,
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        " runtime ",
    );

    let mut y = panel_y + 2;

    // --- Stats (two-column) ---
    let c1 = panel_x + 2;
    let c2 = panel_x + 13;
    let w = panel_width - 4;

    print_clipped(ctx, c1, y, w, &format!("turn  {}", world.turn));
    print_clipped(ctx, c2, y, 10, &format!("depth {}", world.depth));
    y += 1;
    print_clipped(
        ctx,
        c1,
        y,
        13,
        &format!("hp {}/{}", world.player_hp().current, world.player_hp().max),
    );
    print_clipped(ctx, c2, y, 14, &format!("mode {:?}", world.mode));
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
    if world.cheat_unlocked {
        ctx.print_color(c2 + 9, y, RGB::named(GREEN), RGB::named(BLACK), "cheat!");
    }
    y += 2;

    let content_bottom = panel_y + panel_height - 2;
    let log_reserved_rows = if event_log_uses_side_panel(ctx) { 8 } else { 0 };
    let max_key_rows = (content_bottom - y + 1 - log_reserved_rows).max(0);
    y = render_key_summary(ctx, world, c1, y, w, max_key_rows);

    // Fragment count
    let collected = world.fragment_registry.collected_count();
    if collected > 0 && y <= content_bottom {
        y += 1;
        if y <= content_bottom {
            print_clipped(ctx, c1, y, w, &format!("memories {}/33", collected));
        }
    }

    // Ending display
    if let Some(ref ending) = world.ending {
        if y <= content_bottom {
            y += 2;
            print_clipped_color(ctx, c1, y, w, "--- ENDING ---", RGB::named(YELLOW));
            y += 1;
            for line in ending.lines() {
                if y < panel_y + panel_height - 2 {
                    print_clipped(ctx, c1, y, w, line);
                }
                y += 1;
            }
        }
    }

    if event_log_uses_side_panel(ctx) {
        y += 2;
        let log_height = panel_y + panel_height - 1 - y;
        render_event_log_panel(ctx, world, c1, y, w, log_height);
    }
}

fn render_key_summary(
    ctx: &mut Frame,
    world: &World,
    x: i32,
    mut y: i32,
    width: i32,
    max_rows: i32,
) -> i32 {
    if max_rows <= 0 {
        return y;
    }

    let start_y = y;
    let mut rows_used = 0;

    print_section_header(ctx, x, y, width, "keys");
    y += 1;
    rows_used += 1;

    if !world.player_can_attack && rows_used < max_rows {
        print_clipped(ctx, x, y, width, "HELPLESS - find wizard!");
        y += 1;
        rows_used += 1;
    }

    let summaries = binding_summaries(world);
    let remaining_rows = (max_rows - rows_used).max(0) as usize;
    let needs_hint = summaries.len() > remaining_rows;
    let binding_rows = if needs_hint {
        remaining_rows.saturating_sub(1)
    } else {
        remaining_rows
    };

    for summary in summaries.iter().take(binding_rows) {
        render_binding_summary(ctx, summary, x, y, width);
        y += 1;
        rows_used += 1;
    }

    if needs_hint && rows_used < max_rows {
        print_clipped_color(ctx, x, y, width, "See more bindings: Tab", RGB::named(CYAN));
        y += 1;
    }

    y.max(start_y + 1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BindingSummary {
    command: String,
    label: String,
    keys: Vec<String>,
    is_new: bool,
}

fn render_binding_summary(ctx: &mut Frame, summary: &BindingSummary, x: i32, y: i32, width: i32) {
    if width <= 0 {
        return;
    }

    let key_text = summary
        .keys
        .iter()
        .map(|key| display_key(key))
        .collect::<Vec<_>>()
        .join("/");
    let key_width = key_text.chars().count() as i32;
    let label_width = (width - key_width - 1).max(1);
    let label = if summary.is_new {
        format!("{} [new]", summary.label)
    } else {
        summary.label.clone()
    };
    let label_color = if summary.is_new {
        RGB::named(YELLOW)
    } else {
        RGB::named(WHITE)
    };

    print_clipped_color(ctx, x, y, label_width, &label, label_color);
    if key_width > 0 && label_width + 1 < width {
        print_clipped(
            ctx,
            x + label_width + 1,
            y,
            width - label_width - 1,
            &key_text,
        );
    }
}

fn binding_summaries(world: &World) -> Vec<BindingSummary> {
    let mut summaries: Vec<BindingSummary> = Vec::new();
    for (key, command) in &world.bindings {
        if let Some(summary) = summaries
            .iter_mut()
            .find(|summary| summary.command == *command)
        {
            summary.keys.push(key.clone());
            summary.is_new |= world.new_binding_keys.contains(key);
            continue;
        }

        summaries.push(BindingSummary {
            command: command.clone(),
            label: binding_label(command),
            keys: vec![key.clone()],
            is_new: world.new_binding_keys.contains(key)
                || (command == "(toggle-inspector!)" && !world.new_rule_ids.is_empty())
                || (command == "(toggle-keybindings!)" && world.has_new_bindings),
        });
    }

    for summary in &mut summaries {
        summary.keys.sort_by_key(|key| binding_key_priority(key));
    }
    summaries.sort_by(|a, b| {
        binding_command_priority(&a.command)
            .cmp(&binding_command_priority(&b.command))
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.keys.cmp(&b.keys))
    });

    summaries
}

fn binding_label(command: &str) -> String {
    match command {
        "(move! :north)" => "move north".into(),
        "(move! :south)" => "move south".into(),
        "(move! :west)" => "move west".into(),
        "(move! :east)" => "move east".into(),
        "(wait!)" => "wait".into(),
        "(descend!)" => "descend".into(),
        "(ascend!)" => "ascend".into(),
        "(block!)" => "block".into(),
        "(shove!)" => "shove".into(),
        "(toggle-inspector!)" => "inspector".into(),
        "(toggle-console!)" => "console".into(),
        "(toggle-keybindings!)" => "bindings".into(),
        "(quit!)" => "quit".into(),
        command if command.contains("do-attack") => "attack".into(),
        command => command.to_string(),
    }
}

fn binding_command_priority(command: &str) -> usize {
    match command {
        "(move! :north)" => 0,
        "(move! :south)" => 1,
        "(move! :west)" => 2,
        "(move! :east)" => 3,
        "(wait!)" => 4,
        "(descend!)" => 5,
        "(ascend!)" => 6,
        "(block!)" => 7,
        "(shove!)" => 8,
        "(toggle-inspector!)" => 9,
        "(toggle-console!)" => 10,
        "(toggle-keybindings!)" => 11,
        "(quit!)" => 12,
        command if command.contains("do-attack") => 13,
        _ => 100,
    }
}

fn binding_key_priority(key: &str) -> usize {
    match key {
        "h" => 0,
        "j" => 1,
        "k" => 2,
        "l" => 3,
        "left" => 4,
        "down" => 5,
        "up" => 6,
        "right" => 7,
        "." => 8,
        ">" => 9,
        "<" => 10,
        "b" => 11,
        "v" => 12,
        "i" => 13,
        "`" => 14,
        "tab" => 15,
        "q" => 16,
        key => 100 + key.bytes().map(usize::from).sum::<usize>(),
    }
}

fn render_overlay_backdrop(ctx: &mut Frame) {
    let bg = RGB::named(BLACK);
    for y in 0..ctx.height() {
        for x in 0..ctx.width() {
            ctx.set(x, y, RGB::named(BLACK), bg, ' ');
        }
    }
    // Re-render is handled by the overlay draw that follows
}

fn render_inspector(ctx: &mut Frame, world: &World) {
    let x = if ctx.width() > 4 { 2 } else { 0 };
    let y = if ctx.height() > 3 { 1 } else { 0 };
    let width = (ctx.width() - x * 2).max(1);
    let height = (ctx.height() - y - 1).max(1);

    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " rules ");

    let mut line_y = y + 2;
    let inner_w = (width - 4).max(1);
    let rules: Vec<_> = world
        .registry
        .iter()
        .filter(|r| world.known_rule_ids.contains(r.id))
        .collect();
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

    let has_new = !world.new_rule_ids.is_empty();

    for (i, rule) in rules.iter().enumerate() {
        let expanded = i == selected;
        let is_new = world.new_rule_ids.contains(rule.id);
        let prefix = if expanded { "v" } else { ">" };
        let hl = if expanded {
            RGB::named(YELLOW)
        } else if is_new {
            RGB::named(CYAN)
        } else {
            RGB::named(GRAY)
        };

        let header = if is_new {
            format!("{prefix} {} [NEW]", rule.name)
        } else {
            format!("{prefix} {}", rule.name)
        };
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

    let nav = if has_new {
        format!(
            "j/k select  i/esc close  {}/{} rules  (new rules highlighted)",
            selected.saturating_add(1),
            rules.len()
        )
    } else {
        format!(
            "j/k select  i/esc close  {}/{} rules",
            selected.saturating_add(1),
            rules.len()
        )
    };
    print_clipped(ctx, x + 2, y + height - 2, inner_w, &nav);
}

fn print_section_header(ctx: &mut Frame, x: i32, y: i32, max_width: i32, title: &str) {
    let header = format!("-- {title} ");
    let fill = "-".repeat((max_width as usize).saturating_sub(header.len()));
    let line = format!("{header}{fill}");
    ctx.print_color(x, y, RGB::named(CYAN), RGB::named(BLACK), &line);
}

/// Show a tooltip with entity info when the mouse hovers over a map position
/// that contains a living entity.
fn render_entity_tooltip(ctx: &mut Frame, world: &World) {
    let (mx, my) = ctx.mouse_pos();
    let screen_x = mx - MAP_X;
    let screen_y = my - MAP_Y;

    if screen_x < 0 || screen_x >= VIEWPORT_WIDTH || screen_y < 0 || screen_y >= VIEWPORT_HEIGHT {
        return;
    }

    let map_x = world.camera_x + screen_x;
    let map_y = world.camera_y + screen_y;

    if !world.map.contains(Position::new(map_x, map_y)) {
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

    let tooltip_w = 24.min(ctx.width().max(1));
    let tooltip_x = (mx + 2).min(ctx.width() - tooltip_w).max(0);
    let tooltip_y = my.min(log_box_y(ctx) - 3 - tip_lines.len() as i32).max(0);
    let tooltip_h = tip_lines.len() as i32 + 2;

    let fg = RGB::named(WHITE);
    let bg = RGB::named(BLACK);

    fill_rect(ctx, tooltip_x, tooltip_y, tooltip_w, tooltip_h, bg);

    for dx in 0..tooltip_w {
        ctx.set(tooltip_x + dx, tooltip_y, fg, bg, '-');
        ctx.set(tooltip_x + dx, tooltip_y + tooltip_h - 1, fg, bg, '-');
    }

    for dy in 0..tooltip_h {
        ctx.set(tooltip_x, tooltip_y + dy, fg, bg, '|');
        ctx.set(tooltip_x + tooltip_w - 1, tooltip_y + dy, fg, bg, '|');
    }

    ctx.set(tooltip_x, tooltip_y, fg, bg, '+');
    ctx.set(tooltip_x + tooltip_w - 1, tooltip_y, fg, bg, '+');
    ctx.set(tooltip_x, tooltip_y + tooltip_h - 1, fg, bg, '+');
    ctx.set(
        tooltip_x + tooltip_w - 1,
        tooltip_y + tooltip_h - 1,
        fg,
        bg,
        '+',
    );

    for (i, line) in tip_lines.iter().enumerate() {
        let clipped: String = line.chars().take(tooltip_w as usize - 2).collect();
        ctx.print_color(tooltip_x + 1, tooltip_y + 1 + i as i32, fg, bg, &clipped);
    }
}

fn render_event_log(ctx: &mut Frame, world: &World) {
    let log_y = log_box_y(ctx);
    let log_height = log_box_height(ctx);
    let log_width = ctx.width();
    if log_height < 3 || log_width < 4 {
        return;
    }

    draw_box(ctx, 0, log_y, log_width, log_height, " event log ");

    let text_width = (log_width - 4).max(1);
    render_event_log_lines(
        ctx,
        world,
        1,
        log_y + 1,
        text_width,
        (log_height - 2).max(0),
    );
}

fn render_event_log_panel(ctx: &mut Frame, world: &World, x: i32, y: i32, width: i32, height: i32) {
    if width < 4 || height < 2 {
        return;
    }

    print_section_header(ctx, x, y, width, "event log");
    render_event_log_lines(ctx, world, x, y + 1, width, height - 1);
}

fn render_event_log_lines(ctx: &mut Frame, world: &World, x: i32, y: i32, width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }

    let visible_lines = height as usize;
    let lines = wrapped_log_lines(world.event_log.entries(), width as usize);
    let max_start = lines.len().saturating_sub(visible_lines);
    let start = max_start.saturating_sub(world.event_log_scroll);

    for (line_index, line) in lines[start..].iter().take(visible_lines).enumerate() {
        let y = y + line_index as i32;
        if let Some(color) = line.color {
            print_clipped_color(ctx, x, y, width, &line.text, color);
        } else {
            print_clipped(ctx, x, y, width, &line.text);
        }
    }
}

fn wrapped_log_lines(entries: &[LogEntry], max_width: usize) -> Vec<LogEntry> {
    entries
        .iter()
        .flat_map(|entry| {
            wrap_text(&entry.text, max_width)
                .into_iter()
                .map(|text| LogEntry {
                    text,
                    color: entry.color,
                })
        })
        .collect()
}

fn render_console(ctx: &mut Frame, world: &World) {
    let (x, y, width, height) = console_bounds(ctx);
    let inner_width = (width - 4).max(1);
    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " glyph console ");

    for (i, line) in wrap_text(
        "Glyph REPL — press Ctrl+E to open an external editor for multi-line input. Try (help). ESC or ` to close.",
        inner_width as usize,
    )
    .iter()
    .enumerate()
    {
        let trimmed: String = line.chars().take(inner_width as usize).collect();
        ctx.print_color(
            x + 2,
            y + 2 + i as i32,
            RGB::named(WHITE),
            RGB::named(BLACK),
            &trimmed,
        );
    }

    let output_y = y + 4;
    let prompt_y = y + height - 2;
    let output_height = prompt_y - output_y - 1; // total lines available for output + input

    // Wrap input buffer into lines for multi-line display (word-wrap at inner width)
    let input_wrap_width = (width - 6).max(1) as usize;
    let input_wrapped = if world.console_buffer.is_empty() {
        vec![String::new()]
    } else {
        wrap_text(&world.console_buffer, input_wrap_width)
    };
    let cursor_visual = cursor_visual_pos(
        &world.console_buffer,
        world.console_cursor,
        input_wrap_width,
    );
    let cursor_visual_line = cursor_visual.0;
    let cursor_visual_col = cursor_visual.1;

    // Reserve space for input at the bottom — cap to keep at least 1 line for output
    let max_input_lines = (output_height - 1).max(1) as usize;
    let input_line_count = input_wrapped.len().min(max_input_lines);
    let first_visible_input_line = if input_wrapped.len() > max_input_lines {
        cursor_visual_line
            .saturating_add(1)
            .saturating_sub(max_input_lines)
            .min(input_wrapped.len().saturating_sub(max_input_lines))
    } else {
        0
    };
    let visible_input =
        &input_wrapped[first_visible_input_line..first_visible_input_line + input_line_count];

    // Output rendered in remaining space above the input area
    let output_available = output_height - input_line_count as i32;
    if !world.console_output.is_empty() && output_available > 0 {
        let lines = if is_diagnostic_output(&world.console_output) {
            clipped_lines(&world.console_output, inner_width as usize)
        } else {
            wrap_text(&world.console_output, inner_width as usize)
        };
        let visible_output = output_available as usize;
        let max_start = lines.len().saturating_sub(visible_output);
        let start = max_start.saturating_sub(world.console_output_scroll);
        let highlight_output = console_output_uses_syntax_highlighting(&world.console_output);
        for (i, line) in lines[start..].iter().enumerate().take(visible_output) {
            let y = output_y + i as i32;
            if let Some(color) = world.console_output_color {
                print_clipped_color(ctx, x + 2, y, inner_width, line, color);
            } else if highlight_output {
                let spans = highlight::highlight(line);
                print_highlighted(ctx, x + 2, y, inner_width, &spans);
            } else {
                print_clipped(ctx, x + 2, y, inner_width, line);
            }
        }
    }

    // Render wrapped input lines at the bottom
    let input_inner_width = (width - 6).max(1);
    let input_start_y = prompt_y - input_line_count as i32 + 1;

    for (i, line) in visible_input.iter().enumerate() {
        let line_y = input_start_y + i as i32;
        let is_cursor_on_this_line = first_visible_input_line + i == cursor_visual_line;

        // Prompt prefix: "> " on last line, "  " elsewhere
        if i == visible_input.len() - 1 {
            ctx.print_color(x + 2, line_y, RGB::named(WHITE), RGB::named(BLACK), "> ");
        } else {
            ctx.print_color(x + 2, line_y, RGB::named(GRAY), RGB::named(BLACK), "  ");
        }

        if is_cursor_on_this_line {
            // Split line at cursor column so we can draw the block cursor at the right spot
            let before: String = line.chars().take(cursor_visual_col).collect();
            let rest: String = line.chars().skip(cursor_visual_col).collect();
            let cursor_char = rest.chars().next().unwrap_or(' ');

            // Text before cursor (normally highlighted)
            let spans_before = highlight::highlight(&before);
            print_highlighted(ctx, x + 4, line_y, input_inner_width, &spans_before);

            // Block cursor at the visual position
            let cursor_x = x + 4 + (before.chars().count() as i32).min(input_inner_width);
            if cursor_x < x + 2 + input_inner_width {
                ctx.set(
                    cursor_x,
                    line_y,
                    RGB::named(BLACK),
                    RGB::named(WHITE),
                    cursor_char,
                );
            }

            // Text after cursor (normally highlighted from x+5+offset)
            let spans_rest = highlight::highlight(&rest);
            print_highlighted(ctx, cursor_x + 1, line_y, input_inner_width, &spans_rest);
        } else {
            let spans = highlight::highlight(line);
            print_highlighted(ctx, x + 4, line_y, input_inner_width, &spans);
        }
    }
}

fn console_bounds(ctx: &Frame) -> (i32, i32, i32, i32) {
    let margin_x = if ctx.width() > 8 { 3 } else { 0 };
    let margin_y = if ctx.height() > 8 { 2 } else { 0 };
    let max_width = (ctx.width() - margin_x * 2).max(1);
    let max_height = (ctx.height() - margin_y * 2).max(1);
    let width = max_width.min(120);
    let height = max_height.min(24);
    let x = ((ctx.width() - width) / 2).max(0);
    let y = (ctx.height() - height - margin_y).max(0);

    (x, y, width, height)
}

/// Map a byte offset in `text` to a (wrapped_line, column) position in the
/// word-wrapped display at `max_width`.
fn cursor_visual_pos(text: &str, cursor_byte: usize, max_width: usize) -> (usize, usize) {
    let prefix = &text[..cursor_byte.min(text.len())];
    let wrapped = wrap_text(prefix, max_width);
    let line = wrapped.len().saturating_sub(1);
    let col = wrapped.last().map(|s| s.chars().count()).unwrap_or(0);
    (line, col)
}

fn is_diagnostic_output(text: &str) -> bool {
    text.contains("Error: syntax error") && text.contains("[glyph:")
}

fn console_output_uses_syntax_highlighting(text: &str) -> bool {
    text.starts_with("=> Glyph help ")
}

fn clipped_lines(text: &str, max_width: usize) -> Vec<String> {
    text.lines()
        .map(|line| line.chars().take(max_width).collect())
        .collect()
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut remaining = raw_line;
        while remaining.chars().count() > max_width {
            let mut split = 0;
            let mut last_space = None;
            for (idx, ch) in remaining.char_indices() {
                if split >= max_width {
                    break;
                }
                if ch.is_whitespace() {
                    last_space = Some(idx);
                }
                split += 1;
            }

            let byte_split = last_space.unwrap_or_else(|| {
                remaining
                    .char_indices()
                    .nth(max_width)
                    .map(|(idx, _)| idx)
                    .unwrap_or(remaining.len())
            });
            lines.push(remaining[..byte_split].trim_end().to_string());
            remaining = remaining[byte_split..].trim_start();
        }

        lines.push(remaining.to_string());
    }

    lines
}

fn render_keybindings(ctx: &mut Frame, world: &World) {
    let width = (ctx.width() - 4).clamp(1, 80);
    let height = (ctx.height() - 4).clamp(1, 38);
    let x = ((ctx.width() - width) / 2).max(0);
    let y = ((ctx.height() - height) / 2).max(0);

    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " keybindings ");

    let inner_x = x + 2;
    let inner_w = (width - 4).max(1);
    let mut line_y = y + 2;

    print_section_header(ctx, inner_x, line_y, inner_w, "bindings");
    line_y += 1;

    if world.bindings.is_empty() {
        ctx.print_color(
            inner_x + 1,
            line_y,
            RGB::named(GRAY),
            RGB::named(BLACK),
            "(none — use the console to bind keys)",
        );
        line_y += 1;
    } else {
        let mut sorted: Vec<_> = world.bindings.iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (key, command) in &sorted {
            let is_new = world.new_binding_keys.contains(*key);
            let key_label = format!("[{}]", key);
            let fg = if is_new {
                RGB::named(CYAN)
            } else {
                RGB::named(WHITE)
            };
            print_clipped_color(ctx, inner_x + 1, line_y, 6, &key_label, fg);
            if is_new {
                print_clipped_color(ctx, inner_x + 7, line_y, 6, "[NEW]", RGB::named(CYAN));
                print_clipped(ctx, inner_x + 13, line_y, inner_w - 13, command);
            } else {
                print_clipped(ctx, inner_x + 7, line_y, inner_w - 7, command);
            }
            line_y += 1;
            if line_y > y + height - 3 {
                break;
            }
        }
    }

    line_y = (y + height - 2).min(line_y + 1);
    print_clipped(ctx, inner_x, line_y, inner_w, "tab/esc close");
}

fn render_memories(ctx: &mut Frame, world: &World) {
    let width = (ctx.width() - 4).clamp(1, 92);
    let height = (ctx.height() - 4).clamp(1, 38);
    let x = ((ctx.width() - width) / 2).max(0);
    let y = ((ctx.height() - height) / 2).max(0);

    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " memories ");

    let inner_x = x + 2;
    let inner_w = (width - 4).max(1);
    let content_rows = (height - 5).max(0) as usize;
    let collected: Vec<_> = world
        .fragment_registry
        .all()
        .iter()
        .filter(|fragment| fragment.status == crate::fragment::FragmentStatus::Collected)
        .collect();

    let mut lines: Vec<(String, RGB)> = Vec::new();
    if collected.is_empty() {
        lines.push((
            "(no recovered memories yet)".to_string(),
            RGB::named(DARK_GRAY),
        ));
    } else {
        for fragment in &collected {
            lines.push((
                format!("{}  weight {}", fragment.id, fragment.weight),
                RGB::named(CYAN),
            ));
            for wrapped in wrap_text(&fragment.text, inner_w as usize) {
                lines.push((format!("  {wrapped}"), RGB::named(WHITE)));
            }
            lines.push((String::new(), RGB::named(WHITE)));
        }
    }

    let max_scroll = lines.len().saturating_sub(content_rows);
    let scroll = world.memory_scroll.min(max_scroll);
    let mut line_y = y + 2;
    for (text, color) in lines.iter().skip(scroll).take(content_rows) {
        print_clipped_color(ctx, inner_x, line_y, inner_w, text, *color);
        line_y += 1;
    }

    let footer = format!(
        "j/k scroll  m/esc close  {}/{} collected",
        collected.len(),
        world.fragment_registry.findable_count()
    );
    print_clipped(ctx, inner_x, y + height - 2, inner_w, &footer);
}

fn render_death_screen(ctx: &mut Frame, world: &World) {
    let width = (ctx.width() - 4).clamp(1, 60);
    let height = (ctx.height() - 4).clamp(1, 18);
    let x = ((ctx.width() - width) / 2).max(0);
    let y = ((ctx.height() - height) / 2).max(0);

    fill_rect(ctx, x, y, width, height, RGB::named(BLACK));
    draw_box(ctx, x, y, width, height, " death ");

    let inner_x = x + 2;
    let inner_w = (width - 4).max(1);
    let mut line_y = y + 2;

    ctx.print_color(
        inner_x,
        line_y,
        RGB::named(RED),
        RGB::named(BLACK),
        "YOU HAVE PERISHED",
    );
    line_y += 2;

    print_clipped(
        ctx,
        inner_x,
        line_y,
        inner_w,
        &format!("Depth: {}  |  Turn: {}", world.depth, world.turn),
    );
    line_y += 2;

    print_clipped(
        ctx,
        inner_x,
        line_y,
        inner_w,
        "[r]       Respawn at this depth",
    );
    line_y += 1;
    print_clipped(
        ctx,
        inner_x,
        line_y,
        inner_w,
        "[shift+r] Restart from depth 1",
    );
    line_y += 1;
    print_clipped(ctx, inner_x, line_y, inner_w, "[esc/q]   Quit");
}

fn draw_box(ctx: &mut Frame, x: i32, y: i32, width: i32, height: i32, title: &str) {
    if width < 2 || height < 2 {
        return;
    }

    let fg = RGB::named(WHITE);
    let bg = RGB::named(BLACK);

    for dx in 0..width {
        ctx.set(x + dx, y, fg, bg, '-');
        ctx.set(x + dx, y + height - 1, fg, bg, '-');
    }

    for dy in 0..height {
        ctx.set(x, y + dy, fg, bg, '|');
        ctx.set(x + width - 1, y + dy, fg, bg, '|');
    }

    ctx.set(x, y, fg, bg, '+');
    ctx.set(x + width - 1, y, fg, bg, '+');
    ctx.set(x, y + height - 1, fg, bg, '+');
    ctx.set(x + width - 1, y + height - 1, fg, bg, '+');
    print_clipped(ctx, x + 2, y, width - 4, title);
}

fn fill_rect(ctx: &mut Frame, x: i32, y: i32, width: i32, height: i32, color: RGB) {
    if width <= 0 || height <= 0 {
        return;
    }

    for dy in 0..height {
        for dx in 0..width {
            ctx.set(x + dx, y + dy, RGB::named(WHITE), color, ' ');
        }
    }
}

fn print_highlighted(ctx: &mut Frame, x: i32, y: i32, max_width: i32, spans: &[Span]) {
    let mut cx = x;
    for span in spans {
        let fg = span.color();
        for ch in span.text.chars() {
            if cx < x + max_width {
                ctx.set(cx, y, fg, RGB::named(BLACK), ch);
                cx += 1;
            }
        }
    }
}

fn print_clipped(ctx: &mut Frame, x: i32, y: i32, max_width: i32, text: &str) {
    if max_width <= 0 || y < 0 || y >= ctx.height() {
        return;
    }

    let clipped: String = text.chars().take(max_width as usize).collect();
    ctx.print_color(x, y, RGB::named(WHITE), RGB::named(BLACK), &clipped);
}

fn print_clipped_color(ctx: &mut Frame, x: i32, y: i32, max_width: i32, text: &str, color: RGB) {
    if max_width <= 0 || y < 0 || y >= ctx.height() {
        return;
    }

    let clipped: String = text.chars().take(max_width as usize).collect();
    ctx.print_color(x, y, color, RGB::named(BLACK), &clipped);
}

fn top_panel_height(ctx: &Frame) -> i32 {
    if event_log_uses_side_panel(ctx) {
        ctx.height().max(1)
    } else {
        (log_box_y(ctx) + 1).max(1)
    }
}

fn log_box_y(ctx: &Frame) -> i32 {
    if ctx.height() > MAP_BOX_HEIGHT {
        MAP_BOX_HEIGHT - 1
    } else {
        ctx.height().saturating_sub(1)
    }
}

fn log_box_height(ctx: &Frame) -> i32 {
    (ctx.height() - log_box_y(ctx)).max(0)
}

fn event_log_uses_side_panel(ctx: &Frame) -> bool {
    let panel_width = (ctx.width() - MAP_BOX_WIDTH).max(0);
    panel_width >= PANEL_MIN_WIDTH && bottom_log_height(ctx) < MIN_BOTTOM_LOG_HEIGHT
}

fn bottom_log_height(ctx: &Frame) -> i32 {
    if ctx.height() > MAP_BOX_HEIGHT {
        ctx.height() - (MAP_BOX_HEIGHT - 1)
    } else {
        0
    }
}

/// Maps a binding key to a clearer display string.
fn display_key(key: &str) -> &str {
    match key {
        ">" => "shift+.",
        "<" => "shift+,",
        k => k,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_header_wraps_to_two_lines_at_eighty() {
        let text = "Glyph REPL — press Ctrl+E to open an external editor for multi-line input. Try (help). ESC or ` to close.";
        let lines = wrap_text(text, 80);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].chars().count() <= 80);
        assert!(lines[1].chars().count() <= 80);
    }

    #[test]
    fn log_entries_wrap_into_visible_rows() {
        let color = RGB::named(CYAN);
        let entries = vec![LogEntry {
            text: "Open the console (`) and bind attack to a key".to_string(),
            color: Some(color),
        }];

        let lines = wrapped_log_lines(&entries, 18);
        let text = lines
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            text,
            vec!["Open the console", "(`) and bind", "attack to a key"]
        );
        assert!(lines.iter().all(|line| line.color == Some(color)));
    }

    #[test]
    fn console_uses_floating_bounds_on_large_terminals() {
        let frame = Frame::new(180, 60);

        let (x, y, width, height) = console_bounds(&frame);

        assert!(x > 0);
        assert!(y > 0);
        assert!(width < frame.width());
        assert!(height < frame.height());
        assert_eq!(width, 120);
        assert_eq!(height, 24);
    }

    #[test]
    fn console_bounds_fit_tiny_terminals() {
        let frame = Frame::new(7, 6);

        let (x, y, width, height) = console_bounds(&frame);

        assert_eq!((x, y), (0, 0));
        assert_eq!((width, height), (7, 6));
    }

    #[test]
    fn short_terminals_move_event_log_into_side_panel() {
        let short = Frame::new(MAP_BOX_WIDTH + PANEL_MIN_WIDTH, MAP_BOX_HEIGHT + 3);
        assert!(event_log_uses_side_panel(&short));
        assert_eq!(top_panel_height(&short), short.height());

        let tall = Frame::new(
            MAP_BOX_WIDTH + PANEL_MIN_WIDTH,
            MAP_BOX_HEIGHT + MIN_BOTTOM_LOG_HEIGHT,
        );
        assert!(!event_log_uses_side_panel(&tall));
    }

    #[test]
    fn narrow_terminals_keep_event_log_below_map() {
        let narrow = Frame::new(MAP_BOX_WIDTH + PANEL_MIN_WIDTH - 1, MAP_BOX_HEIGHT + 3);

        assert!(!event_log_uses_side_panel(&narrow));
    }

    #[test]
    fn key_summary_groups_duplicate_commands() {
        let mut world = World::minimal();
        world.bindings.insert("k".into(), "(move! :north)".into());
        world.bindings.insert("up".into(), "(move! :north)".into());
        world
            .bindings
            .insert("i".into(), "(toggle-inspector!)".into());
        world.new_rule_ids.insert("slime-hunt".into());

        let summaries = binding_summaries(&world);
        let north = summaries
            .iter()
            .find(|summary| summary.command == "(move! :north)")
            .expect("north movement binding should be summarized");
        assert_eq!(north.label, "move north");
        assert_eq!(north.keys, vec!["k", "up"]);

        let inspector = summaries
            .iter()
            .find(|summary| summary.command == "(toggle-inspector!)")
            .expect("inspector binding should be summarized");
        assert_eq!(inspector.label, "inspector");
        assert!(inspector.is_new);
    }

    #[test]
    fn diagnostic_lines_are_clipped_without_reflowing() {
        let report = "Error: syntax error\n   +-[glyph:1:17]\n 1 | (bind-key :z (do";

        assert!(is_diagnostic_output(report));
        assert_eq!(
            clipped_lines(report, 80),
            vec![
                "Error: syntax error",
                "   +-[glyph:1:17]",
                " 1 | (bind-key :z (do",
            ]
        );
    }

    #[test]
    fn help_output_uses_syntax_highlighting() {
        assert!(console_output_uses_syntax_highlighting(
            "=> Glyph help (page 5/6): language reference\n  (if test then else)"
        ));
        assert!(!console_output_uses_syntax_highlighting("=> (1 2 3)"));
    }
}

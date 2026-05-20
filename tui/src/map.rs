//! Static map and bracket-lib pathing implementation.
//!
//! The map owns terrain, walkability, pathfinding exits, and the v1 flashlight
//! ray caster. It does not know about entities beyond receiving a player
//! position/facing for visibility queries.

use std::collections::{HashSet, VecDeque};

use bracket_lib::prelude::{
    Algorithm2D, BaseMap, DistanceAlg, Point, RandomNumberGenerator, SmallVec,
};

use serde::{Deserialize, Serialize};

use crate::entity::{Direction, Position};

pub const MAP_WIDTH: i32 = 55;
pub const MAP_HEIGHT: i32 = 33;
pub const FLASHLIGHT_RADIUS: i32 = 12;
const FLASHLIGHT_SPREAD_DOT: f32 = 0.70;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Floor,
    Wall,
    StairsDown,
    StairsUp,
    Fire,
}

#[derive(Clone, Debug)]
pub struct Map {
    pub width: i32,
    pub height: i32,
    tiles: Vec<TileType>,
    /// Dijkstra distance field from player (cached, recomputed on tick).
    dijkstra: Vec<i32>,
    dijkstra_target: Position,
}

impl Map {
    /// Create a map with every tile set to the given type.
    pub fn new_filled(width: i32, height: i32, tile: TileType) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            tiles: vec![tile; size],
            dijkstra: vec![i32::MAX; size],
            dijkstra_target: Position::new(-1, -1),
        }
    }

    pub fn new_static() -> Self {
        let size = (MAP_WIDTH * MAP_HEIGHT) as usize;
        let mut map = Self {
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            tiles: vec![TileType::Floor; size],
            dijkstra: vec![i32::MAX; size],
            dijkstra_target: Position::new(-1, -1),
        };

        for x in 0..MAP_WIDTH {
            map.set_tile(Position::new(x, 0), TileType::Wall);
            map.set_tile(Position::new(x, MAP_HEIGHT - 1), TileType::Wall);
        }

        for y in 0..MAP_HEIGHT {
            map.set_tile(Position::new(0, y), TileType::Wall);
            map.set_tile(Position::new(MAP_WIDTH - 1, y), TileType::Wall);
        }

        for x in 8..44 {
            if x != 23 && x != 24 {
                map.set_tile(Position::new(x, 8), TileType::Wall);
            }
        }

        for y in 13..25 {
            if y != 18 {
                map.set_tile(Position::new(31, y), TileType::Wall);
            }
        }

        for x in 34..50 {
            if x != 42 {
                map.set_tile(Position::new(x, 21), TileType::Wall);
            }
        }

        map
    }

    /// Generate a random room-based dungeon with depth-scaled difficulty.
    /// Uses region-based placement, Kruskal MST corridors, and room typing.
    pub fn generate(width: i32, height: i32, depth: u32) -> MapGenOutput {
        let mut map = Self {
            width,
            height,
            tiles: vec![TileType::Wall; (width * height) as usize],
            dijkstra: vec![i32::MAX; (width * height) as usize],
            dijkstra_target: Position::new(-1, -1),
        };

        let mut rng = RandomNumberGenerator::new();
        let mut rooms: Vec<Room> = Vec::new();

        // --- Region-based room placement ---
        let cols = ((width - 4) / 11).clamp(2, 5);
        let rows = ((height - 4) / 10).clamp(2, 4);
        let region_w = (width - 2) / cols;
        let region_h = (height - 2) / rows;

        for ry in 0..rows {
            for rx in 0..cols {
                let region_x = 1 + rx * region_w;
                let region_y = 1 + ry * region_h;

                for _ in 0..20 {
                    let w = rng.range(5, 11);
                    let h = rng.range(5, 11);
                    let x = rng.range(region_x, (region_x + region_w - w).max(region_x + 1));
                    let y = rng.range(region_y, (region_y + region_h - h).max(region_y + 1));
                    let room = Room {
                        x,
                        y,
                        w,
                        h,
                        kind: RoomType::Combat,
                    };

                    if map.contains(Position::new(x + w, y + h))
                        && rooms.iter().all(|other| !room.overlaps(other))
                    {
                        map.carve_room(&room);
                        rooms.push(room);
                        break;
                    }
                }
            }
        }

        if rooms.is_empty() {
            // Fallback: single room in the center
            let room = Room {
                x: width / 2 - 3,
                y: height / 2 - 3,
                w: 6,
                h: 6,
                kind: RoomType::Entrance,
            };
            map.carve_room(&room);
            rooms.push(room);
        }

        // --- Build complete graph of room centers ---
        let n = rooms.len();
        let centers: Vec<Position> = rooms.iter().map(|r| r.center()).collect();
        let mut edges: Vec<MstEdge> = Vec::with_capacity(n * (n - 1) / 2);
        for i in 0..n {
            for j in (i + 1)..n {
                edges.push(MstEdge {
                    from: i,
                    to: j,
                    weight: centers[i].manhattan_distance(centers[j]),
                });
            }
        }
        edges.sort_by_key(|e| e.weight);

        // --- Kruskal's MST ---
        let mut uf = UnionFind::new(n);
        let mut mst_edges: Vec<usize> = Vec::with_capacity(n - 1);
        let mut remaining: Vec<usize> = Vec::new();
        for (idx, edge) in edges.iter().enumerate() {
            if uf.union(edge.from, edge.to) {
                mst_edges.push(idx);
            } else {
                remaining.push(idx);
            }
        }

        // --- Add ~15% extra loops for tactical interest ---
        let extra_count = ((remaining.len() as f32) * 0.15).ceil() as usize;
        // Shuffle remaining by swapping each with a random index
        for i in 0..remaining.len() {
            let j = rng.range(0, remaining.len() as i32) as usize;
            remaining.swap(i, j);
        }
        let selected_edges: Vec<usize> = mst_edges
            .iter()
            .copied()
            .chain(remaining.into_iter().take(extra_count))
            .collect();

        // --- Carve corridors ---
        for idx in &selected_edges {
            let edge = &edges[*idx];
            map.carve_corridor(centers[edge.from], centers[edge.to]);
        }

        // --- Assign room types ---
        // rooms[0] = Entrance; farthest by BFS over MST = Exit
        rooms[0].kind = RoomType::Entrance;

        // Build adjacency from MST edges for BFS
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for idx in &mst_edges {
            let e = &edges[*idx];
            adj[e.from].push(e.to);
            adj[e.to].push(e.from);
        }

        let farthest = bfs_farthest(0, &adj);
        if farthest != 0 {
            rooms[farthest].kind = RoomType::Exit;
        } else if n > 1 {
            rooms[n - 1].kind = RoomType::Exit;
        }

        // Distribute remaining room types by depth
        let (boss_pct, treasure_pct) = match depth {
            1 => (0, 30),
            2..=3 => (15, 30),
            _ => (25, 35),
        };

        for i in 1..n {
            if i == farthest {
                continue;
            }
            let roll = rng.range(0, 100);
            if roll < boss_pct && boss_pct > 0 {
                rooms[i].kind = RoomType::Boss;
            } else if roll < boss_pct + treasure_pct {
                rooms[i].kind = RoomType::Treasure;
            } else {
                rooms[i].kind = RoomType::Combat;
            }
        }

        // --- Build output ---
        let player_start = centers[0];
        let stairs_up = player_start;
        let stairs_down = if farthest != 0 {
            centers[farthest]
        } else {
            player_start
        };

        // Ensure stairs tiles are set (rooms are already carved, just overlay)
        map.set_tile(stairs_up, TileType::StairsUp);
        map.set_tile(stairs_down, TileType::StairsDown);

        let mut combat_spawns: Vec<Position> = Vec::new();
        let mut boss_spawns: Vec<Position> = Vec::new();
        for room in &rooms {
            match room.kind {
                RoomType::Combat | RoomType::Treasure => {
                    combat_spawns.push(room.center());
                }
                RoomType::Boss => {
                    boss_spawns.push(room.center());
                }
                _ => {}
            }
        }

        MapGenOutput {
            map,
            player_start,
            stairs_up,
            stairs_down,
            combat_spawns,
            boss_spawns,
        }
    }

    /// Generate a cellular automata cave with depth-scaled enemies.
    pub fn generate_cave(depth: u32) -> MapGenOutput {
        let width = MAP_WIDTH;
        let height = MAP_HEIGHT;
        let mut map = Self {
            width,
            height,
            tiles: vec![TileType::Wall; (width * height) as usize],
            dijkstra: vec![i32::MAX; (width * height) as usize],
            dijkstra_target: Position::new(-1, -1),
        };

        let mut rng = RandomNumberGenerator::new();

        // 1. Random noise: 45% floor, borders always wall
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let floor = rng.range(0, 100) < 45;
                if floor {
                    map.set_tile(Position::new(x, y), TileType::Floor);
                }
            }
        }

        // 2. Cellular automata smoothing (4 iterations)
        for _ in 0..4 {
            let mut next = map.tiles.clone();
            for y in 1..height - 1 {
                for x in 1..width - 1 {
                    let idx = map.idx(Position::new(x, y));
                    let wall_count = map.count_wall_neighbors(x, y);
                    if map.tiles[idx] == TileType::Wall {
                        if wall_count < 4 {
                            next[idx] = TileType::Floor;
                        }
                    } else if wall_count >= 5 {
                        next[idx] = TileType::Wall;
                    }
                }
            }
            map.tiles = next;
        }

        // 3. Flood-fill: keep only the largest connected floor region
        let connected = map.largest_floor_region();
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let pos = Position::new(x, y);
                if map.tile(pos) == TileType::Floor && !connected.contains(&pos) {
                    map.set_tile(pos, TileType::Wall);
                }
            }
        }

        // 4. Place entrance near center, exit at farthest reachable point
        let center = Position::new(width / 2, height / 2);
        let player_start = map
            .find_nearest_floor(center)
            .unwrap_or(Position::new(2, 2));
        let stairs_up = player_start;
        let stairs_down = map.find_farthest_floor(player_start, &connected);

        map.set_tile(stairs_up, TileType::StairsUp);
        map.set_tile(stairs_down, TileType::StairsDown);

        // 5. Scatter combat spawns at minimum distance from player and each other
        let spawn_count = 1 + depth as usize;
        let mut combat_spawns: Vec<Position> = Vec::new();
        let mut candidates: Vec<Position> = connected
            .iter()
            .copied()
            .filter(|p| p.manhattan_distance(player_start) >= 8)
            .collect();

        // Shuffle candidates
        for i in 0..candidates.len() {
            let j = rng.range(0, candidates.len() as i32) as usize;
            candidates.swap(i, j);
        }

        for &pos in &candidates {
            if combat_spawns.len() >= spawn_count {
                break;
            }
            if combat_spawns.iter().all(|s| s.manhattan_distance(pos) >= 5) {
                combat_spawns.push(pos);
            }
        }

        MapGenOutput {
            map,
            player_start,
            stairs_up,
            stairs_down,
            combat_spawns,
            boss_spawns: Vec::new(),
        }
    }

    fn count_wall_neighbors(&self, x: i32, y: i32) -> usize {
        let mut count = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let pos = Position::new(x + dx, y + dy);
                if !self.contains(pos) || self.tile(pos) == TileType::Wall {
                    count += 1;
                }
            }
        }
        count
    }

    fn largest_floor_region(&self) -> HashSet<Position> {
        let mut visited: HashSet<Position> = HashSet::new();
        let mut largest: HashSet<Position> = HashSet::new();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let start = Position::new(x, y);
                if self.tile(start) != TileType::Floor || visited.contains(&start) {
                    continue;
                }

                let mut region = HashSet::new();
                let mut queue = VecDeque::new();
                queue.push_back(start);
                visited.insert(start);

                while let Some(pos) = queue.pop_front() {
                    region.insert(pos);
                    for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                        let neighbor = Position::new(pos.x + dx, pos.y + dy);
                        if self.tile(neighbor) == TileType::Floor && !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }

                if region.len() > largest.len() {
                    largest = region;
                }
            }
        }

        largest
    }

    fn find_nearest_floor(&self, target: Position) -> Option<Position> {
        let mut queue = VecDeque::new();
        let mut visited: HashSet<Position> = HashSet::new();
        queue.push_back(target);
        visited.insert(target);

        while let Some(pos) = queue.pop_front() {
            if self.contains(pos) && self.tile(pos) == TileType::Floor {
                return Some(pos);
            }
            for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let neighbor = Position::new(pos.x + dx, pos.y + dy);
                if self.contains(neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
        None
    }

    fn find_farthest_floor(&self, start: Position, floor_set: &HashSet<Position>) -> Position {
        let mut queue = VecDeque::new();
        let mut visited: HashSet<Position> = HashSet::new();
        let mut farthest = start;
        queue.push_back(start);
        visited.insert(start);

        while let Some(pos) = queue.pop_front() {
            farthest = pos;
            for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let neighbor = Position::new(pos.x + dx, pos.y + dy);
                if floor_set.contains(&neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        farthest
    }

    fn carve_room(&mut self, room: &Room) {
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                self.set_tile(Position::new(x, y), TileType::Floor);
            }
        }
    }

    fn carve_corridor(&mut self, from: Position, to: Position) {
        let mut x = from.x;
        let mut y = from.y;
        while x != to.x {
            self.set_tile(Position::new(x, y), TileType::Floor);
            x += if to.x > x { 1 } else { -1 };
        }
        while y != to.y {
            self.set_tile(Position::new(x, y), TileType::Floor);
            y += if to.y > y { 1 } else { -1 };
        }
    }

    pub fn idx(&self, pos: Position) -> usize {
        (pos.y * self.width + pos.x) as usize
    }

    pub fn point_for_idx(&self, idx: usize) -> Point {
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        Point::new(x, y)
    }

    pub fn position_for_idx(&self, idx: usize) -> Position {
        Position::from_point(self.point_for_idx(idx))
    }

    pub fn contains(&self, pos: Position) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }

    pub fn tile(&self, pos: Position) -> TileType {
        self.tiles[self.idx(pos)]
    }

    pub fn is_walkable(&self, pos: Position) -> bool {
        self.contains(pos) && self.tile(pos) != TileType::Wall
    }

    pub fn flashlight_tiles(&self, origin: Position, facing: Direction) -> HashSet<Position> {
        let mut lit = HashSet::new();
        if !self.contains(origin) {
            return lit;
        }

        lit.insert(origin);
        for target in self.flashlight_targets(origin, facing) {
            for pos in self.ray_until_blocked(origin, target) {
                lit.insert(pos);
            }
        }
        lit
    }

    /// Compute Dijkstra distance field from `target` over walkable tiles.
    /// Cached: no-op if called again with same target on same tick.
    pub fn compute_dijkstra(&mut self, target: Position) {
        if target == self.dijkstra_target {
            return;
        }
        self.dijkstra_target = target;
        self.dijkstra.fill(i32::MAX);

        if !self.contains(target) {
            return;
        }

        let target_idx = self.idx(target);
        self.dijkstra[target_idx] = 0;

        let mut queue = VecDeque::new();
        queue.push_back(target);

        while let Some(pos) = queue.pop_front() {
            let idx = self.idx(pos);
            let dist = self.dijkstra[idx] + 1;
            for (dx, dy) in &[(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
                let neighbor = Position::new(pos.x + dx, pos.y + dy);
                if self.is_walkable(neighbor) {
                    let nidx = self.idx(neighbor);
                    if self.dijkstra[nidx] > dist {
                        self.dijkstra[nidx] = dist;
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    /// Best neighbor step toward the current Dijkstra target. Returns `None`
    /// when unreachable (dijkstra value is i32::MAX) or already at target.
    pub fn dijkstra_best_step(&self, from: Position) -> Option<Position> {
        if !self.contains(from) {
            return None;
        }
        let current_dist = self.dijkstra[self.idx(from)];
        if current_dist == i32::MAX {
            return None;
        }
        let mut best: Option<(Position, i32)> = None;
        for (dx, dy) in &[(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
            let neighbor = Position::new(from.x + dx, from.y + dy);
            if !self.contains(neighbor) {
                continue;
            }
            let nd = self.dijkstra[self.idx(neighbor)];
            if nd < current_dist {
                match best {
                    None => best = Some((neighbor, nd)),
                    Some((_, bd)) if nd < bd => best = Some((neighbor, nd)),
                    _ => {}
                }
            }
        }
        best.map(|(pos, _)| pos)
    }

    pub(crate) fn set_tile(&mut self, pos: Position, tile: TileType) {
        if self.contains(pos) {
            let idx = self.idx(pos);
            self.tiles[idx] = tile;
        }
    }

    fn maybe_exit(&self, exits: &mut SmallVec<[(usize, f32); 10]>, pos: Position) {
        if self.is_walkable(pos) {
            exits.push((self.idx(pos), 1.0));
        }
    }

    fn flashlight_targets(&self, origin: Position, facing: Direction) -> Vec<Position> {
        let (fx, fy) = facing.delta();
        let facing_len = ((fx * fx + fy * fy) as f32).sqrt();
        let mut targets = Vec::new();

        for y in (origin.y - FLASHLIGHT_RADIUS)..=(origin.y + FLASHLIGHT_RADIUS) {
            for x in (origin.x - FLASHLIGHT_RADIUS)..=(origin.x + FLASHLIGHT_RADIUS) {
                let pos = Position::new(x, y);
                if !self.contains(pos) || pos == origin {
                    continue;
                }

                let dx = x - origin.x;
                let dy = y - origin.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq > FLASHLIGHT_RADIUS * FLASHLIGHT_RADIUS {
                    continue;
                }

                let dist = (dist_sq as f32).sqrt();
                let dot = ((dx * fx + dy * fy) as f32) / (dist * facing_len);
                if dot >= FLASHLIGHT_SPREAD_DOT {
                    targets.push(pos);
                }
            }
        }

        targets
    }

    fn ray_until_blocked(&self, origin: Position, target: Position) -> Vec<Position> {
        let mut ray = Vec::new();
        for pos in bresenham_line(origin, target).into_iter().skip(1) {
            if !self.contains(pos) {
                break;
            }

            ray.push(pos);
            if self.tile(pos) == TileType::Wall {
                break;
            }
        }
        ray
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let pos = self.position_for_idx(idx);
        let mut exits = SmallVec::new();
        self.maybe_exit(&mut exits, pos.offset(-1, 0));
        self.maybe_exit(&mut exits, pos.offset(1, 0));
        self.maybe_exit(&mut exits, pos.offset(0, -1));
        self.maybe_exit(&mut exits, pos.offset(0, 1));
        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Manhattan.distance2d(self.point_for_idx(idx1), self.point_for_idx(idx2))
    }
}

fn bresenham_line(start: Position, end: Position) -> Vec<Position> {
    let mut points = Vec::new();
    let mut x = start.x;
    let mut y = start.y;
    let dx = (end.x - start.x).abs();
    let dy = -(end.y - start.y).abs();
    let sx = if start.x < end.x { 1 } else { -1 };
    let sy = if start.y < end.y { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push(Position::new(x, y));
        if x == end.x && y == end.y {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }

    points
}

#[derive(Clone, Copy, Debug)]
struct MstEdge {
    from: usize,
    to: usize,
    weight: i32,
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        let mut parent = Vec::with_capacity(n);
        for i in 0..n {
            parent.push(i);
        }
        Self {
            parent,
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) -> bool {
        let xr = self.find(x);
        let yr = self.find(y);
        if xr == yr {
            return false;
        }
        if self.rank[xr] < self.rank[yr] {
            self.parent[xr] = yr;
        } else if self.rank[xr] > self.rank[yr] {
            self.parent[yr] = xr;
        } else {
            self.parent[yr] = xr;
            self.rank[xr] += 1;
        }
        true
    }
}

fn bfs_farthest(start: usize, adj: &[Vec<usize>]) -> usize {
    let n = adj.len();
    let mut dist = vec![0i32; n];
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();
    visited[start] = true;
    queue.push_back(start);

    while let Some(u) = queue.pop_front() {
        for &v in &adj[u] {
            if !visited[v] {
                visited[v] = true;
                dist[v] = dist[u] + 1;
                queue.push_back(v);
            }
        }
    }

    dist.iter()
        .enumerate()
        .max_by_key(|(_, d)| **d)
        .map(|(i, _)| i)
        .unwrap_or(start)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RoomType {
    Entrance,
    Exit,
    Combat,
    Treasure,
    Boss,
}

pub struct MapGenOutput {
    pub map: Map,
    pub player_start: Position,
    pub stairs_up: Position,
    pub stairs_down: Position,
    pub combat_spawns: Vec<Position>,
    pub boss_spawns: Vec<Position>,
}

struct Room {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    kind: RoomType,
}

impl Room {
    fn center(&self) -> Position {
        Position::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    fn overlaps(&self, other: &Room) -> bool {
        self.x - 1 < other.x + other.w
            && self.x + self.w + 1 > other.x
            && self.y - 1 < other.y + other.h
            && self.y + self.h + 1 > other.y
    }
}

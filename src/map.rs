//! Terrain sampling and drawing into a palette-indexed pixel grid.
//!
//! The grid is a little larger than the map itself: the extra cells hold the
//! beveled frame and the N/E/S/W compass letters, like Xaero's minimap.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use weft_sdk::{Client, abi::BlockPos};

use crate::icons::{self, ICON_SIZE};
use crate::palette::{
    self, B_GRASS, B_LEAVES_LIGHT, B_SNOW, B_WATER, P_ARROW, P_FRAME_DARK, P_FRAME_LIGHT,
    P_FRAME_OUTER, P_HOSTILE, P_ITEM, P_PASSIVE, P_SPR_DARK, P_TRANSPARENT,
    P_UNDISCOVERED, P_WHITE, SHADE_DARK, SHADE_LIGHT, SHADE_NORMAL,
};
use crate::{settings, state, waypoints};

static COLOR_CACHE: LazyLock<RwLock<HashMap<u32, Option<u8>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn color_of(state_id: u32) -> Option<u8> {
    if state_id == 0 {
        return None;
    }
    if let Some(c) = COLOR_CACHE.read().unwrap().get(&state_id) {
        return *c;
    }
    let c = palette::surface_base(state_id);
    COLOR_CACHE.write().unwrap().insert(state_id, c);
    c
}

/// Finds the highest terrain block in a column. A coarse scan with a binary
/// refinement keeps this cheap even when the player flies hundreds of blocks
/// above the ground.
fn top_block(c: &Client, x: i32, z: i32, ceiling: i32, floor: i32, step: i32) -> Option<(i32, u8)> {
    let mut y = ceiling;
    while y >= floor {
        match c.block(BlockPos::new(x, y, z)) {
            Some(st) => {
                if let Some(base) = color_of(st) {
                    // Walk up to the top of this run of solid blocks.
                    let mut found = y;
                    let mut top = (y + step).min(ceiling);
                    while found + 1 <= top {
                        let mid = (found + top + 1) / 2;
                        if c.block(BlockPos::new(x, mid, z)).is_some_and(|s| color_of(s).is_some()) {
                            found = mid;
                        } else {
                            top = mid - 1;
                        }
                    }
                    let base = color_of(c.block(BlockPos::new(x, found, z))?).unwrap_or(base);
                    return Some((found, base));
                }
                y -= step;
            }
            None => y = (y & !15) - 1,
        }
    }
    None
}

struct Grid {
    w: usize,
    h: usize,
    px: Vec<u8>,
    height: Vec<i32>,
    base: Vec<u8>,
    wx: Vec<i32>,
    wz: Vec<i32>,
    mask: Vec<bool>,
    x0: usize,
    y0: usize,
    iw: usize,
    ih: usize,
}

impl Grid {
    fn new(w: usize, h: usize, x0: usize, y0: usize, iw: usize, ih: usize) -> Self {
        let n = w * h;
        Grid {
            w, h,
            px: vec![P_TRANSPARENT; n],
            height: vec![i32::MIN; n],
            base: vec![0; n],
            wx: vec![0; n],
            wz: vec![0; n],
            mask: vec![false; n],
            x0, y0, iw, ih,
        }
    }

    fn index(&self, x: isize, y: isize) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.w as isize || y >= self.h as isize {
            None
        } else {
            Some(y as usize * self.w + x as usize)
        }
    }

    fn set(&mut self, x: isize, y: isize, color: u8) {
        if let Some(i) = self.index(x, y) {
            if self.mask[i] {
                self.px[i] = color;
            }
        }
    }

    fn set_raw(&mut self, x: isize, y: isize, color: u8) {
        if let Some(i) = self.index(x, y) {
            self.px[i] = color;
        }
    }
}

fn sample_terrain(
    g: &mut Grid,
    c: &Client,
    cx: f64,
    cz: f64,
    py: i32,
    zoom: f64,
    facing: f64,
    depth: i32,
    cave: bool,
    step: i32,
) {
    let (sin, cos) = facing.sin_cos();
    let hw = g.iw as f64 / 2.0;
    let hh = g.ih as f64 / 2.0;
    let (ceiling, floor) = if cave {
        (py, py - depth)
    } else {
        // Look well above and below the player so the surface is still found
        // when flying high in creative.
        (py + 64, py - 160)
    };
    for row in 0..g.ih {
        for col in 0..g.iw {
            let i = (g.y0 + row) * g.w + g.x0 + col;
            let u = (col as f64 + 0.5 - hw) * zoom;
            let v = (row as f64 + 0.5 - hh) * zoom;
            let wx = (cx + u * cos - v * sin).floor() as i32;
            let wz = (cz + u * sin + v * cos).floor() as i32;
            g.wx[i] = wx;
            g.wz[i] = wz;
            g.mask[i] = true;
            if let Some((y, base)) = top_block(c, wx, wz, ceiling, floor, step) {
                g.height[i] = y;
                g.base[i] = base;
            }
        }
    }
}

/// 0.35 at night, 1.0 in daytime, ramping at dawn and dusk.
fn daylight(c: &Client) -> f32 {
    let t = (c.day_time() % 24000) as f32;
    if t < 6000.0 {
        0.35 + 0.65 * (t / 6000.0)
    } else if t < 12000.0 {
        1.0
    } else if t < 18000.0 {
        1.0 - 0.65 * ((t - 12000.0) / 6000.0)
    } else {
        0.35
    }
}

fn shade_terrain(g: &mut Grid, circle: bool, light: f32) {
    let cx = (g.x0 + g.iw / 2) as f64;
    let cy = (g.y0 + g.ih / 2) as f64;
    let radius = (g.iw.min(g.ih) / 2) as f64 - 1.0;
    for row in 0..g.ih {
        for col in 0..g.iw {
            let x = g.x0 + col;
            let y = g.y0 + row;
            let i = y * g.w + x;
            if circle {
                let d = ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt();
                if d > radius {
                    g.mask[i] = false;
                    continue;
                }
            }
            if g.height[i] == i32::MIN {
                g.px[i] = P_UNDISCOVERED;
                continue;
            }
            let mut shade = if g.base[i] == B_WATER {
                SHADE_NORMAL
            } else {
                // Compare with the neighbours up and left: a cell lower than
                // its neighbour is in shadow, a higher one catches the light.
                let h = g.height[i];
                let higher = (col > 0 && g.height[i - 1] > h)
                    || (row > 0 && g.height[i - g.w] > h);
                let lower = (col + 1 < g.iw && g.height[i + 1] < h)
                    || (row + 1 < g.ih && g.height[i + g.w] < h);
                if higher {
                    SHADE_DARK
                } else if lower {
                    SHADE_LIGHT
                } else {
                    SHADE_NORMAL
                }
            };
            // A little texture on grass and leaves, without making confetti.
            if (B_GRASS..=B_LEAVES_LIGHT).contains(&g.base[i]) || g.base[i] == B_SNOW {
                let h = (g.wx[i] as u32).wrapping_mul(73856093)
                    ^ (g.wz[i] as u32).wrapping_mul(19349663);
                if h % 5 == 0 && shade < SHADE_LIGHT {
                    shade += 1;
                }
            }
            // Daylight picks the shade: night uses the darkest of the four.
            let shade = (((shade as f32 + 0.5) * light).round() as i32).clamp(0, 3) as u8;
            g.px[i] = g.base[i] * 4 + shade;
        }
    }
}

fn draw_frame(g: &mut Grid, circle: bool) {
    if circle {
        let cx = (g.x0 + g.iw / 2) as f64;
        let cy = (g.y0 + g.ih / 2) as f64;
        let r = (g.iw.min(g.ih) / 2) as f64 - 1.0;
        for y in 0..g.h {
            for x in 0..g.w {
                let d = ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt();
                // Only the rings OUTSIDE the map get the frame; painting the
                // inside would hide all the terrain under a flat gray color.
                let c = if d > r && d <= r + 1.0 {
                    P_FRAME_DARK
                } else if d > r + 1.0 && d <= r + 2.0 {
                    P_FRAME_LIGHT
                } else if d > r + 2.0 && d <= r + 3.0 {
                    P_FRAME_OUTER
                } else {
                    continue;
                };
                g.px[y * g.w + x] = c;
            }
        }
    } else {
        let rings = [
            (1isize, P_FRAME_DARK),
            (2isize, P_FRAME_LIGHT),
            (3isize, P_FRAME_OUTER),
        ];
        for (n, color) in rings {
            let l = g.x0 as isize - n;
            let r = (g.x0 + g.iw) as isize - 1 + n;
            let t = g.y0 as isize - n;
            let b = (g.y0 + g.ih) as isize - 1 + n;
            for x in l..=r {
                g.set_raw(x, t, color);
                g.set_raw(x, b, color);
            }
            for y in t..=b {
                g.set_raw(l, y, color);
                g.set_raw(r, y, color);
            }
        }
    }
}

fn chunk_lines(g: &mut Grid) {
    if !settings::SHOW_GRID.get() {
        return;
    }
    for i in 0..g.px.len() {
        if g.mask[i] && (g.wx[i].rem_euclid(16) == 0 || g.wz[i].rem_euclid(16) == 0) {
            g.px[i] = P_FRAME_DARK;
        }
    }
}

fn project(g: &Grid, cx: f64, cz: f64, facing: f64, wx: i32, wz: i32) -> Option<(isize, isize)> {
    let (sin, cos) = facing.sin_cos();
    let dx = wx as f64 + 0.5 - cx;
    let dz = wz as f64 + 0.5 - cz;
    let u = dx * cos + dz * sin;
    let v = -dx * sin + dz * cos;
    let zoom = if g.iw == state::INNER { settings::ZOOM.get() } else { settings::BIG_ZOOM.get() };
    let col = (u / zoom + g.iw as f64 / 2.0).round() as isize + g.x0 as isize;
    let row = (v / zoom + g.ih as f64 / 2.0).round() as isize + g.y0 as isize;
    Some((col, row))
}

fn sprite_color(ch: char) -> Option<u8> {
    use crate::palette::{
        P_SPR_BROWN, P_SPR_BROWN_DARK, P_SPR_BRICK, P_SPR_BRICK_DARK, P_SPR_DARK,
        P_SPR_EYE_RED, P_SPR_EYE_RED_DARK, P_SPR_GRAY, P_SPR_GRAY_DARK, P_SPR_GREEN,
        P_SPR_GREEN_DARK, P_SPR_GREEN_LIGHT, P_SPR_LIGHT_GRAY, P_SPR_ORANGE, P_SPR_PINK,
        P_SPR_PINK_DARK, P_SPR_SNOUT, P_SPR_SNOUT_DARK, P_SPR_TAN, P_SPR_TAN_DARK,
        P_SPR_TAN_LIGHT, P_SPR_WHITE, P_SPR_YELLOW,
    };
    Some(match ch {
        'd' | 'k' => P_SPR_DARK,
        'w' => P_SPR_WHITE,
        'g' => P_SPR_GREEN,
        'G' => P_SPR_GREEN_DARK,
        'i' => P_SPR_GREEN_LIGHT,
        'p' => P_SPR_PINK,
        'P' => P_SPR_PINK_DARK,
        'b' => P_SPR_BROWN,
        'B' => P_SPR_BROWN_DARK,
        't' => P_SPR_TAN,
        'T' => P_SPR_TAN_DARK,
        'h' => P_SPR_TAN_LIGHT,
        's' => P_SPR_SNOUT,
        'S' => P_SPR_SNOUT_DARK,
        'm' => P_SPR_BRICK,
        'M' => P_SPR_BRICK_DARK,
        'r' => P_SPR_EYE_RED,
        'R' => P_SPR_EYE_RED_DARK,
        'o' => P_SPR_ORANGE,
        'y' => P_SPR_YELLOW,
        'a' => P_SPR_LIGHT_GRAY,
        'A' => P_SPR_GRAY,
        'v' => P_SPR_GRAY_DARK,
        'e' => palette::P_ARROW_DARK,
        'x' => P_ARROW,
        _ => return None,
    })
}

/// 12x12 pixel-art faces, colored to match the mob portraits used by Xaero's radar.
const SPRITES: &[(&str, &[&str])] = &[
    ("creeper", &[
        "..GGiiiiGG..", ".GiiiiiiiiG.", "GiiiiiiiiiiG", "GiiigggiiiiG",
        "GiiigggiiiiG", "GiiiiiiiiiiG", "GikiiiiikiiG", "GikiiiiikiiG",
        "GiiiiiiiiiiG", ".GikkkkkkiG.", "..GGkkkkGG..", "...GGGGGG...",
    ]),
    ("zombie", &[
        "..GGiiiiGG..", ".GiiiiiiiiG.", "GiiiiiiiiiiG", "GiiiiiiiiiiG",
        "GiiigggiiiiG", "GiiigggiiiiG", "GiiiigiiiiiG", "GikiiiiikiiG",
        "GikiiiiikiiG", "GiiiiiiiiiiG", ".GikkkkkkiG.", "..GGkkkkGG..",
    ]),
    ("skeleton", &[
        "..aaaaaaaa..", ".aaaaaaaaaa.", "aaaaaaaaaaaa", "aaAAaaaaAAaa",
        "aaAAaaaaAAaa", "aaaaaaaaaaaa", "aaaAAAAAAaaa", "aaaAvvvvAaaa",
        "aaaAAAAAAaaa", "aaaaaaaaaaaa", ".aaAAAAAAaa.", "..aaAAAAaa..",
    ]),
    ("spider", &[
        "mmmmmmmmmmmm", "mMMmmmmmmMMm", "mmMMmmmmMMmm", "mmmmmmmmmmmm",
        "mmrmmmmmrmmm", "mmRrmmmRrmmm", "mmmmmmmmmmmm", "mmrmmmmmrmmm",
        "mmRrmmmRrmmm", "mmmMMMMMMmmm", "mMMmmmmmmMMm", "mmmmmmmmmmmm",
    ]),
    ("pig", &[
        ".pppppppppp.", "pppppppppppp", "pppppppppppp", "pkppppppppkp",
        "pwppppppppwp", "pppppppppppp", "pppPPPPPPppp", "ppPssssssPpp",
        "ppPssssssPpp", "pppPSSSSPppp", "pppppppppppp", ".pppppppppp.",
    ]),
    ("sheep", &[
        ".hhhhhhhhhh.", "hhhthhhhthhh", "hhttthhttthh", "hkwttttttwkh",
        "hkwttttttwkh", "hhttthhttthh", "hhhthhhhthhh", "hhhhhhhhhhhh",
        "hhhhtttthhhh", "hhhssssshhhh", "hhhSSSSShhhh", "..hhhhhhhh..",
    ]),
    ("cow", &[
        "..BBBBBBBB..", ".BbwwwwwwbB.", "BbwwwwwwwwbB", "BwkwAwwAkwkB",
        "BwkwAwwAkwkB", "BwwwAAAAwwwB", "BwwsssssswwB", "BwwsssssswwB",
        "BwwwSSSSwwwB", ".BwwwwwwwwB.", "..BBBBBBBB..", "...BBBBBB...",
    ]),
    ("chicken", &[
        "..wwwwwwww..", ".wwwwwwwwww.", "wwwyywwyywww", "wwwyywwyywww",
        "wwwwwwwwwwww", "wwwwwwwwwwww", "wwwyywwyywww", "wwwwwwwwwwww",
        "..wwwwwwww..", "...wwwwww...", "....oooo....", "...oooooo...",
    ]),
    ("player", &[
        "..bbbbbbbb..", ".bbttttttbb.", "bbtthhtthhbb", "bkwttttttwkb",
        "bkwttttttwkb", "bbtthhtthhbb", ".bbttttttbb.", "..bbbbbbbb..",
        "...bTTTTb...", "...bTTTTb...", "..bbTTTTbb..", "..bbbbbbbb..",
    ]),
];

fn stamp(g: &mut Grid, cx: isize, cy: isize, glyph: &[&str], raw: bool) {
    let mid = glyph.len() as f64 / 2.0;
    for (row, line) in glyph.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let Some(color) = sprite_color(ch) else { continue };
            let x = cx + col as isize - mid as isize;
            let y = cy + row as isize - mid as isize;
            if raw {
                g.set_raw(x, y, color);
            } else {
                g.set(x, y, color);
            }
        }
    }
}

/// Stamps one picture-derived icon (palette indices from `src/icons.rs`).
fn stamp_icon(g: &mut Grid, cx: isize, cy: isize, pixels: &[u8]) {
    let mid = ICON_SIZE as isize / 2;
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let idx = pixels[y * ICON_SIZE + x];
            if idx == P_TRANSPARENT {
                continue;
            }
            g.set(cx + x as isize - mid, cy + y as isize - mid, idx);
        }
    }
}

fn draw_entities(g: &mut Grid, c: &Client, cx: f64, cz: f64, facing: f64, player_kind: u32) {
    if !settings::SHOW_RADAR.get() {
        return;
    }
    let reach = if g.iw == state::INNER { 96.0 } else { 256.0 };
    for id in c.entities() {
        let Some(e) = c.entity(id) else { continue };
        let dx = e.pos.x - cx;
        let dz = e.pos.z - cz;
        if dx * dx + dz * dz > reach * reach {
            continue;
        }
        let name = weft_sdk::name_of(weft_sdk::abi::entry_kind::ENTITY, e.kind);
        let Some((col, row)) = project(g, cx, cz, facing, e.pos.x.floor() as i32, e.pos.z.floor() as i32) else { continue };
        if e.kind == player_kind || name.contains("player") {
            if let Some((_, sprite)) = SPRITES.iter().find(|(k, _)| *k == "player") {
                stamp(g, col, row, sprite, false);
            }
        } else if let Some(key) = palette::sprite_of(e.kind, name) {
            // Prefer the actual reference picture; the hand-written sprites stay
            // as a fallback for mobs without one (creeper, chicken).
            if let Some((_, pixels)) = icons::ICONS.iter().find(|(k, _)| *k == key) {
                stamp_icon(g, col, row, pixels);
            } else if let Some((_, sprite)) = SPRITES.iter().find(|(k, _)| *k == key) {
                stamp(g, col, row, sprite, false);
            }
        } else if name.contains("item") {
            g.set(col, row, P_ITEM);
        } else if ["zombie", "skeleton", "spider", "creeper"].iter().any(|m| name.contains(m)) {
            g.set(col, row, P_HOSTILE);
        } else {
            g.set(col, row, P_PASSIVE);
        }
    }
}

fn draw_waypoints(g: &mut Grid, cx: f64, cz: f64, facing: f64, dim: u64) {
    if !settings::SHOW_WAYPOINTS.get() {
        return;
    }
    for wp in waypoints::in_dimension(dim) {
        let Some((col, row)) = project(g, cx, cz, facing, wp.x, wp.z) else { continue };
        for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
            g.set(col + dx, row + dy, wp.color);
        }
        g.set(col, row, P_WHITE);
    }
}

/// A plain red arrowhead with a darker red rim, like Xaero's player marker.
const ARROW: [&str; 8] = [
    "....e....",
    "...exe...",
    "..exxxe..",
    ".exxxxxe.",
    "exxxxxxxe",
    "eaxxxxxae",
    ".eaaxxaae",
    "..eeeeee.",
];

fn draw_player(g: &mut Grid, angle: f64) {
    let cx = (g.x0 + g.iw / 2) as isize;
    let cy = (g.y0 + g.ih / 2) as isize;
    let (sin, cos) = angle.sin_cos();
    let mid = 4.0;
    for (row, line) in ARROW.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let Some(color) = sprite_color(ch) else { continue };
            let x = col as f64 + 0.5 - 4.5;
            let y = row as f64 + 0.5 - mid;
            let rx = x * cos - y * sin;
            let ry = x * sin + y * cos;
            g.set_raw(cx + rx.round() as isize, cy + ry.round() as isize, color);
        }
    }
}

const LETTERS: [[&str; 7]; 4] = [
    ["w...w", "ww..w", "w.w.w", "w..ww", "w...w", "w...w", "w...w"], // N
    ["wwwww", "w....", "w....", "wwww.", "w....", "w....", "wwwww"], // E
    [".wwww", "w....", "w....", ".www.", "....w", "....w", "wwww."], // S
    ["w...w", "w...w", "w...w", "w.w.w", "ww.ww", "w...w", "w...w"], // W
];

fn stamp_shadow(g: &mut Grid, cx: isize, cy: isize, glyph: &[&str]) {
    for (row, line) in glyph.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if sprite_color(ch).is_some() {
                let x = cx + col as isize - glyph.len() as isize / 2;
                let y = cy + row as isize - glyph.len() as isize / 2;
                g.set_raw(x + 1, y + 1, P_SPR_DARK);
            }
        }
    }
}

fn draw_compass(g: &mut Grid, facing: f64) {
    let (sin, cos) = facing.sin_cos();
    let cx = (g.x0 + g.iw / 2) as f64;
    let cy = (g.y0 + g.ih / 2) as f64;
    let r = (g.iw.min(g.ih) / 2) as f64 + 4.0;
    let dirs = [(-sin, -cos), (cos, -sin), (sin, cos), (-cos, sin)];
    for (letter, (dx, dy)) in LETTERS.iter().zip(dirs) {
        let x = (cx + dx * r).round() as isize;
        let y = (cy + dy * r).round() as isize;
        stamp_shadow(g, x, y, letter);
        stamp(g, x, y, letter, true);
    }
}

pub fn build_minimap(c: &Client) {
    if !settings::ENABLED.get() {
        state::publish_map(vec![P_TRANSPARENT; state::GRID * state::GRID]);
        return;
    }
    let st = c.state();
    let dim = weft_sdk::dimension_key(st.dim);
    let facing = if settings::LOCK_NORTH.get() { 0.0 } else { -st.yaw as f64 };
    let circle = settings::SHAPE.get() == 1;
    let mut g = Grid::new(
        state::GRID, state::GRID,
        state::MARGIN, state::MARGIN, state::INNER, state::INNER,
    );
    sample_terrain(
        &mut g, c, st.pos.x, st.pos.z, st.pos.y.floor() as i32, settings::ZOOM.get(), facing,
        settings::CAVE_DEPTH.get() as i32, settings::CAVE_MODE.get(), 8,
    );
    shade_terrain(&mut g, circle, daylight(c));
    chunk_lines(&mut g);
    draw_waypoints(&mut g, st.pos.x, st.pos.z, facing, dim);
    let player_kind = c.player_state().map_or(0, |p| p.kind);
    draw_entities(&mut g, c, st.pos.x, st.pos.z, facing, player_kind);
    draw_frame(&mut g, circle);
    draw_compass(&mut g, facing);
    // Last, so grid lines and entity dots can never cover the player arrow.
    draw_player(&mut g, if settings::LOCK_NORTH.get() { facing } else { 0.0 });
    state::publish_map(g.px);
}

/// The fullscreen map is a PNG instead of thousands of UI image cells, so it
/// can use the same fine pixels as the minimap while filling the screen.
pub fn build_big_map(c: &Client) {
    let st = c.state();
    let dim = weft_sdk::dimension_key(st.dim);
    let mut g = Grid::new(state::BIG_W, state::BIG_H, 0, 0, state::BIG_W, state::BIG_H);
    sample_terrain(
        &mut g, c, st.pos.x, st.pos.z, st.pos.y.floor() as i32, settings::BIG_ZOOM.get(), 0.0,
        settings::CAVE_DEPTH.get() as i32, settings::CAVE_MODE.get(), 24,
    );
    shade_terrain(&mut g, false, daylight(c));
    chunk_lines(&mut g);
    draw_waypoints(&mut g, st.pos.x, st.pos.z, 0.0, dim);
    let player_kind = c.player_state().map_or(0, |p| p.kind);
    draw_entities(&mut g, c, st.pos.x, st.pos.z, 0.0, player_kind);

    // A thin white rim, like the border of the map in the reference pictures.
    for x in 0..state::BIG_W {
        g.set_raw(x as isize, 0, P_WHITE);
        g.set_raw(x as isize, (state::BIG_H - 1) as isize, P_WHITE);
    }
    for y in 0..state::BIG_H {
        g.set_raw(0, y as isize, P_WHITE);
        g.set_raw((state::BIG_W - 1) as isize, y as isize, P_WHITE);
    }
    draw_player(&mut g, -st.yaw as f64);

    let mut rgba = vec![0u8; state::BIG_W * state::BIG_H * 4];
    for (i, idx) in g.px.iter().enumerate() {
        let rgb = crate::palette_rgb::RGB[(*idx as usize).min(255)];
        rgba[i * 4] = rgb[0];
        rgba[i * 4 + 1] = rgb[1];
        rgba[i * 4 + 2] = rgb[2];
        rgba[i * 4 + 3] = 255;
    }
    weft_sdk::client::image_save("minimap_big.png", state::BIG_W as u32, state::BIG_H as u32, &rgba);
    state::publish_big(g.px);
}

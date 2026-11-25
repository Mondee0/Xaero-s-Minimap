//! All mod settings. They show up in Settings > Mod Settings on their own;
//! the mod's own settings screen offers the same values in a Xaero-like layout.

use weft_sdk::{Config, Registry, TextConfig};

pub static ENABLED: Config<bool> = Config::named("enabled");
/// 0 = Small, 1 = Medium, 2 = Large.
pub static SIZE: Config<u32> = Config::named("size");
/// 0 = Square, 1 = Circle.
pub static SHAPE: Config<u32> = Config::named("shape");
/// 0 = top-left, 1 = top-right, 2 = bottom-left, 3 = bottom-right.
pub static LOCATION: Config<u32> = Config::named("location");
/// Multiplier on the window-relative map size.
pub static MAP_SCALE: Config<f64> = Config::named("map_scale");
/// Blocks per map pixel.
pub static ZOOM: Config<f64> = Config::named("zoom");
pub static LOCK_NORTH: Config<bool> = Config::named("lock_north");
pub static COORDS: Config<bool> = Config::named("coords");
pub static SHOW_DIMENSION: Config<bool> = Config::named("show_dimension");
pub static SHOW_FACING: Config<bool> = Config::named("show_facing");
pub static SHOW_TIME: Config<bool> = Config::named("show_time");
pub static SHOW_WAYPOINTS: Config<bool> = Config::named("show_waypoints");
pub static SHOW_RADAR: Config<bool> = Config::named("show_radar");
pub static SHOW_GRID: Config<bool> = Config::named("show_grid");
pub static CAVE_MODE: Config<bool> = Config::named("cave_mode");
pub static CAVE_DEPTH: Config<i64> = Config::named("cave_depth");
/// Blocks per pixel on the fullscreen map.
pub static BIG_ZOOM: Config<f64> = Config::named("big_zoom");
/// Serialized waypoints (see `waypoints.rs`).
pub static WAYPOINTS: TextConfig = TextConfig::named("waypoints");

/// Size one map cell so the whole map is about 72% of the window height —
/// the proportion of the two maps in Xaero's promo picture — whatever the
/// player's GUI scale is.
pub fn minimap_cell(window_h_px: u32, gui_scale: f32) -> f32 {
    let base = (window_h_px.max(1) as f32 / gui_scale.max(0.5)) * 0.45 / crate::state::GRID as f32;
    let mult = match SIZE.get() {
        0 => 0.8,
        2 => 1.25,
        _ => 1.0,
    };
    base * mult * MAP_SCALE.get() as f32
}

pub fn register(r: &mut Registry) {
    r.config(&ENABLED).bool(true);
    r.config(&SIZE).choice(1, &["Small", "Medium", "Large"]);
    r.config(&SHAPE).choice(1, &["Square", "Circle"]);
    r.config(&LOCATION).choice(1, &["Top-left", "Top-right", "Bottom-left", "Bottom-right"]);
    r.config(&MAP_SCALE).float(0.85, 0.4, 1.4).step(0.05);
    r.config(&ZOOM).float(0.8, 0.4, 8.0).step(0.1);
    r.config(&LOCK_NORTH).bool(false);
    r.config(&COORDS).bool(true);
    r.config(&SHOW_DIMENSION).bool(true);
    r.config(&SHOW_FACING).bool(false);
    r.config(&SHOW_TIME).bool(false);
    r.config(&SHOW_WAYPOINTS).bool(true);
    r.config(&SHOW_RADAR).bool(true);
    r.config(&SHOW_GRID).bool(false);
    r.config(&CAVE_MODE).bool(false);
    r.config(&CAVE_DEPTH).int(24, 4, 128);
    r.config(&BIG_ZOOM).float(2.0, 0.5, 8.0).step(0.25);
    r.text_config(&WAYPOINTS, "");
}

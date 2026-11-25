//! Shared state between the client frame hook, the map builder and the HUD.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

use weft_sdk::{HudElement, Key, Menu, abi};

/// Whole HUD grid: the map itself plus a 10-cell margin for frame and compass.
pub const GRID: usize = 180;
pub const INNER: usize = 160;
pub const MARGIN: usize = 10;
pub const BIG_W: usize = 800;
pub const BIG_H: usize = 450;

fn blank(n: usize) -> Arc<Vec<u8>> {
    Arc::new(vec![super::palette::P_TRANSPARENT; n])
}

/// One element per corner: the location setting shows exactly one of them,
/// which lets the map move without a restart.
pub static MINIMAP_TL: HudElement = HudElement::named("minimap_tl");
pub static MINIMAP_TR: HudElement = HudElement::named("minimap_tr");
pub static MINIMAP_BL: HudElement = HudElement::named("minimap_bl");
pub static MINIMAP_BR: HudElement = HudElement::named("minimap_br");
pub static BIG_MAP: HudElement = HudElement::named("big_map");
pub static WAYPOINT_SCREEN: Menu = Menu::named("waypoints");
pub static SETTINGS_SCREEN: Menu = Menu::named("settings");

pub static TOGGLE: Key = Key::named("toggle");
pub static BIG: Key = Key::named("big_map");
pub static WAYPOINTS: Key = Key::named("waypoints");
pub static ZOOM_IN: Key = Key::named("zoom_in");
pub static ZOOM_OUT: Key = Key::named("zoom_out");
pub static ENLARGE: Key = Key::named("enlarge");
pub static QUICK_WAYPOINT: Key = Key::named("quick_waypoint");
pub static GRID_KEY: Key = Key::named("grid");
pub static RADAR_KEY: Key = Key::named("radar");
pub static WAYPOINTS_KEY: Key = Key::named("waypoints_on_map");
pub static CAVE_KEY: Key = Key::named("cave_mode");

pub static MAP: LazyLock<RwLock<Arc<Vec<u8>>>> =
    LazyLock::new(|| RwLock::new(blank(GRID * GRID)));
pub static BIG_PIXELS: LazyLock<RwLock<Arc<Vec<u8>>>> =
    LazyLock::new(|| RwLock::new(blank(BIG_W * BIG_H)));
pub static MAP_REV: AtomicU64 = AtomicU64::new(0);
pub static BIG_REV: AtomicU64 = AtomicU64::new(0);
pub static BIG_OPEN: AtomicBool = AtomicBool::new(false);

/// Client state captured in the frame hook. UI callbacks get a view without an
/// engine pointer, so `c.state()` is all zeros there — never use it in a UI.
#[derive(Clone, Copy, Default)]
pub struct Snapshot {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub dim: u32,
    pub day_time: u64,
    pub window_w: u32,
    pub window_h: u32,
    pub gui_scale: f32,
}

static SNAPSHOT: LazyLock<RwLock<Snapshot>> = LazyLock::new(|| RwLock::new(Snapshot::default()));
pub static ENLARGE_HELD: AtomicBool = AtomicBool::new(false);

pub fn snapshot() -> Snapshot {
    *SNAPSHOT.read().unwrap()
}

pub fn update_snapshot(st: &abi::ClientState, day_time: u64) {
    *SNAPSHOT.write().unwrap() = Snapshot {
        x: st.pos.x,
        y: st.pos.y,
        z: st.pos.z,
        yaw: st.yaw,
        dim: st.dim,
        day_time,
        window_w: st.window[0].max(1),
        window_h: st.window[1].max(1),
        gui_scale: if st.gui_scale > 0.0 { st.gui_scale } else { 2.0 },
    };
}

pub fn map_image() -> Arc<Vec<u8>> {
    MAP.read().unwrap().clone()
}

pub fn publish_map(pixels: Vec<u8>) {
    *MAP.write().unwrap() = Arc::new(pixels);
    MAP_REV.fetch_add(1, Ordering::Relaxed);
}

pub fn publish_big(pixels: Vec<u8>) {
    *BIG_PIXELS.write().unwrap() = Arc::new(pixels);
    BIG_REV.fetch_add(1, Ordering::Relaxed);
}

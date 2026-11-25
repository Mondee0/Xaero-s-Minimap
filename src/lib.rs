//! A recreation of Xaero's Minimap for Weft.
//!
//! Built only on the public SDK: terrain is read with `Client::block`, the map
//! is a grid of palette-colored GUI cells rebuilt by a HUD element, and the
//! waypoints live in a text setting. No engine changes and no runtime textures.

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

mod icons;
mod map;
mod palette;
mod palette_rgb;
mod settings;
mod state;
mod ui;
mod waypoints;

fn init(r: &mut Registry) {
    settings::register(r);
    ui::register(r);
}

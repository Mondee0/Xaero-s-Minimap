//! The fixed map palette: 32 terrain colors in 4 shades (0..127), then markers,
//! frame and mob-sprite colors (128..159). Each entry is a one-pixel GUI texture
//! made by `tools/generate-assets.ps1`.

#![allow(dead_code)]

use weft_sdk::{abi::entry_kind, name_of};

// Terrain bases (the shade is added as 0..3).
pub const B_GRASS: u8 = 0;
pub const B_GRASS_LIGHT: u8 = 1;
pub const B_GRASS_DARK: u8 = 2;
pub const B_LEAVES: u8 = 3;
pub const B_LEAVES_DARK: u8 = 4;
pub const B_LEAVES_LIGHT: u8 = 5;
pub const B_DIRT: u8 = 6;
pub const B_DIRT_DARK: u8 = 7;
pub const B_STONE: u8 = 8;
pub const B_STONE_DARK: u8 = 9;
pub const B_STONE_LIGHT: u8 = 10;
pub const B_COBBLE: u8 = 11;
pub const B_SAND: u8 = 12;
pub const B_SAND_LIGHT: u8 = 13;
pub const B_SANDSTONE: u8 = 14;
pub const B_GRAVEL: u8 = 15;
pub const B_WATER: u8 = 16;
pub const B_WATER_DARK: u8 = 17;
pub const B_WATER_LIGHT: u8 = 18;
pub const B_SNOW: u8 = 19;
pub const B_ICE: u8 = 20;
pub const B_WOOD: u8 = 21;
pub const B_PLANKS: u8 = 22;
pub const B_BARK: u8 = 23;
pub const B_FLOWERS: u8 = 24;
pub const B_YELLOW: u8 = 25;
pub const B_RED: u8 = 26;
pub const B_LAVA: u8 = 27;
pub const B_NETHER: u8 = 28;
pub const B_END: u8 = 29;
pub const B_ORE: u8 = 30;
pub const B_MOSS: u8 = 31;

pub const SHADE_DARK: u8 = 0;
pub const SHADE_MID_DARK: u8 = 1;
pub const SHADE_NORMAL: u8 = 2;
pub const SHADE_LIGHT: u8 = 3;

// Overlays, frame and sprite colors.
pub const P_TRANSPARENT: u8 = 128;
pub const P_UNDISCOVERED: u8 = 129;
pub const P_BLACK: u8 = 130;
pub const P_WHITE: u8 = 131;
pub const P_FRAME_OUTER: u8 = 132;
pub const P_FRAME_LIGHT: u8 = 133;
pub const P_FRAME_DARK: u8 = 134;
pub const P_FRAME_MID: u8 = 135;
pub const P_HOSTILE: u8 = 136;
pub const P_PASSIVE: u8 = 137;
pub const P_ITEM: u8 = 138;
pub const P_PLAYER: u8 = 139;
pub const P_WP_BLUE: u8 = 140;
pub const P_WP_GREEN: u8 = 141;
pub const P_WP_ORANGE: u8 = 142;
pub const P_WP_PINK: u8 = 143;
pub const P_WP_YELLOW: u8 = 144;
pub const P_ARROW: u8 = 145;
pub const P_ARROW_DARK: u8 = 146;
pub const P_SPR_DARK: u8 = 147;
pub const P_SPR_WHITE: u8 = 148;
pub const P_SPR_GREEN: u8 = 149;
pub const P_SPR_GREEN_DARK: u8 = 150;
pub const P_SPR_PINK: u8 = 151;
pub const P_SPR_PINK_DARK: u8 = 152;
pub const P_SPR_BROWN: u8 = 153;
pub const P_SPR_BROWN_DARK: u8 = 154;
pub const P_SPR_RED: u8 = 155;
pub const P_SPR_ORANGE: u8 = 156;
pub const P_SPR_YELLOW: u8 = 157;
pub const P_SPR_GRAY: u8 = 158;
pub const P_SPR_GRAY_DARK: u8 = 159;
pub const P_SPR_LIGHT_GRAY: u8 = 160;
pub const P_SPR_TAN: u8 = 161;
pub const P_SPR_TAN_DARK: u8 = 162;
pub const P_SPR_TAN_LIGHT: u8 = 163;
pub const P_SPR_SNOUT: u8 = 164;
pub const P_SPR_SNOUT_DARK: u8 = 165;
pub const P_SPR_BRICK: u8 = 166;
pub const P_SPR_BRICK_DARK: u8 = 167;
pub const P_SPR_EYE_RED: u8 = 168;
pub const P_SPR_EYE_RED_DARK: u8 = 169;
pub const P_SPR_GREEN_LIGHT: u8 = 170;

pub const WAYPOINT_COLORS: [u8; 5] = [
    P_WP_YELLOW,
    P_WP_BLUE,
    P_WP_GREEN,
    P_WP_ORANGE,
    P_WP_PINK,
];

/// GUI texture name of a palette entry.
pub fn texture(idx: u8) -> &'static str {
    static NAMES: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
        (0..256).map(|i| format!("minimap:gui/minimap/p{i:03}")).collect()
    });
    &NAMES[(idx as usize).min(255)]
}

/// In-world label color (0xRRGGBBAA) of a waypoint palette entry.
pub fn waypoint_rgb(idx: u8) -> u32 {
    match idx {
        P_WP_GREEN => 0x30D070FF,
        P_WP_ORANGE => 0xF09020FF,
        P_WP_PINK => 0xE050A0FF,
        P_WP_BLUE => 0x3060F0FF,
        _ => 0xF0D028FF,
    }
}

/// Terrain base of a block that should show on the map; `None` for plants and
/// other thin things that hide the ground rather than being it.
pub fn surface_base(state: u32) -> Option<u8> {
    let full = name_of(entry_kind::BLOCK, state);
    let name = full.split('#').next().unwrap_or(full);
    let path = name.split_once(':').map_or(name, |(_, p)| p);
    if matches!(
        path,
        "tall_grass" | "sapling" | "torch" | "lever" | "redstone_torch" | "comparator"
            | "repeater" | "observer" | "stone_button"
    ) {
        return None;
    }
    Some(match path {
        "grass" => B_GRASS,
        "snowy_grass" => B_GRASS_LIGHT,
        "oak_leaves" | "leaves" => B_LEAVES,
        "birch_leaves" => B_LEAVES_LIGHT,
        "spruce_leaves" => B_LEAVES_DARK,
        "mossy_cobblestone" | "slime_block" => B_MOSS,
        "dirt" | "farmland" => B_DIRT,
        "clay" => B_STONE_LIGHT,
        "stone" | "stone_bricks" => B_STONE,
        "bricks" => B_RED,
        "sand" => B_SAND,
        "sandstone" => B_SANDSTONE,
        "gravel" => B_GRAVEL,
        "cobblestone" => B_COBBLE,
        "water" => B_WATER,
        "ice" => B_ICE,
        "snow" => B_SNOW,
        "oak_log" | "birch_log" | "spruce_log" | "bark" => B_BARK,
        "oak_planks" | "birch_planks" | "spruce_planks" | "crafting_table" | "chest"
        | "furnace" | "woodcutter" | "ladder" | "oak_door" | "dispenser" | "piston"
        | "piston_head" | "sticky_piston" | "sticky_piston_head" | "bed" | "beds" => B_PLANKS,
        "cactus" | "sugar_cane" => B_LEAVES_LIGHT,
        // Flowers and crops stay close to grass: bright single-block specks
        // make the map look like confetti instead of terrain.
        "wheat" | "dandelion" | "rose" => B_GRASS_LIGHT,
        "lava" => B_LAVA,
        "obsidian" | "redstone_block" | "redstone_lamp" | "golden_lamp" => B_NETHER,
        "wool" | "floater" => B_END,
        "coal_block" | "coal_ore" | "iron_block" | "iron_ore" | "diamond_block"
        | "diamond_ore" => B_ORE,
        "gold_block" | "gold_ore" | "glowstone" => B_YELLOW,
        "redstone_ore" | "redstone" => B_DIRT,
        _ => {
            // Unknown blocks get one of four stable earth tones rather than a
            // random color from the whole palette.
            let h = full.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
            [B_GRASS, B_DIRT, B_STONE, B_LEAVES][(h % 4) as usize]
        }
    })
}

/// Sprite key of an entity kind, or `None` for things without a face sprite.
pub fn sprite_of(kind: u32, name: &str) -> Option<&'static str> {
    if kind != 0 && name.contains("player") {
        return Some("player");
    }
    if name.contains("creeper") {
        Some("creeper")
    } else if name.contains("zombie") {
        Some("zombie")
    } else if name.contains("skeleton") {
        Some("skeleton")
    } else if name.contains("spider") {
        Some("spider")
    } else if name.contains("pig") {
        Some("pig")
    } else if name.contains("cow") {
        Some("cow")
    } else if name.contains("sheep") {
        Some("sheep")
    } else if name.contains("chicken") {
        Some("chicken")
    } else {
        None
    }
}

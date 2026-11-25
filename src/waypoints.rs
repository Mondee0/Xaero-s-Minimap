//! Waypoints: plain client-side data, saved in the `minimap:waypoints` text setting.
//! Xaero stores them per world/dimension; we keep the dimension key on each one
//! and only show the ones that belong where the player currently is.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

use crate::palette::WAYPOINT_COLORS;
use crate::settings::WAYPOINTS;

#[derive(Clone, Debug)]
pub struct Waypoint {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub color: u8,
    pub dim: u64,
}

#[derive(Default)]
pub struct Editor {
    pub name: String,
    pub selected: isize,
}

static LIST: LazyLock<RwLock<Vec<Waypoint>>> = LazyLock::new(|| RwLock::new(Vec::new()));
pub static EDITOR: LazyLock<Mutex<Editor>> = LazyLock::new(|| Mutex::new(Editor::default()));
static LOADED: AtomicBool = AtomicBool::new(false);

fn escape(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            ';' => out.push_str("\\;"),
            '|' => out.push_str("\\|"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
}

/// Splits on an unescaped separator, dropping the escaping backslash.
fn split_on(s: &str, separator: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut esc = false;
    for c in s.chars() {
        if esc {
            if c == 'n' {
                cur.push('\n');
            } else {
                cur.push(c);
            }
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == separator {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn save() {
    let mut text = String::new();
    for wp in LIST.read().unwrap().iter() {
        if !text.is_empty() {
            text.push('|');
        }
        escape(&wp.name, &mut text);
        text.push_str(&format!(
            ";{};{};{};{};{}",
            wp.x, wp.y, wp.z, wp.color, wp.dim
        ));
    }
    WAYPOINTS.set(&text);
}

fn ensure_loaded() {
    if LOADED.swap(true, Ordering::Relaxed) {
        return;
    }
    let mut list = Vec::new();
    for record in split_on(&WAYPOINTS.get(), '|') {
        let f = split_on(&record, ';');
        if f.len() < 6 {
            continue;
        }
        let parse = |i: usize| f[i].trim().parse::<i64>().unwrap_or(0);
        list.push(Waypoint {
            name: f[0].clone(),
            x: parse(1) as i32,
            y: parse(2) as i32,
            z: parse(3) as i32,
            color: (parse(4) as u8).clamp(48, 63),
            dim: parse(5) as u64,
        });
    }
    *LIST.write().unwrap() = list;
}

/// All waypoints that belong to `dim`, cloned for drawing.
pub fn in_dimension(dim: u64) -> Vec<Waypoint> {
    ensure_loaded();
    LIST.read()
        .unwrap()
        .iter()
        .filter(|w| w.dim == dim)
        .cloned()
        .collect()
}

pub fn all() -> Vec<Waypoint> {
    ensure_loaded();
    LIST.read().unwrap().clone()
}

pub fn add(name: &str, x: i32, y: i32, z: i32, dim: u64) {
    ensure_loaded();
    let color = WAYPOINT_COLORS[LIST.read().unwrap().len() % WAYPOINT_COLORS.len()];
    let name = if name.trim().is_empty() {
        "Waypoint".to_string()
    } else {
        name.trim().to_string()
    };
    let mut list = LIST.write().unwrap();
    // Keep the text setting small even after many deaths / quick waypoints.
    if list.len() >= 64 {
        list.remove(0);
    }
    list.push(Waypoint { name, x, y, z, color, dim });
    drop(list);
    save();
}

pub fn add_death(x: i32, y: i32, z: i32, dim: u64) {
    add("Death point", x, y, z, dim);
}

pub fn delete(index: usize) -> bool {
    ensure_loaded();
    let mut list = LIST.write().unwrap();
    if index >= list.len() {
        return false;
    }
    list.remove(index);
    drop(list);
    save();
    true
}

pub fn recolor(index: usize) {
    ensure_loaded();
    let mut list = LIST.write().unwrap();
    if let Some(w) = list.get_mut(index) {
        let pos = WAYPOINT_COLORS.iter().position(|c| *c == w.color).unwrap_or(0);
        w.color = WAYPOINT_COLORS[(pos + 1) % WAYPOINT_COLORS.len()];
    }
    drop(list);
    save();
}

pub fn rename(index: usize, name: &str) {
    ensure_loaded();
    let mut list = LIST.write().unwrap();
    if let Some(w) = list.get_mut(index) {
        w.name = name.to_string();
    }
    drop(list);
    save();
}

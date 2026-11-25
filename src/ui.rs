//! HUD elements, settings/waypoint screens, key bindings and the client frame hook.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use weft_sdk::{
    Arg, ClientView, Output, Registry, S, TextOut, abi, client::Client, hud_anchor,
};

use crate::palette;
use crate::{map, settings, state, waypoints};

static LAST_MINIMAP: LazyLock<Mutex<Instant>> = LazyLock::new(|| Mutex::new(Instant::now()));
static LAST_BIG: LazyLock<Mutex<Instant>> = LazyLock::new(|| Mutex::new(Instant::now()));
static WAS_DEAD: AtomicBool = AtomicBool::new(false);

fn coord_line(_v: &ClientView, out: &mut TextOut) {
    if !settings::COORDS.get() {
        return;
    }
    let s = state::snapshot();
    out.text(&format!("XYZ: {:.1} / {:.1} / {:.1}", s.x, s.y, s.z));
}

fn info_line(_v: &ClientView, out: &mut TextOut) {
    let s = state::snapshot();
    let mut parts = Vec::new();
    if settings::SHOW_DIMENSION.get() {
        let name = weft_sdk::name_of(abi::entry_kind::DIMENSION, s.dim);
        parts.push(name.strip_prefix("base:").unwrap_or(name).to_string());
    }
    if settings::SHOW_FACING.get() {
        let deg = (-s.yaw.to_degrees()).rem_euclid(360.0);
        let dir = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"][((deg + 22.5) / 45.0) as usize % 8];
        parts.push(format!("{dir} {deg:.0}°"));
    }
    if settings::SHOW_TIME.get() {
        let t = s.day_time % 24000;
        let phase = if (6000..18000).contains(&t) { "Day" } else { "Night" };
        parts.push(format!("{phase} {t}"));
    }
    out.text(&parts.join("  "));
}

fn waypoint_line(_v: &ClientView, out: &mut TextOut) {
    if !settings::SHOW_WAYPOINTS.get() {
        return;
    }
    let s = state::snapshot();
    let dim = weft_sdk::dimension_key(s.dim);
    let mut nearest: Option<(f64, &waypoints::Waypoint)> = None;
    // A small owned copy keeps the closure simple and avoids holding a lock here.
    let list = waypoints::in_dimension(dim);
    for wp in &list {
        let dx = wp.x as f64 - s.x;
        let dy = wp.y as f64 - s.y;
        let dz = wp.z as f64 - s.z;
        let d = (dx * dx + dy * dy + dz * dz).sqrt();
        if nearest.as_ref().is_none_or(|(best, _)| d < *best) {
            nearest = Some((d, wp));
        }
    }
    if let Some((d, wp)) = nearest {
        out.text(&format!("→ {}  {d:.0}m", wp.name));
    }
}

fn draw_minimap(ui: &mut weft_sdk::Ui) {
    if !settings::ENABLED.get() {
        return;
    }
    let px = state::map_image();
    let snap = state::snapshot();
    let enlarged = state::ENLARGE_HELD.load(Ordering::Relaxed);
    let cell = (settings::minimap_cell(snap.window_h, snap.gui_scale)
        * if enlarged { 1.3 } else { 1.0 })
        .max(0.35);
    // No gap inside the grid: a gap between rows would stretch the map and
    // leave visible stripes between the cells.
    ui.column(S::column(), |ui| {
        ui.column(S::column(), |ui| {
            for row in 0..state::GRID {
                ui.row(S::row(), |ui| {
                    for col in 0..state::GRID {
                        ui.image(palette::texture(px[row * state::GRID + col]), cell, cell);
                    }
                });
            }
        });
        ui.column(S::column().gap(3.0).pad(2.0), |ui| {
            if settings::COORDS.get() {
                ui.text_bind_styled(
                    S::column().class("minimap_coords").scale(0.9).no_wrap(),
                    coord_line,
                );
            }
            if settings::SHOW_DIMENSION.get() || settings::SHOW_FACING.get() || settings::SHOW_TIME.get() {
                ui.text_bind_styled(S::column().class("minimap_coords").no_wrap(), info_line);
            }
            ui.text_bind_styled(S::column().class("minimap_coords").no_wrap(), waypoint_line);
        });
    });
}

fn minimap_hud(r: &mut Registry) {
    let locations = [
        (&state::MINIMAP_TL, hud_anchor::TOP_LEFT, 0u32),
        (&state::MINIMAP_TR, hud_anchor::TOP_RIGHT, 1),
        (&state::MINIMAP_BL, hud_anchor::BOTTOM_LEFT, 2),
        (&state::MINIMAP_BR, hud_anchor::BOTTOM_RIGHT, 3),
    ];
    for (element, anchor, id) in locations {
        r.hud_element(element, anchor, 0, move |ui| {
            if settings::LOCATION.get() == id {
                draw_minimap(ui);
            }
        })
        .rebuild_when(move |_| {
            if settings::LOCATION.get() == id {
                state::MAP_REV.load(Ordering::Relaxed)
            } else {
                0
            }
        });
    }
}

fn big_map_hud(r: &mut Registry) {
    // Anchored at the top-left with a late order: the picture is sized to the
    // whole window, so it covers every corner and never follows the camera.
    r.hud_element(&state::BIG_MAP, hud_anchor::TOP_LEFT, 10000, |ui| {
        if !state::BIG_OPEN.load(Ordering::Relaxed) {
            return;
        }
        let snap = state::snapshot();
        let rev = state::BIG_REV.load(Ordering::Relaxed);
        let name = format!("file:minimap_big.png|{rev}");
        let scale = snap.gui_scale.max(0.5);
        // The HUD stack has a 4px inset; overshoot so the picture reaches all
        // four edges of the real window.
        let w = snap.window_w as f32 / scale + 16.0;
        let h = snap.window_h as f32 / scale + 16.0;
        // A negative pad pulls the picture into the HUD stack's 4px inset so
        // it sits flush with the top and left edges of the window.
        ui.column(S::column().pad(-8.0), |ui| {
            ui.image(&name, w, h);
        });
    })
    .rebuild_when(|_| {
        if state::BIG_OPEN.load(Ordering::Relaxed) {
            state::BIG_REV.load(Ordering::Relaxed).wrapping_add(1)
        } else {
            0
        }
    });
}

fn toggle_button(ui: &mut weft_sdk::Ui, key: &str, value: fn() -> bool, setter: fn(bool)) {
    ui.button(key, S::default(), move |_| setter(!value()));
    ui.text_bind(move |_, out| {
        out.set(if value() { "minimap.on" } else { "minimap.off" }, &[]);
    });
}

fn settings_screen(r: &mut Registry) {
    r.menu(&state::SETTINGS_SCREEN)
        .screen(|ui| {
            ui.panel("panel", S::column().pad(8.0).gap(3.0).min_w(270.0), |ui| {
                ui.text_styled(
                    S::column().class("title").scale(2.0),
                    "minimap.settings.title",
                    &[],
                );
                toggle_button(ui, "minimap.settings.enabled", || settings::ENABLED.get(), |v| settings::ENABLED.set(v));
                ui.button("minimap.settings.cycle_size", S::default(), |_| {
                    settings::SIZE.set((settings::SIZE.get() + 1) % 3);
                });
                ui.text_bind(|_, out| {
                    out.text(["minimap.size.small", "minimap.size.medium", "minimap.size.large"]
                        [settings::SIZE.get() as usize % 3]);
                });
                ui.button("minimap.settings.cycle_shape", S::default(), |_| {
                    settings::SHAPE.set((settings::SHAPE.get() + 1) % 2);
                });
                ui.text_bind(|_, out| {
                    out.text(if settings::SHAPE.get() == 1 { "minimap.shape.circle" } else { "minimap.shape.square" });
                });
                ui.button("minimap.settings.cycle_location", S::default(), |_| {
                    settings::LOCATION.set((settings::LOCATION.get() + 1) % 4);
                });
                ui.text_bind(|_, out| {
                    out.set(
                        [
                            "minimap.location.tl",
                            "minimap.location.tr",
                            "minimap.location.bl",
                            "minimap.location.br",
                        ][settings::LOCATION.get() as usize % 4],
                        &[],
                    );
                });
                ui.slider(
                    S::column().w(200.0).h(16.0),
                    |_| settings::MAP_SCALE.get() as f32,
                    |_, v| settings::MAP_SCALE.set(v as f64),
                );
                ui.text_bind(|_, out| out.text(&format!("Map size: {:.0}%", settings::MAP_SCALE.get() * 100.0)));
                ui.slider(
                    S::column().w(200.0).h(16.0),
                    |_| ((settings::ZOOM.get() - 0.5) / 7.5) as f32,
                    |_, v| settings::ZOOM.set(0.5 + v as f64 * 7.5),
                );
                ui.text_bind(|_, out| out.text(&format!("Zoom: {:.1}", settings::ZOOM.get())));
                toggle_button(ui, "minimap.settings.lock_north", || settings::LOCK_NORTH.get(), |v| settings::LOCK_NORTH.set(v));
                toggle_button(ui, "minimap.settings.coords", || settings::COORDS.get(), |v| settings::COORDS.set(v));
                toggle_button(ui, "minimap.settings.dimension", || settings::SHOW_DIMENSION.get(), |v| settings::SHOW_DIMENSION.set(v));
                toggle_button(ui, "minimap.settings.facing", || settings::SHOW_FACING.get(), |v| settings::SHOW_FACING.set(v));
                toggle_button(ui, "minimap.settings.time", || settings::SHOW_TIME.get(), |v| settings::SHOW_TIME.set(v));
                toggle_button(ui, "minimap.settings.waypoints", || settings::SHOW_WAYPOINTS.get(), |v| settings::SHOW_WAYPOINTS.set(v));
                toggle_button(ui, "minimap.settings.radar", || settings::SHOW_RADAR.get(), |v| settings::SHOW_RADAR.set(v));
                toggle_button(ui, "minimap.settings.grid", || settings::SHOW_GRID.get(), |v| settings::SHOW_GRID.set(v));
                toggle_button(ui, "minimap.settings.cave", || settings::CAVE_MODE.get(), |v| settings::CAVE_MODE.set(v));
                ui.slider(
                    S::column().w(200.0).h(16.0),
                    |_| ((settings::BIG_ZOOM.get() - 1.0) / 15.0) as f32,
                    |_, v| settings::BIG_ZOOM.set(1.0 + v as f64 * 15.0),
                );
                ui.text_bind(|_, out| out.text(&format!("Map zoom: {:.1}", settings::BIG_ZOOM.get())));
                ui.button("minimap.settings.close", S::default(), |v| v.client().menu_close());
            });
        })
        .pauses();
}

const WP_ROWS: usize = 8;

fn waypoint_screen(r: &mut Registry) {
    r.menu(&state::WAYPOINT_SCREEN)
        .screen(|ui| {
            ui.panel("panel", S::column().pad(8.0).gap(2.0).min_w(340.0).scroll(), |ui| {
                ui.text_styled(
                    S::column().class("title").scale(2.0),
                    "minimap.waypoints.title",
                    &[],
                );
                ui.input("", 32, S::column().w(220.0), |_, text| {
                    waypoints::EDITOR.lock().unwrap().name = text.to_string();
                });
                ui.row(S::row().gap(4.0), |ui| {
                    ui.button("minimap.waypoints.add", S::default(), |_| {
                        let s = state::snapshot();
                        let name = waypoints::EDITOR.lock().unwrap().name.clone();
                        waypoints::add(
                            &name,
                            s.x.floor() as i32,
                            s.y.floor() as i32,
                            s.z.floor() as i32,
                            weft_sdk::dimension_key(s.dim),
                        );
                    });
                    ui.button("minimap.waypoints.rename", S::default(), |_| {
                        let (i, name) = {
                            let e = waypoints::EDITOR.lock().unwrap();
                            (e.selected, e.name.clone())
                        };
                        if i >= 0 {
                            waypoints::rename(i as usize, &name);
                        }
                    });
                });
                ui.text("minimap.waypoints.hint");
                for i in 0..WP_ROWS {
                    ui.row(S::row().gap(3.0), |ui| {
                        ui.text_bind_styled(S::row().min_w(180.0), move |_, out| {
                            let list = waypoints::all();
                            if let Some(w) = list.get(i) {
                                out.text(&format!("{}  {}  {}/{}", w.name, i + 1, w.x, w.z));
                            } else {
                                out.set("minimap.waypoints.empty", &[]);
                            }
                        });
                        ui.button("minimap.waypoints.select", S::default(), move |_| {
                            waypoints::EDITOR.lock().unwrap().selected = i as isize;
                        });
                        ui.button("minimap.waypoints.color", S::default(), move |_| {
                            waypoints::recolor(i);
                        });
                        ui.button("minimap.waypoints.delete", S::default(), move |_| {
                            waypoints::delete(i);
                        });
                    });
                }
                ui.button("minimap.waypoints.close", S::default(), |v| v.client().menu_close());
            });
        })
        .pauses();
}

fn open_big_map() {
    *LAST_BIG.lock().unwrap() = Instant::now() - Duration::from_secs(1);
    state::BIG_OPEN.store(true, Ordering::Relaxed);
}

fn keys(r: &mut Registry) {
    r.key(&state::TOGGLE, "Y").on_client(|_| {
        settings::ENABLED.set(!settings::ENABLED.get());
    });
    r.key(&state::BIG, "M").on_client(|_| {
        let was_open = state::BIG_OPEN.swap(false, Ordering::Relaxed);
        if !was_open {
            open_big_map();
        }
    });
    r.key(&state::WAYPOINTS, "B").on_client(|c| {
        c.client().menu_open(&state::WAYPOINT_SCREEN);
    });
    // = / - and the wheel: zoom the fullscreen map while it is open,
    // otherwise zoom the corner minimap.
    r.key(&state::ZOOM_IN, "Equal, wheel_up").on_client(|_| {
        if state::BIG_OPEN.load(Ordering::Relaxed) {
            settings::BIG_ZOOM.set((settings::BIG_ZOOM.get() - 0.25).max(0.5));
        } else {
            settings::ZOOM.set((settings::ZOOM.get() + 0.25).min(8.0));
        }
    });
    r.key(&state::ZOOM_OUT, "Minus, wheel_down").on_client(|_| {
        if state::BIG_OPEN.load(Ordering::Relaxed) {
            settings::BIG_ZOOM.set((settings::BIG_ZOOM.get() + 0.25).min(8.0));
        } else {
            settings::ZOOM.set((settings::ZOOM.get() - 0.25).max(0.5));
        }
    });
    r.key(&state::ENLARGE, "X");
    r.key(&state::QUICK_WAYPOINT, "N").on_client(|_| {
        let s = state::snapshot();
        waypoints::add(
            "Quick",
            s.x.floor() as i32,
            s.y.floor() as i32,
            s.z.floor() as i32,
            weft_sdk::dimension_key(s.dim),
        );
    });
    r.key(&state::GRID_KEY, "G").on_client(|_| {
        settings::SHOW_GRID.set(!settings::SHOW_GRID.get());
    });
    r.key(&state::RADAR_KEY, "R").on_client(|_| {
        settings::SHOW_RADAR.set(!settings::SHOW_RADAR.get());
    });
    r.key(&state::WAYPOINTS_KEY, "V").on_client(|_| {
        settings::SHOW_WAYPOINTS.set(!settings::SHOW_WAYPOINTS.get());
    });
    r.key(&state::CAVE_KEY, "C").on_client(|_| {
        settings::CAVE_MODE.set(!settings::CAVE_MODE.get());
    });
}

fn in_world_waypoints(c: &Client, st: &abi::ClientState) {
    if !settings::SHOW_WAYPOINTS.get() || !settings::ENABLED.get() {
        return;
    }
    let dim = weft_sdk::dimension_key(st.dim);
    for wp in waypoints::in_dimension(dim) {
        let dx = wp.x as f64 - st.pos.x;
        let dz = wp.z as f64 - st.pos.z;
        let flat = (dx * dx + dz * dz).sqrt();
        if flat > 128.0 {
            continue;
        }
        let base = [wp.x as f64 + 0.5, wp.y as f64 + 0.5, wp.z as f64 + 0.5];
        let color = palette::waypoint_rgb(wp.color);
        c.draw_text(
            None,
            [base[0], base[1] + 2.4, base[2]],
            "minimap.waypoint.label",
            &[Arg::Text(&wp.name), Arg::Int(flat.round() as i64)],
            color,
            1.0,
        );
        c.draw_fill(
            None,
            [base[0] - 0.05, base[1], base[2] - 0.05],
            [base[0] + 0.05, base[1] + 512.0, base[2] + 0.05],
            (color & 0xFFFFFF00) | 0x40,
        );
    }
}

fn frame_hook(r: &mut Registry) {
    r.client_frame(|c| {
        let st = c.state();
        if st.flags & abi::client_flags::IN_WORLD == 0 {
            return;
        }
        state::update_snapshot(&st, c.day_time());
        state::ENLARGE_HELD.store(c.key_held(&state::ENLARGE), Ordering::Relaxed);
        let now = Instant::now();
        let mut last = LAST_MINIMAP.lock().unwrap();
        if now.duration_since(*last) >= Duration::from_millis(300) {
            *last = now;
            drop(last);
            map::build_minimap(c);
        } else {
            drop(last);
        }
        if state::BIG_OPEN.load(Ordering::Relaxed) {
            let mut last_big = LAST_BIG.lock().unwrap();
            if now.duration_since(*last_big) >= Duration::from_millis(300) {
                *last_big = now;
                drop(last_big);
                map::build_big_map(c);
            }
        }
        let dead = st.flags & abi::client_flags::DEAD != 0;
        if dead && !WAS_DEAD.swap(dead, Ordering::Relaxed) {
            waypoints::add_death(
                st.pos.x.floor() as i32,
                st.pos.y.floor() as i32,
                st.pos.z.floor() as i32,
                weft_sdk::dimension_key(st.dim),
            );
        } else if !dead {
            WAS_DEAD.store(false, Ordering::Relaxed);
        }
        in_world_waypoints(c, &st);
    });
}

fn commands(r: &mut Registry) {
    r.client_command("map", "", |_| {
        let was_open = state::BIG_OPEN.swap(false, Ordering::Relaxed);
        if !was_open {
            open_big_map();
        }
    });
    r.client_command("minimap", "", |c| {
        c.client().menu_open(&state::SETTINGS_SCREEN);
    });
    r.client_command("waypoints", "", |c| {
        c.client().menu_open(&state::WAYPOINT_SCREEN);
    });
}

pub fn register(r: &mut Registry) {
    minimap_hud(r);
    big_map_hud(r);
    settings_screen(r);
    waypoint_screen(r);
    keys(r);
    commands(r);
    frame_hook(r);
}

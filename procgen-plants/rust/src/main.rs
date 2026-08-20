//! procgen-plants — rule-driven L-system plants growing across a landscape.
//!
//! Pipeline, in the order the modules run:
//!
//!   species.json  ->  species.rs   (data: grammar + growth numbers)
//!                 ->  lsystem.rs   (rewrite the grammar into a long string)
//!                 ->  turtle.rs    (walk the string, emit stems/leaves/flowers)
//!                 ->  world.rs     (terrain, placement, per-plant sprout dates)
//!                 ->  render.rs    (draw one week of the time-lapse)
//!
//! `species.json` is hot-reloaded: save the file and the landscape rebuilds.

mod lsystem;
mod render;
mod rng;
mod species;
mod turtle;
mod world;

use macroquad::prelude::*;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use species::Garden;
use world::Landscape;

fn window_conf() -> Conf {
    Conf {
        window_title: "procgen-plants".to_owned(),
        window_width: 1280,
        window_height: 760,
        high_dpi: true,
        ..Default::default()
    }
}

struct App {
    path: PathBuf,
    last_modified: Option<SystemTime>,
    garden: Garden,
    land: Landscape,
    week: f32,
    playing: bool,
    weeks_per_second: f32,
    error: Option<String>,
    status: String,
    status_until: f64,
    show_help: bool,
}

#[macroquad::main(window_conf)]
async fn main() {
    let path = locate_species_file();

    let (garden, error) = match load(&path) {
        Ok(g) => (g, None),
        Err(e) => (fallback_garden(), Some(e)),
    };

    let mut app = App {
        last_modified: modified_at(&path),
        land: Landscape::generate(&garden),
        garden,
        path,
        week: 0.0,
        playing: true,
        weeks_per_second: 2.5,
        error,
        status: String::new(),
        status_until: 0.0,
        show_help: true,
    };

    let mut reload_timer = 0.0f32;

    loop {
        let dt = get_frame_time().min(0.1);

        // --- hot reload -------------------------------------------------
        reload_timer += dt;
        if reload_timer > 0.4 {
            reload_timer = 0.0;
            if modified_at(&app.path) != app.last_modified {
                app.last_modified = modified_at(&app.path);
                match load(&app.path) {
                    Ok(g) => {
                        app.garden = g;
                        app.land = Landscape::generate(&app.garden);
                        app.error = None;
                        app.flash("species.json reloaded");
                    }
                    Err(e) => app.error = Some(e),
                }
            }
        }

        app.handle_input(dt);

        if app.playing {
            app.week += app.weeks_per_second * dt;
            if app.week > app.garden.world.total_weeks {
                app.week = 0.0;
            }
        }
        app.week = app.week.clamp(0.0, app.garden.world.total_weeks);

        render::draw_scene(&app.garden, &app.land, app.week);
        app.draw_hud();

        next_frame().await;
    }
}

impl App {
    fn flash(&mut self, msg: &str) {
        self.status = msg.to_string();
        self.status_until = get_time() + 2.5;
    }

    fn handle_input(&mut self, dt: f32) {
        if is_key_pressed(KeyCode::Space) {
            self.playing = !self.playing;
        }
        if is_key_pressed(KeyCode::H) {
            self.show_help = !self.show_help;
        }
        if is_key_pressed(KeyCode::R) {
            // Reseed: a brand new hillside from the same species rules.
            self.garden.world.seed = self.garden.world.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            self.land = Landscape::generate(&self.garden);
            self.flash(&format!("reseeded ({})", self.garden.world.seed));
        }
        if is_key_pressed(KeyCode::Key0) {
            self.week = 0.0;
        }
        if is_key_down(KeyCode::Right) {
            self.week += 6.0 * dt;
        }
        if is_key_down(KeyCode::Left) {
            self.week -= 6.0 * dt;
        }
        if is_key_pressed(KeyCode::Equal) {
            self.weeks_per_second = (self.weeks_per_second * 1.5).min(20.0);
        }
        if is_key_pressed(KeyCode::Minus) {
            self.weeks_per_second = (self.weeks_per_second / 1.5).max(0.2);
        }
        if is_key_pressed(KeyCode::Escape) {
            std::process::exit(0);
        }

        // Click or drag anywhere on the timeline strip to scrub.
        let h = screen_height();
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if my > h - 46.0 {
                self.playing = false;
                self.week = (mx / screen_width()).clamp(0.0, 1.0) * self.garden.world.total_weeks;
            }
        }
    }

    fn draw_hud(&self) {
        let w = screen_width();
        let h = screen_height();

        // Timeline strip.
        let bar_y = h - 30.0;
        draw_rectangle(0.0, h - 46.0, w, 46.0, render::with_alpha(render::SKY_TOP, 0.82));
        draw_rectangle(24.0, bar_y, w - 48.0, 3.0, render::with_alpha(render::INK_MUTED, 0.35));
        let progress = (self.week / self.garden.world.total_weeks.max(0.001)).clamp(0.0, 1.0);
        draw_rectangle(24.0, bar_y, (w - 48.0) * progress, 3.0, render::SUN);
        draw_circle(24.0 + (w - 48.0) * progress, bar_y + 1.5, 5.0, render::SUN);

        // Week readout.
        let label = format!(
            "week {:>4.1}   {}",
            self.week,
            render::calendar_label(self.week)
        );
        draw_text(&label, 24.0, h - 54.0, 22.0, render::INK);

        let right = format!(
            "{} plants   {} species   {:.1} wk/s   {}",
            self.land.plants.len(),
            self.garden.species.len(),
            self.weeks_per_second,
            if self.playing { "playing" } else { "paused" }
        );
        let dims = measure_text(&right, None, 18, 1.0);
        draw_text(&right, w - dims.width - 24.0, h - 54.0, 18.0, render::INK_MUTED);

        if self.show_help {
            let lines = [
                "space  play / pause",
                "< >    scrub weeks",
                "0      back to week 0",
                "+ -    speed",
                "R      reseed landscape",
                "H      hide this",
                "",
                "edit species.json — it reloads on save",
            ];
            let mut y = 34.0;
            for line in lines {
                draw_text(line, 24.0, y, 17.0, render::INK_MUTED);
                y += 20.0;
            }
        }

        if self.land.truncated {
            draw_text(
                "grammar hit the size budget — lower `iterations`",
                24.0,
                h - 78.0,
                17.0,
                Color::new(0.92, 0.72, 0.35, 1.0),
            );
        }

        if let Some(err) = &self.error {
            draw_rectangle(0.0, 0.0, w, 30.0, Color::new(0.45, 0.13, 0.13, 0.92));
            draw_text(&format!("species.json: {err}"), 12.0, 20.0, 18.0, WHITE);
        } else if get_time() < self.status_until {
            draw_text(&self.status, w * 0.5 - 80.0, 34.0, 18.0, render::SUN);
        }
    }
}

fn load(path: &Path) -> Result<Garden, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let garden: Garden = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if garden.species.is_empty() {
        return Err("no species defined".into());
    }
    Ok(garden)
}

fn modified_at(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// Look for `species.json` beside the working directory first, then beside the
/// crate. That way `cargo run` works from anywhere in the repo.
fn locate_species_file() -> PathBuf {
    let cwd = PathBuf::from("species.json");
    if cwd.exists() {
        return cwd;
    }
    let beside_crate = Path::new(env!("CARGO_MANIFEST_DIR")).join("species.json");
    if beside_crate.exists() {
        return beside_crate;
    }
    cwd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;
    use crate::turtle;

    /// The shipped `species.json` must parse and every species in it must
    /// produce a plant with actual extent — a typo in a rule usually shows up
    /// here as a zero-height plant long before it shows up on screen.
    #[test]
    fn shipped_species_file_builds_every_plant() {
        let garden = load(&locate_species_file()).expect("species.json should parse");
        assert!(!garden.species.is_empty());

        for sp in &garden.species {
            let mut rng = Rng::new(7);
            let geo = turtle::build(sp, &mut rng);
            assert!(!geo.segments.is_empty(), "{} drew no stems", sp.name);
            assert!(geo.height > 0.0, "{} has no height", sp.name);
            assert!(geo.max_order > 0.0, "{} has no growth path", sp.name);
            assert!(!geo.truncated, "{} blew the segment budget", sp.name);
            assert!(sp.mature_height > 0.0, "{} has no target height", sp.name);
        }
    }

    /// Placement must never silently drop a species the user is editing.
    #[test]
    fn every_species_gets_at_least_one_plant() {
        let garden = load(&locate_species_file()).expect("species.json should parse");
        let land = Landscape::generate(&garden);
        for (i, sp) in garden.species.iter().enumerate() {
            assert!(
                land.plants.iter().any(|p| p.species_index == i),
                "{} was left out of the landscape",
                sp.name
            );
        }
    }
}

/// Used only when `species.json` is missing or unparseable, so the window still
/// shows something instead of a black screen.
fn fallback_garden() -> Garden {
    let mut rules = std::collections::HashMap::new();
    rules.insert("X".to_string(), "F+[[X]-X]-F[-FX]+XL".to_string());
    rules.insert("F".to_string(), "FF".to_string());
    Garden {
        world: Default::default(),
        species: vec![species::Species {
            name: "fallback".into(),
            axiom: "X".into(),
            rules,
            iterations: 5,
            angle_deg: 24.0,
            angle_jitter_deg: 4.0,
            step_len: 6.0,
            len_falloff: 0.94,
            base_thickness: 1.6,
            thickness_taper: 0.85,
            leaf_size: 2.0,
            weeks_to_mature: 14.0,
            germination_week: 0.0,
            germination_spread: 3.0,
            flower_start: 0.7,
            stem_color: "#6B4A2F".into(),
            leaf_color: "#4E8C3A".into(),
            autumn_color: "#C4772F".into(),
            flower_color: "#E0A6C0".into(),
            weight: 1.0,
            mature_height: 0.3,
            height_variance: 0.2,
        }],
    }
}

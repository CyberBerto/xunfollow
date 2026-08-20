//! Drawing. Nothing here decides *what* a plant looks like — that all comes
//! from the species data and the geometry the turtle produced. This module
//! only maps plant-local coordinates onto the screen and picks colors.

use macroquad::prelude::*;

use crate::species::Garden;
use crate::turtle::reveal;
use crate::world::{smoothstep_range, species_of, Landscape};

// ---- palette -------------------------------------------------------------
// One coherent dusk-lab palette, shared with the browser preview.
pub const SKY_TOP: Color = Color::new(0.051, 0.098, 0.125, 1.0);
pub const SKY_BOTTOM: Color = Color::new(0.141, 0.208, 0.173, 1.0);
pub const HILL_FAR: Color = Color::new(0.086, 0.141, 0.114, 1.0);
pub const HILL_MID: Color = Color::new(0.110, 0.184, 0.145, 1.0);
pub const GROUND: Color = Color::new(0.141, 0.224, 0.173, 1.0);
pub const GROUND_EDGE: Color = Color::new(0.251, 0.400, 0.294, 1.0);
pub const SOIL: Color = Color::new(0.071, 0.110, 0.090, 1.0);
pub const SUN: Color = Color::new(0.863, 0.922, 0.776, 1.0);
pub const INK: Color = Color::new(0.894, 0.929, 0.906, 1.0);
pub const INK_MUTED: Color = Color::new(0.561, 0.627, 0.596, 1.0);

pub fn draw_scene(garden: &Garden, land: &Landscape, week: f32) {
    let w = screen_width();
    let h = screen_height();

    draw_sky(w, h);
    draw_sun(w, h, week, garden.world.total_weeks);
    hill_layer(land, w, h, 0.86, -0.09, HILL_FAR);
    draw_haze(w, h);
    hill_layer(land, w, h, 0.62, -0.045, HILL_MID);
    draw_ground(land, w, h);

    for plant in &land.plants {
        draw_plant(garden, land, plant, w, h, week);
    }
}

fn draw_sky(w: f32, h: f32) {
    const BANDS: usize = 56;
    let band_h = h / BANDS as f32;
    for i in 0..BANDS {
        let t = i as f32 / (BANDS - 1) as f32;
        draw_rectangle(0.0, i as f32 * band_h, w, band_h + 1.0, mix(SKY_TOP, SKY_BOTTOM, t));
    }
}

fn draw_sun(w: f32, h: f32, week: f32, total_weeks: f32) {
    // The sun tracks the year: low in week 0, high at midsummer, low again.
    let year = (week / total_weeks.max(1.0)).clamp(0.0, 1.0);
    let arc = (year * std::f32::consts::PI).sin();
    let x = w * (0.12 + 0.7 * year);
    let y = h * (0.52 - 0.34 * arc);

    for ring in (1..=6).rev() {
        draw_circle(x, y, h * 0.017 * ring as f32, with_alpha(SUN, 0.03));
    }
    draw_circle(x, y, h * 0.014, with_alpha(SUN, 0.7));
}

/// Haze along the skyline — the classic landscape depth cue, and it stops the
/// upper half of the frame reading as an empty gradient.
fn draw_haze(w: f32, h: f32) {
    const BANDS: usize = 32;
    let (top, bottom) = (h * 0.5, h * 0.85);
    let band_h = (bottom - top) / BANDS as f32;
    for i in 0..BANDS {
        let t = i as f32 / (BANDS - 1) as f32;
        // Peak opacity in the middle of the band, fading out at both edges.
        let a = (1.0 - (t - 0.55).abs() / 0.55).max(0.0) * 0.55;
        draw_rectangle(0.0, top + i as f32 * band_h, w, band_h + 1.0, with_alpha(SKY_BOTTOM, a));
    }
}

fn hill_layer(land: &Landscape, w: f32, h: f32, flatten: f32, lift: f32, color: Color) {
    const STEPS: usize = 120;
    let dx = w / STEPS as f32;
    for i in 0..STEPS {
        let u = i as f32 / STEPS as f32;
        let ground = land.ground_at(u * flatten + 0.37);
        let y = h * (ground + lift);
        draw_rectangle(i as f32 * dx, y, dx + 1.0, h - y, color);
    }
}

fn draw_ground(land: &Landscape, w: f32, h: f32) {
    const STEPS: usize = 260;
    const DEPTH_BANDS: usize = 24;
    let dx = w / STEPS as f32;

    for i in 0..STEPS {
        let u = i as f32 / STEPS as f32;
        let y = h * land.ground_at(u);

        // Earth darkens with depth, so the foreground reads as soil rather
        // than a flat slab of colour under the plants.
        let band_h = (h - y) / DEPTH_BANDS as f32;
        for b in 0..DEPTH_BANDS {
            let t = b as f32 / (DEPTH_BANDS - 1).max(1) as f32;
            draw_rectangle(
                i as f32 * dx,
                y + b as f32 * band_h,
                dx + 1.0,
                band_h + 1.0,
                mix(GROUND, SOIL, t),
            );
        }
        // A grass-lit rim along the skyline, kept close in value to the ground.
        draw_rectangle(i as f32 * dx, y, dx + 1.0, 1.5, mix(GROUND, GROUND_EDGE, 0.45));
    }
}

fn draw_plant(
    garden: &Garden,
    land: &Landscape,
    plant: &crate::world::Plant,
    w: f32,
    h: f32,
    week: f32,
) {
    let growth = plant.growth(week);
    if growth <= 0.0 {
        return;
    }

    let species = species_of(garden, plant);
    let geo = &plant.geometry;

    // Growth clock: how far along the longest branch path we have travelled.
    let t = growth * (geo.max_order + 1.0);

    // Species are authored by the height they reach, not by pixel step sizes:
    // normalise the raw geometry so a mature plant lands on its target height.
    // That keeps `step_len` and `len_falloff` purely about *shape*.
    let norm = (h * plant.mature_height) / geo.height.max(0.001);
    // A seedling is not just a partial adult — it is also physically smaller.
    let size_ramp = 0.30 + 0.70 * growth;
    let depth_scale = 1.0 - 0.42 * plant.depth;
    let scale = norm * size_ramp * depth_scale;

    let base_x = plant.u * w;
    let base_y = h * land.ground_at(plant.u) - h * 0.045 * plant.depth;
    let flip = if plant.flip { -1.0 } else { 1.0 };

    // Aerial perspective: distant plants fade toward the sky behind them.
    let haze = plant.depth * 0.45;
    let stem = mix(species.stem(), SKY_BOTTOM, haze);

    // Leaves shift toward their autumn color late in the year.
    let autumn = smoothstep_range(
        garden.world.autumn_week,
        garden.world.autumn_week + 6.0,
        week,
    );
    let leaf = mix(mix(species.leaf(), species.autumn(), autumn), SKY_BOTTOM, haze);
    let flower_color = mix(species.flower(), SKY_BOTTOM, haze);

    let to_screen = |x: f32, y: f32| (base_x + x * flip * scale, base_y - y * scale);

    for seg in &geo.segments {
        let Some(frac) = reveal(seg.order, t) else { continue };
        let (x0, y0) = to_screen(seg.x0, seg.y0);
        let (x1, y1) = to_screen(
            seg.x0 + (seg.x1 - seg.x0) * frac,
            seg.y0 + (seg.y1 - seg.y0) * frac,
        );
        draw_line(x0, y0, x1, y1, (seg.thickness * scale).max(0.7), stem);
    }

    for m in &geo.leaves {
        let Some(open) = reveal(m.order, t) else { continue };
        let (x, y) = to_screen(m.x, m.y);
        draw_leaf(x, y, m.angle * flip, m.size * scale * open, leaf);
    }

    if growth > species.flower_start {
        let bloom = smoothstep_range(species.flower_start, (species.flower_start + 0.22).min(1.0), growth);
        // Petals wither back a little once autumn sets in.
        let bloom = bloom * (1.0 - 0.75 * autumn);
        for m in &geo.flowers {
            if reveal(m.order, t).is_none() {
                continue;
            }
            let (x, y) = to_screen(m.x, m.y);
            draw_flower(x, y, m.size * scale * bloom, flower_color);
        }
    }
}

/// A leaf is a lens shape: two triangles sharing the stem axis.
fn draw_leaf(x: f32, y: f32, angle: f32, size: f32, color: Color) {
    if size < 0.4 {
        return;
    }
    let (sx, cy) = (angle.sin(), angle.cos());
    let tip = vec2(x + sx * size * 2.2, y - cy * size * 2.2);
    let mid = vec2(x + sx * size * 1.1, y - cy * size * 1.1);
    // Perpendicular to the stem direction.
    let perp = vec2(cy, sx) * size * 0.85;
    let base = vec2(x, y);
    draw_triangle(base, mid + perp, tip, color);
    draw_triangle(base, mid - perp, tip, color);
}

fn draw_flower(x: f32, y: f32, size: f32, color: Color) {
    if size < 0.4 {
        return;
    }
    for i in 0..5 {
        let a = i as f32 * std::f32::consts::TAU / 5.0;
        draw_circle(x + a.cos() * size * 0.62, y + a.sin() * size * 0.62, size * 0.55, color);
    }
    draw_circle(x, y, size * 0.34, with_alpha(SUN, 0.85));
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

/// Turn a week number into a readable "Month, week N" label.
pub fn calendar_label(week: f32) -> String {
    const MONTHS: [&str; 12] = [
        "March", "April", "May", "June", "July", "August", "September", "October",
        "November", "December", "January", "February",
    ];
    let total_days = week * 7.0;
    let month = ((total_days / 30.4) as usize).min(MONTHS.len() - 1);
    let day = (total_days - month as f32 * 30.4).max(0.0) as u32 + 1;
    format!("{} {}", MONTHS[month], day.min(30))
}

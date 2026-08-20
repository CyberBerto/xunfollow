//! Species definitions — the "rules" half of the project.
//!
//! A species is pure data: an L-system grammar plus the numbers that describe
//! how to draw and how fast it grows. Everything here is loaded from
//! `species.json` at runtime, so adding a new plant never requires touching
//! Rust code.

use macroquad::prelude::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level contents of `species.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Garden {
    #[serde(default)]
    pub world: World,
    pub species: Vec<Species>,
}

/// Global settings for the landscape itself.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    /// Changing this regenerates terrain and plant placement.
    #[serde(default = "d_seed")]
    pub seed: u32,
    /// How many weeks the timeline covers.
    #[serde(default = "d_total_weeks")]
    pub total_weeks: f32,
    /// How many plants to scatter across the hillside.
    #[serde(default = "d_plant_count")]
    pub plant_count: usize,
    /// 0.0 = flat plain, 1.0 = dramatic hills.
    #[serde(default = "d_roughness")]
    pub terrain_roughness: f32,
    /// Week at which leaves start turning. Set above `total_weeks` to disable.
    #[serde(default = "d_autumn_week")]
    pub autumn_week: f32,
}

impl Default for World {
    fn default() -> Self {
        World {
            seed: d_seed(),
            total_weeks: d_total_weeks(),
            plant_count: d_plant_count(),
            terrain_roughness: d_roughness(),
            autumn_week: d_autumn_week(),
        }
    }
}

/// One plant species.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Species {
    pub name: String,

    // ---- the grammar ----
    /// The starting string, e.g. `"X"`.
    pub axiom: String,
    /// Replacement rules, e.g. `{"X": "F[+X]F[-X]+X", "F": "FF"}`.
    pub rules: HashMap<String, String>,
    /// How many rewrite passes to run. Cost grows fast — 4 to 6 is the sweet spot.
    #[serde(default = "d_iterations")]
    pub iterations: u32,

    // ---- turtle interpretation ----
    /// Degrees turned by `+` and `-`.
    #[serde(default = "d_angle")]
    pub angle_deg: f32,
    /// Random wobble added to every turn, so no two plants are identical.
    #[serde(default)]
    pub angle_jitter_deg: f32,
    /// Length of one `F` step. These are *relative* units: the finished plant
    /// is rescaled to `mature_height`, so only the ratios here matter.
    #[serde(default = "d_step_len")]
    pub step_len: f32,
    /// Each `[` shrinks subsequent steps by this factor (0.9 = 10% shorter).
    /// Wrapping a rule's recursion in brackets is how a stem tapers.
    #[serde(default = "d_falloff")]
    pub len_falloff: f32,
    /// Stem width at the base, in the same relative units as `step_len`.
    #[serde(default = "d_thickness")]
    pub base_thickness: f32,
    /// Stem width multiplier per branch level.
    #[serde(default = "d_taper")]
    pub thickness_taper: f32,
    /// Size of a leaf drawn by `L`, in the same relative units.
    #[serde(default = "d_leaf_size")]
    pub leaf_size: f32,

    // ---- growth over time ----
    /// Weeks from sprouting to full size.
    #[serde(default = "d_maturity")]
    pub weeks_to_mature: f32,
    /// Earliest week this species sprouts.
    #[serde(default)]
    pub germination_week: f32,
    /// Individual plants sprout randomly within this many weeks after that.
    #[serde(default = "d_germ_spread")]
    pub germination_spread: f32,
    /// Fraction of growth (0-1) after which `O` symbols bloom.
    #[serde(default = "d_flower_start")]
    pub flower_start: f32,

    // ---- appearance ----
    #[serde(default = "d_stem_color")]
    pub stem_color: String,
    #[serde(default = "d_leaf_color")]
    pub leaf_color: String,
    #[serde(default = "d_autumn_color")]
    pub autumn_color: String,
    #[serde(default = "d_flower_color")]
    pub flower_color: String,

    /// Relative likelihood of this species appearing in the landscape.
    #[serde(default = "d_weight")]
    pub weight: f32,
    /// Height of a mature plant, as a fraction of the view height. The raw
    /// turtle geometry is normalised to hit this, which is why `step_len` and
    /// `len_falloff` above are purely about *shape*.
    #[serde(default = "d_mature_height")]
    pub mature_height: f32,
    /// How much that height varies between individuals.
    #[serde(default = "d_height_variance")]
    pub height_variance: f32,
}

impl Species {
    pub fn stem(&self) -> Color {
        parse_hex(&self.stem_color, Color::new(0.42, 0.30, 0.20, 1.0))
    }
    pub fn leaf(&self) -> Color {
        parse_hex(&self.leaf_color, Color::new(0.30, 0.55, 0.24, 1.0))
    }
    pub fn autumn(&self) -> Color {
        parse_hex(&self.autumn_color, Color::new(0.77, 0.47, 0.18, 1.0))
    }
    pub fn flower(&self) -> Color {
        parse_hex(&self.flower_color, Color::new(0.88, 0.65, 0.75, 1.0))
    }
}

/// Parse `"#RRGGBB"` (or `"#RGB"`), falling back to `default` on anything odd.
pub fn parse_hex(s: &str, default: Color) -> Color {
    let h = s.trim().trim_start_matches('#');
    let expand = |c: char| -> Option<u8> { c.to_digit(16).map(|d| (d * 17) as u8) };

    let (r, g, b) = match h.len() {
        3 => {
            let mut it = h.chars();
            match (
                it.next().and_then(expand),
                it.next().and_then(expand),
                it.next().and_then(expand),
            ) {
                (Some(r), Some(g), Some(b)) => (r, g, b),
                _ => return default,
            }
        }
        6 => match (
            u8::from_str_radix(&h[0..2], 16),
            u8::from_str_radix(&h[2..4], 16),
            u8::from_str_radix(&h[4..6], 16),
        ) {
            (Ok(r), Ok(g), Ok(b)) => (r, g, b),
            _ => return default,
        },
        _ => return default,
    };

    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

// Defaults keep `species.json` terse: omit a field and you get a sane value.
fn d_seed() -> u32 { 1_337 }
fn d_total_weeks() -> f32 { 40.0 }
fn d_plant_count() -> usize { 9 }
fn d_roughness() -> f32 { 0.55 }
fn d_autumn_week() -> f32 { 30.0 }
fn d_iterations() -> u32 { 5 }
fn d_angle() -> f32 { 22.5 }
fn d_step_len() -> f32 { 4.0 }
fn d_falloff() -> f32 { 0.92 }
fn d_thickness() -> f32 { 1.5 }
fn d_taper() -> f32 { 0.86 }
fn d_leaf_size() -> f32 { 2.0 }
fn d_maturity() -> f32 { 14.0 }
fn d_germ_spread() -> f32 { 3.0 }
fn d_flower_start() -> f32 { 0.7 }
fn d_stem_color() -> String { "#6B4A2F".into() }
fn d_leaf_color() -> String { "#4E8C3A".into() }
fn d_autumn_color() -> String { "#C4772F".into() }
fn d_flower_color() -> String { "#E0A6C0".into() }
fn d_weight() -> f32 { 1.0 }
fn d_mature_height() -> f32 { 0.25 }
fn d_height_variance() -> f32 { 0.2 }

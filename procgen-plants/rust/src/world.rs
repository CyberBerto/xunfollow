//! The landscape: terrain shape, where plants stand, and when each one sprouts.
//!
//! Everything here is derived from one integer seed, so the same seed always
//! produces the same hillside with the same plants in the same places.

use crate::rng::{fbm_1d, Rng};
use crate::species::{Garden, Species};
use crate::turtle::{self, Geometry};

pub struct Plant {
    pub species_index: usize,
    /// Horizontal position as a fraction of the view width.
    pub u: f32,
    /// 0.0 = nearest the viewer, 1.0 = furthest back.
    pub depth: f32,
    /// Mature height for this individual, as a fraction of the view height.
    pub mature_height: f32,
    /// Week this individual breaks ground.
    pub germination_week: f32,
    /// Weeks from sprouting to full size (varies slightly per individual).
    pub weeks_to_mature: f32,
    /// Mirror the plant horizontally, so repeated grammars read as different.
    pub flip: bool,
    pub geometry: Geometry,
}

impl Plant {
    /// Growth in `0.0..=1.0` at a given week, eased so plants ramp up and
    /// settle rather than growing at a constant rate.
    pub fn growth(&self, week: f32) -> f32 {
        let raw = ((week - self.germination_week) / self.weeks_to_mature.max(0.1)).clamp(0.0, 1.0);
        smoothstep(raw)
    }
}

pub struct Landscape {
    pub seed: u32,
    pub roughness: f32,
    pub plants: Vec<Plant>,
    /// Set when any plant hit the segment or string budget.
    pub truncated: bool,
}

impl Landscape {
    /// Regrow the entire world from scratch.
    pub fn generate(garden: &Garden) -> Landscape {
        let world = &garden.world;
        let mut rng = Rng::new(world.seed);
        let mut plants = Vec::new();
        let mut truncated = false;

        if garden.species.is_empty() {
            return Landscape { seed: world.seed, roughness: world.terrain_roughness, plants, truncated };
        }

        let weights: Vec<f32> = garden.species.iter().map(|s| s.weight).collect();
        let count = world.plant_count.clamp(1, 60);

        // Give every species at least one individual before filling the rest in
        // by weight — otherwise an unlucky seed silently drops a species you
        // just finished editing, which is baffling while tuning rules.
        let mut assignment: Vec<usize> = (0..count)
            .map(|i| if i < garden.species.len() { i } else { rng.weighted_index(&weights) })
            .collect();
        // Shuffle so the guaranteed ones are not all bunched on the left.
        for i in (1..assignment.len()).rev() {
            let j = (rng.unit() * (i + 1) as f32) as usize;
            assignment.swap(i, j.min(i));
        }

        for i in 0..count {
            let idx = assignment[i].min(garden.species.len() - 1);
            let species = &garden.species[idx];

            // Spread plants across the width in jittered slots so they do not
            // clump the way pure uniform random would.
            let slot = (i as f32 + 0.5) / count as f32;
            let u = (slot + rng.jitter(0.45 / count as f32)).clamp(0.02, 0.98);

            let depth = rng.unit().powf(1.4); // bias toward the foreground
            let mature_height =
                (species.mature_height * (1.0 + rng.jitter(species.height_variance))).max(0.01);

            let germination_week = species.germination_week + rng.unit() * species.germination_spread;
            let weeks_to_mature = species.weeks_to_mature * (1.0 + rng.jitter(0.15));

            // Each plant gets its own generator so its random turn-wobble is
            // stable, and independent of how many plants came before it.
            let mut plant_rng = Rng::new(world.seed ^ ((i as u32 + 1).wrapping_mul(0x9E37_79B9)));
            let geometry = turtle::build(species, &mut plant_rng);
            truncated |= geometry.truncated;

            plants.push(Plant {
                species_index: idx,
                u,
                depth,
                mature_height,
                germination_week,
                weeks_to_mature,
                flip: rng.unit() < 0.5,
                geometry,
            });
        }

        // Draw furthest first so foreground plants overlap correctly.
        plants.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        Landscape { seed: world.seed, roughness: world.terrain_roughness, plants, truncated }
    }

    /// Ground height at horizontal fraction `u`, as a fraction of view height
    /// measured from the top (so a larger number is lower on screen).
    pub fn ground_at(&self, u: f32) -> f32 {
        let n = fbm_1d(u * 3.2 + 11.0, self.seed, 4) - 0.5;
        0.80 - 0.13 * self.roughness * n * 2.0
    }
}

pub fn species_of<'a>(garden: &'a Garden, plant: &Plant) -> &'a Species {
    &garden.species[plant.species_index.min(garden.species.len().saturating_sub(1))]
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linear ramp from 0 at `edge0` to 1 at `edge1`, smoothed.
pub fn smoothstep_range(edge0: f32, edge1: f32, x: f32) -> f32 {
    if (edge1 - edge0).abs() < f32::EPSILON {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    smoothstep((x - edge0) / (edge1 - edge0))
}

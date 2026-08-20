//! Turtle graphics: turn an L-system string into drawable geometry.
//!
//! We walk the string with a pen ("turtle") that has a position and a heading.
//!
//!   F  step forward, drawing a stem segment
//!   f  step forward without drawing
//!   +  turn left      -  turn right
//!   [  save position/heading (start a branch)
//!   ]  restore it     (end the branch)
//!   L  drop a leaf here
//!   O  drop a flower/bud here
//!   anything else is a no-op placeholder (X, A, B ... ) that only exists to
//!   drive the rewriting.
//!
//! Coordinates are plant-local: the base sits at (0, 0) and +y points **up**.
//! The renderer flips y, so this module never has to think about screens.
//!
//! The key trick for time-lapse growth is `order`: every segment records how
//! many forward steps came before it along its own branch. Revealing the plant
//! in increasing `order` makes it extend outward from the base, tip by tip,
//! exactly the way a real plant does.

use crate::lsystem;
use crate::rng::Rng;
use crate::species::Species;

pub struct Segment {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub thickness: f32,
    /// Forward steps preceding this one along its branch.
    pub order: f32,
}

pub struct Marker {
    pub x: f32,
    pub y: f32,
    /// Heading of the stem it is attached to, in radians.
    pub angle: f32,
    pub size: f32,
    pub order: f32,
}

#[derive(Default)]
pub struct Geometry {
    pub segments: Vec<Segment>,
    pub leaves: Vec<Marker>,
    pub flowers: Vec<Marker>,
    /// Highest `order` in the plant — the length of its longest growth path.
    pub max_order: f32,
    /// Bounding box in local space, useful for framing.
    pub height: f32,
    pub truncated: bool,
}

/// Number of segments we are willing to draw for one plant.
const MAX_SEGMENTS: usize = 12_000;

#[derive(Clone, Copy)]
struct State {
    x: f32,
    y: f32,
    /// Heading in radians. 0 points straight up; positive turns clockwise.
    angle: f32,
    len: f32,
    thickness: f32,
    order: f32,
}

pub fn build(species: &Species, rng: &mut Rng) -> Geometry {
    let rules = lsystem::compile_rules(&species.rules);
    let expansion = lsystem::expand(&species.axiom, &rules, species.iterations);
    interpret(&expansion.string, species, rng, expansion.truncated)
}

/// Walk an already-expanded string and emit geometry.
pub fn interpret(s: &str, species: &Species, rng: &mut Rng, truncated: bool) -> Geometry {
    let mut geo = Geometry { truncated, ..Default::default() };

    let turn = species.angle_deg.to_radians();
    let jitter = species.angle_jitter_deg.to_radians();

    let mut st = State {
        x: 0.0,
        y: 0.0,
        angle: 0.0,
        len: species.step_len.max(0.05),
        thickness: species.base_thickness.max(0.2),
        order: 0.0,
    };
    let mut stack: Vec<State> = Vec::new();
    let mut max_y: f32 = 0.0;

    for c in s.chars() {
        match c {
            'F' | 'G' => {
                let (dx, dy) = (st.angle.sin() * st.len, st.angle.cos() * st.len);
                let (x1, y1) = (st.x + dx, st.y + dy);

                if geo.segments.len() < MAX_SEGMENTS {
                    geo.segments.push(Segment {
                        x0: st.x,
                        y0: st.y,
                        x1,
                        y1,
                        thickness: st.thickness,
                        order: st.order,
                    });
                } else {
                    geo.truncated = true;
                }

                st.x = x1;
                st.y = y1;
                st.order += 1.0;
                max_y = max_y.max(y1);
                geo.max_order = geo.max_order.max(st.order);
            }
            'f' => {
                st.x += st.angle.sin() * st.len;
                st.y += st.angle.cos() * st.len;
                st.order += 1.0;
                geo.max_order = geo.max_order.max(st.order);
            }
            // Screen y is flipped later, so "turn left" is a negative angle here.
            '+' => st.angle -= turn + rng.jitter(jitter),
            '-' => st.angle += turn + rng.jitter(jitter),
            // A 180° flip, occasionally useful in hand-written grammars.
            '|' => st.angle += std::f32::consts::PI,
            '[' => {
                stack.push(st);
                // Everything past a branch point is shorter and thinner.
                st.len *= species.len_falloff;
                st.thickness = (st.thickness * species.thickness_taper).max(0.35);
            }
            ']' => {
                // A branch is a side excursion: the main stem resumes exactly
                // where it left off, including its place on the growth clock.
                // That is what makes trunk and branches extend together.
                if let Some(prev) = stack.pop() {
                    st = prev;
                }
            }
            'L' => geo.leaves.push(Marker {
                x: st.x,
                y: st.y,
                angle: st.angle,
                size: species.leaf_size,
                order: st.order,
            }),
            'O' => geo.flowers.push(Marker {
                x: st.x,
                y: st.y,
                angle: st.angle,
                size: species.leaf_size * 0.9,
                order: st.order,
            }),
            _ => {}
        }
    }

    geo.height = max_y;
    geo
}

/// How much of a segment is visible at growth clock `t`.
///
/// Returns `None` if the segment has not started growing yet, otherwise a
/// fraction in `(0, 1]` of its full length.
pub fn reveal(order: f32, t: f32) -> Option<f32> {
    let f = t - order;
    if f <= 0.0 {
        None
    } else {
        Some(f.min(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::Species;
    use std::collections::HashMap;

    fn bare_species() -> Species {
        Species {
            name: "test".into(),
            axiom: "F".into(),
            rules: HashMap::new(),
            iterations: 0,
            angle_deg: 90.0,
            angle_jitter_deg: 0.0,
            step_len: 10.0,
            len_falloff: 1.0,
            base_thickness: 2.0,
            thickness_taper: 1.0,
            leaf_size: 3.0,
            weeks_to_mature: 10.0,
            germination_week: 0.0,
            germination_spread: 0.0,
            flower_start: 0.7,
            stem_color: "#000000".into(),
            leaf_color: "#000000".into(),
            autumn_color: "#000000".into(),
            flower_color: "#000000".into(),
            weight: 1.0,
            mature_height: 0.3,
            height_variance: 0.0,
        }
    }

    #[test]
    fn single_step_goes_straight_up() {
        let mut rng = Rng::new(1);
        let geo = interpret("F", &bare_species(), &mut rng, false);
        assert_eq!(geo.segments.len(), 1);
        let s = &geo.segments[0];
        assert!(s.x1.abs() < 1e-4);
        assert!((s.y1 - 10.0).abs() < 1e-4);
    }

    #[test]
    fn brackets_restore_position() {
        let mut rng = Rng::new(1);
        let geo = interpret("F[+F]F", &bare_species(), &mut rng, false);
        assert_eq!(geo.segments.len(), 3);
        // The third segment continues straight up from the first, not from
        // the branch, which is the whole point of the bracket stack.
        assert!(geo.segments[2].x0.abs() < 1e-4);
        assert!((geo.segments[2].y0 - 10.0).abs() < 1e-4);
    }

    #[test]
    fn reveal_clamps_to_one() {
        assert_eq!(reveal(0.0, -1.0), None);
        assert_eq!(reveal(3.0, 3.4).map(|v| (v * 10.0).round()), Some(4.0));
        assert_eq!(reveal(0.0, 99.0), Some(1.0));
    }
}

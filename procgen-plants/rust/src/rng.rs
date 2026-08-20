//! A tiny deterministic random number generator.
//!
//! We deliberately avoid the `rand` crate: everything in this project must be
//! reproducible from a single integer seed, so the same seed always regrows the
//! same landscape. This is a PCG-style xorshift, small enough to port anywhere
//! (the JavaScript preview uses the exact same algorithm).

#[derive(Clone)]
pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        // Avoid a zero state, which would make the generator stick at zero.
        Rng {
            state: seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453) | 1,
        }
    }

    /// Next raw 32-bit value (mulberry32).
    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x6D2B_79F5);
        let mut z = self.state;
        z = (z ^ (z >> 15)).wrapping_mul(z | 1);
        z ^= z.wrapping_add((z ^ (z >> 7)).wrapping_mul(z | 61));
        z ^ (z >> 14)
    }

    /// Uniform in `[0.0, 1.0)`.
    pub fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / 4_294_967_296.0
    }

    /// Uniform in `[lo, hi)`.
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.unit() * (hi - lo)
    }

    /// Uniform in `[-amount, amount)`.
    pub fn jitter(&mut self, amount: f32) -> f32 {
        self.range(-amount, amount)
    }

    /// Pick an index from a list of non-negative weights.
    pub fn weighted_index(&mut self, weights: &[f32]) -> usize {
        let total: f32 = weights.iter().map(|w| w.max(0.0)).sum();
        if total <= 0.0 {
            return 0;
        }
        let mut target = self.unit() * total;
        for (i, w) in weights.iter().enumerate() {
            target -= w.max(0.0);
            if target <= 0.0 {
                return i;
            }
        }
        weights.len() - 1
    }
}

/// Smooth 1D value noise, used for the terrain silhouette.
///
/// `x` is a continuous coordinate; integer positions get a pseudo-random height
/// and everything between is smoothly interpolated.
pub fn value_noise_1d(x: f32, seed: u32) -> f32 {
    let i = x.floor();
    let f = x - i;
    let a = hash_to_unit(i as i32, seed);
    let b = hash_to_unit(i as i32 + 1, seed);
    // Smoothstep the blend so the terrain has no visible creases.
    let t = f * f * (3.0 - 2.0 * f);
    a + (b - a) * t
}

/// Layered value noise: each octave is half the amplitude and double the frequency.
pub fn fbm_1d(x: f32, seed: u32, octaves: u32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut norm = 0.0;
    for o in 0..octaves {
        total += value_noise_1d(x * frequency, seed.wrapping_add(o * 977)) * amplitude;
        norm += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    total / norm
}

fn hash_to_unit(i: i32, seed: u32) -> f32 {
    let mut h = (i as u32).wrapping_mul(374_761_393).wrapping_add(seed.wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) as f32) / 4_294_967_296.0
}

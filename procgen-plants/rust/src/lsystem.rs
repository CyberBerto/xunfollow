//! The L-system itself: string rewriting.
//!
//! This is the whole idea in one function. Start with the axiom, and on every
//! pass replace *every* symbol simultaneously with whatever the rules say. A
//! symbol with no rule maps to itself.
//!
//!   axiom "A", rules { A -> AB, B -> A }
//!   pass 1: AB
//!   pass 2: ABA
//!   pass 3: ABAAB
//!
//! Growth is exponential, so we carry a hard length budget: an accidental
//! `F -> FFFF` at 6 iterations would otherwise try to allocate gigabytes.

use std::collections::HashMap;

/// Longest string we are willing to build for a single plant.
pub const MAX_LEN: usize = 400_000;

/// Result of expanding a grammar, including whether we hit the budget.
pub struct Expansion {
    pub string: String,
    /// Iterations actually completed before the budget stopped us.
    pub iterations_done: u32,
    pub truncated: bool,
}

/// Run `iterations` rewrite passes over `axiom`.
pub fn expand(axiom: &str, rules: &HashMap<char, String>, iterations: u32) -> Expansion {
    let mut current = axiom.to_string();
    let mut done = 0;

    for _ in 0..iterations {
        // Predict the size of the next pass before building it. If it would
        // blow the budget we stop cleanly at the last good iteration rather
        // than producing a half-rewritten (and therefore malformed) string.
        let projected: usize = current
            .chars()
            .map(|c| rules.get(&c).map_or(1, |r| r.len()))
            .sum();

        if projected > MAX_LEN {
            return Expansion { string: current, iterations_done: done, truncated: true };
        }

        let mut next = String::with_capacity(projected);
        for c in current.chars() {
            match rules.get(&c) {
                Some(replacement) => next.push_str(replacement),
                None => next.push(c),
            }
        }
        current = next;
        done += 1;
    }

    Expansion { string: current, iterations_done: done, truncated: false }
}

/// Convert the JSON rule table (string keys) into the char-keyed map we use.
/// Multi-character keys are ignored — this is a context-free, single-symbol
/// L-system, which is all we need for plants.
pub fn compile_rules(raw: &HashMap<String, String>) -> HashMap<char, String> {
    let mut out = HashMap::new();
    for (k, v) in raw {
        let mut chars = k.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            out.insert(c, v.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(pairs: &[(char, &str)]) -> HashMap<char, String> {
        pairs.iter().map(|(c, s)| (*c, s.to_string())).collect()
    }

    #[test]
    fn lindenmayers_algae() {
        let r = rules(&[('A', "AB"), ('B', "A")]);
        assert_eq!(expand("A", &r, 0).string, "A");
        assert_eq!(expand("A", &r, 1).string, "AB");
        assert_eq!(expand("A", &r, 2).string, "ABA");
        assert_eq!(expand("A", &r, 3).string, "ABAAB");
        assert_eq!(expand("A", &r, 5).string, "ABAABABAABAAB");
    }

    #[test]
    fn symbols_without_rules_pass_through() {
        let r = rules(&[('F', "FF")]);
        assert_eq!(expand("F[+F]", &r, 1).string, "FF[+FF]");
    }

    #[test]
    fn budget_stops_runaway_growth() {
        let r = rules(&[('F', "FFFF")]);
        let e = expand("F", &r, 30);
        assert!(e.truncated);
        assert!(e.string.len() <= MAX_LEN);
        assert!(e.iterations_done < 30);
    }
}

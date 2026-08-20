# procgen-plants

Rule-driven plants growing across a procedural hillside, as a time-lapse over
weeks and months. Every species is an **L-system grammar** stored as data, so
adding a plant means writing three lines of rules — never touching the renderer.

Two builds of the same idea:

| | what it is | how to run |
|---|---|---|
| `rust/` | the real project — native, hot-reloads `species.json` | `cd rust && cargo run` |
| `web/` | a browser mirror with a live rule editor | open `web/index.html` |

They share one algorithm and one seeded RNG, so **the same seed grows the same
hillside in both**. Tune rules in the browser, hit *Copy species.json*, paste
into `rust/species.json`, and the native build picks it up on save.

---

## What an L-system is

Invented in 1968 by the biologist Aristid Lindenmayer to model how plants grow.
Three parts:

1. **Alphabet** — the symbols, e.g. `F + - [ ]`
2. **Axiom** — the starting string, e.g. `X`
3. **Rules** — how each symbol is replaced, *all at once*, on every pass

His original algae example, `A → AB`, `B → A`, starting from `A`:

```
pass 0   A
pass 1   AB
pass 2   ABA
pass 3   ABAAB
pass 4   ABAABABA
```

The string is then read as **turtle graphics** — a pen with a position and a
heading walks the string one symbol at a time:

| symbol | meaning |
|---|---|
| `F` | step forward, drawing a stem segment |
| `f` | step forward without drawing |
| `+` `-` | turn left / right by the species' angle |
| `[` `]` | push / pop position and heading — this is what makes branches |
| `L` | drop a leaf here |
| `O` | drop a flower or bud here |
| `\|` | turn 180° |
| anything else | no-op placeholder (`X`, `A`, …) that only drives rewriting |

`[` and `]` are the whole trick. `F[+F]F` means: draw a stem, *save your spot*,
draw a branch off to the left, *jump back*, keep going up the main stem.

---

## How growth over time works

The plant is expanded **once**, at full size. What changes over time is how much
of it is revealed.

Every segment records its `order` — how many forward steps came before it along
its own branch. A growth clock `t` sweeps from 0 upward; a segment with order
`o` is invisible while `t ≤ o`, partially drawn while `o < t < o+1`, and
complete after that. Because branches inherit the order of the point they hang
off, trunk and branches extend together and the plant grows outward from the
base, tip by tip — the way a real one does.

On top of that, each plant:

- **sprouts on its own week** (`germination_week` + a random spread), so the
  hillside fills in gradually instead of all at once;
- **is physically smaller when young**, not just partially drawn;
- **turns colour after `autumn_week`**, and its flowers wither back.

---

## Adding a species

Add an object to `species` in `rust/species.json` (or click **Add** in the web
editor). Only `name`, `axiom` and `rules` are required — everything else has a
default.

```json
{
  "name": "Thistle",
  "axiom": "X",
  "rules": { "X": "F[+X][-X]FO", "F": "FF" },
  "iterations": 4,
  "angle_deg": 28,
  "mature_height": 0.3,
  "flower_start": 0.6
}
```

Fields worth knowing:

- **`mature_height`** — how tall a grown plant is, as a fraction of the view.
  The raw geometry is normalised to hit this, which is why `step_len` and
  `len_falloff` below are purely about *shape*, not size.
- **`len_falloff`** — how much shorter everything gets past a `[`. Wrapping a
  rule's own recursion in brackets (`X → …[X]`) is how you make a stem taper:
  every cycle becomes one branch level deeper. That is exactly what the
  Foxglove does.
- **`angle_jitter_deg`** — random wobble on every turn, so no two individuals
  are identical.
- **`iterations`** — cost is exponential. 4–6 is the useful range for a rule
  like `F → FF`; the Foxglove can afford 16 because its rule only grows the
  string linearly.

Both builds cap string length and segment count and will tell you when a
grammar hits the ceiling, rather than freezing.

---

## Code map

```
species.json  →  species.rs   data: grammar + growth numbers
              →  lsystem.rs   rewrite the grammar into one long string
              →  turtle.rs    walk the string, emit stems / leaves / flowers
              →  world.rs     terrain, placement, per-plant sprout dates
              →  render.rs    draw one week of the time-lapse
```

`rng.rs` is a small deterministic generator (mulberry32) plus 1D value noise for
the terrain. It is deliberately not the `rand` crate: the whole world must be
reproducible from one integer, and the JavaScript build must produce bit-alike
results.

Run the tests with `cd rust && cargo test`. They cover the rewriting rules
(including Lindenmayer's algae), the bracket stack, the runaway-growth budget,
and a check that the shipped `species.json` actually builds every plant.

---

## Native controls

| key | |
|---|---|
| `space` | play / pause |
| `←` `→` | scrub weeks |
| `0` | back to week 0 |
| `+` `-` | playback speed |
| `R` | reseed the hillside |
| `H` | hide the help overlay |

Click or drag the timeline strip to scrub. Editing `species.json` reloads it
live; a JSON error shows in a bar at the top instead of crashing.

---

## Where to take it next

Roughly in order of effort, each one a self-contained thing to hand-build:

1. **Parametric L-systems** — let symbols carry values, e.g. `F(2.5)`, so
   length and angle live in the grammar instead of in the species fields.
2. **Stochastic rules** — several replacements for one symbol with weights, so
   a single species varies structurally rather than only by jitter.
3. **Context-sensitive rules** — a symbol rewrites differently depending on its
   neighbours. This is what lets signals (water, hormones) travel up a plant.
4. **Environment response** — bend branches toward the sun's current position,
   or shorten growth where plants crowd each other.
5. **Real seasons** — leaf drop, dormancy over winter, a second year of growth
   layered on last year's wood.
6. **3D** — the turtle gains a full orientation frame (`&` `^` pitch, `\` `/`
   roll). The grammar and growth clock carry over unchanged; only `turtle.rs`
   and `render.rs` change.

Steps 1–3 are the classic progression from *The Algorithmic Beauty of Plants*
(Prusinkiewicz & Lindenmayer), which is free online and is the reference for
everything here.

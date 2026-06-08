# fleet-midi-harmonizer

Four-part harmony generation from ternary vectors — species counterpoint for agent fleets.

## The Problem

Agent swarms need coordination signals. Most coordination protocols use gossip or consensus on arbitrary state — but what if the coordination signal itself carried *musical structure*? A fleet of agents producing MIDI needs to harmonize: not just agree on a note, but produce four voice parts that obey centuries of voice-leading rules while responding to a shared control signal.

The control signal is ternary: each agent emits a vector of `{-1, 0, +1}` values representing direction (tension, neutrality, resolution). The question: how do you map that to SATB harmony that doesn't sound like garbage?

## The Insight

Species counterpoint is a constraint satisfaction problem with a natural cost function. The rules are finite and enumerable:

- No parallel fifths or octaves between any pair of voices
- Leading tones (scale degree 7) must resolve upward
- No voice crossings (soprano ≥ alto ≥ tenor ≥ bass)
- No augmented intervals (tritones) in melodic motion
- All voices must stay in their ranges (S: C4–A5, A: G3–D5, T: C3–A4, B: C2–C4)

Meanwhile, ternary vectors have a natural harmonic interpretation:
- **Sum > 0** → tension → dominant chords (V, vii°, V7)
- **Sum < 0** → resolution → tonic chords (I, vi)
- **Sum = 0** → neutral → predominant chords (IV, ii, iii)

The cost function combines four weighted terms: voice-leading distance, harmonic tension cost, ternary alignment score, and a heavy penalty for rule violations. Picking the best chord from candidates becomes a simple argmin.

## How It Works

1. A ternary vector arrives (e.g., `[1, 0, -1]`, sum = 0, neutral)
2. The `TernaryToChord` mapper generates diatonic chord candidates based on the vector's sum
3. Each candidate is voiced around the previous bass note, clamped to SATB ranges
4. The `CostFunction` evaluates each (chord, voice) pair:
   - **Voice-leading distance**: total semitone movement from previous voicing (weight: 1.0)
   - **Tension cost**: penalizes dissonant chords when tension budget is low (weight: 0.5)
   - **Ternary alignment**: how well the chord matches the vector's intent (weight: 0.8)
   - **Rule violation penalty**: parallel fifths, unresolved leading tones, etc. (weight: 10.0)
5. The lowest-cost candidate wins and becomes the next step in the progression

The `CounterpointRules` engine checks all six voice pairs for parallel motion, all four voices for leading tone resolution, and all voices for range and crossing violations.

## Code

```rust
use fleet_midi_harmonizer::{ChordProgression, HarmonicContext, Mode};

let ctx = HarmonicContext::new(0, Mode::Major).with_tension(0.5);

let prog = ChordProgression::generate(ctx, &[
    vec![-1, -1,  0],  // → I (resolution)
    vec![ 0,  0,  0],  // → IV (predominant)
    vec![ 1,  1,  0],  // → V7 (tension)
    vec![-1, -1, -1],  // → I (resolution)
]);

for step in &prog.steps {
    println!("{} {} cost={:.2}", step.chord, step.voice, step.cost);
}
// Cmaj S=72 A=64 T=55 B=48 cost=0.56
// Fmaj S=72 A=65 T=57 B=53 cost=3.12
// G7  S=74 A=67 T=55 B=43 cost=5.84
// Cmaj S=72 A=64 T=55 B=48 cost=4.30

let violations = prog.validate();
let analysis = prog.roman_analysis();
```

## Module Map

| Module | Responsibility | Key Types |
|---|---|---|
| `harmony` | Chord theory, key/mode, tension budget | `Chord`, `ChordQuality`, `HarmonicContext`, `Mode` |
| `voice` | SATB voice ranges, crossing detection, distance | `Voice`, range constants |
| `rules` | Counterpoint constraint engine | `CounterpointRules`, `RuleViolation` enum |
| `ternary` | `{-1,0,+1}` → chord candidates, alignment scoring | `TernaryToChord`, `TernaryVector` |
| `cost` | Weighted cost function for candidate selection | `CostFunction` |
| `progression` | Multi-step chord progression with voice leading | `ChordProgression`, `ProgressionStep` |

## Design Decisions

**Why diatonic-only candidates?** Chromatic chords (borrowed chords, secondary dominants) would expand the search space dramatically with marginal benefit for agent fleet coordination. The diatonic constraint keeps the system predictable and controllable. You can always extend `chord_candidates()` with your own mapping.

**Why species counterpoint rules and not jazz voice leading?** The rules here are stricter than jazz voicing (which allows parallel seconds, tritone substitution, etc.) because the output is meant to be *structurally correct* — a foundation. Relax constraints by reducing `rule_violation_penalty` in the cost function.

**Why tension budget as a float?** A binary tension/no-tension flag loses information. A float lets you smoothly transition between passages — opening movements (low tension), development (high tension), recapitulation (low again). The `HarmonicContext` carries this through the entire progression.

**Why 6 voice pairs for parallel checks?** SATB has 4 voices and C(4,2) = 6 pairs: soprano-bass, soprano-tenor, soprano-alto, alto-tenor, alto-bass, tenor-bass. Parallel fifths/octaves between *any* pair is forbidden — not just outer voices.

**Why ternary vectors of length 3?** The three dimensions encode [tension, color, bass_motion]. This gives enough expressiveness (27 possible vectors) without overwhelming the candidate space. Longer vectors work too — the alignment scoring scales with length.

**No-std compatible?** The crate uses only `serde`, `thiserror`, and core `std` formatting. No I/O, no MIDI output, no filesystem. It's a pure computation library that produces `(Chord, Voice)` pairs — rendering to MIDI bytes is your responsibility.

## Stats

- 73 tests, all passing
- Pure Rust, zero unsafe
- Dependencies: `serde`, `thiserror`
- No MIDI I/O — produces note data, not bytes

## License

MIT

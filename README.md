# fleet-midi-harmonizer

Four-part harmony generation from ternary vectors — species counterpoint for agent fleets.

## Overview

`fleet-midi-harmonizer` is a Rust library that generates four-part (SATB) chord progressions from ternary state vectors `{-1, 0, +1}`. It applies species counterpoint rules to produce harmonically valid voice leading, making it suitable for generative music systems, agent-based composition, and real-time MIDI control.

## Architecture

| Module | Purpose |
|---|---|
| `harmony` | `HarmonicContext` (key, mode, tension budget), `Chord` (root, quality, voicing), diatonic triads, Roman numeral analysis |
| `voice` | `Voice` (soprano/alto/tenor/bass as MIDI note numbers), range constraints, crossing detection, semitone distance |
| `rules` | `CounterpointRules` — no parallel fifths/octaves, leading tone resolution, augmented interval avoidance, voice crossing checks |
| `ternary` | `TernaryToChord` — maps `{-1, 0, +1}` vectors to chord candidates with alignment scoring |
| `cost` | `CostFunction` — combines voice-leading smoothness + tension management + ternary alignment + rule violation penalties to pick optimal chords |
| `progression` | `ChordProgression` — sequence of chords with optimal voice leading, full counterpoint validation, Roman numeral analysis |

## Quick Start

```rust
use fleet_midi_harmonizer::{ChordProgression, HarmonicContext, Mode};

let ctx = HarmonicContext::new(0, Mode::Major); // C major
let vectors = vec![
    vec![-1, 0, 0],   // → tonic tendency
    vec![0, 0, 0],    // → neutral / predominant
    vec![1, 0, 0],    // → tension / dominant
    vec![-1, -1, 0],  // → resolution
];

let progression = ChordProgression::generate(ctx, &vectors);

for step in &progression.steps {
    println!("{} [{}] cost={:.2}", step.chord, step.voice, step.cost);
}

// Validate the entire progression for counterpoint violations
let violations = progression.validate();
if violations.is_empty() {
    println!("✓ Clean voice leading!");
}
```

## Ternary Mapping

The ternary vector `{−1, 0, +1}` encodes harmonic intention:

| Vector sum | Meaning | Chord candidates |
|---|---|---|
| **> 0** | Tension / dominant | V, vii°, V7 |
| **< 0** | Resolution / tonic | I, vi |
| **= 0** | Neutral / predominant | IV, ii, iii |

Multi-dimensional vectors (e.g., `[tension, color, bass_motion]`) allow fine-grained control. The alignment scoring function rewards chord choices that match the vector's implied harmonic function.

## Counterpoint Rules

The engine enforces classical species counterpoint constraints:

- **No parallel fifths** — consecutive perfect fifths between any two voices
- **No parallel octaves** — consecutive octave unisons between any two voices
- **Leading tone resolution** — the 7th scale degree must resolve upward
- **No augmented intervals** — tritone leaps (6 semitones) are flagged
- **No voice crossings** — SATB ordering is maintained

## Cost Function

Chord selection minimizes a weighted cost:

```
total_cost = w₁ × voice_leading_distance
           + w₂ × tension_cost
           + w₃ × (1 − ternary_alignment)
           + w₄ × rule_violation_count
```

Default weights are tuned for smooth, idiomatic voice leading. Override via `CostFunction` fields.

## Voice Ranges

| Voice | Range (MIDI) | Approximate |
|---|---|---|
| Soprano | 60–81 | C4–A5 |
| Alto | 55–74 | G3–D5 |
| Tenor | 48–69 | C3–A4 |
| Bass | 36–60 | C2–C4 |

## Features

- **Pure Rust** — no `unsafe`, no C dependencies
- **Serde support** — serialize/deserialize chords, voicings, progressions
- **73 tests** — chord construction, voice ranges, counterpoint rules, ternary mapping, progression generation
- **Clippy clean** — passes with `-D warnings`

## Dependencies

- `serde` — serialization
- `thiserror` — error types for counterpoint violations

## License

MIT

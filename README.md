# fleet-midi-harmonizer

**Conservation-governed MIDI harmonization. SATB voice leading meets ternary algebra.**

Given a melody or chord, this crate generates four-part harmonizations (SATB) that respect counterpoint rules, minimize voice leading distance, and map balanced ternary vectors {-1, 0, +1} to harmonic decisions. The result: harmonizations that sound good because the math guarantees it.

## The Key Insight

Traditional harmony is taught as rules: "no parallel fifths," "resolve the leading tone," "avoid voice crossing." These aren't arbitrary — they're **conservation laws**. Parallel fifths violate conservation of voice-leading distance. Unresolved leading tones violate conservation of tension.

This crate encodes those rules as a **cost function** that scores every possible chord voicing. The cost function balances:
- **Voice leading distance** (smooth transitions between chords)
- **Tension/release** (appropriate harmonic tension for the context)
- **Ternary alignment** (how well the voicing matches a ternary input vector)
- **Rule violations** (parallel fifths, octaves, voice crossing — heavily penalized)

The optimal harmonization minimizes total cost. It's optimization over a musical constraint space.

## Architecture

```
fleet-midi-harmonizer
├── src/
│   ├── harmony.rs      # Chord, ChordQuality, Mode, HarmonicContext
│   ├── voice.rs        # SATB voicing, voice ranges, interval calculations
│   ├── rules.rs        # CounterpointRules: parallel 5ths/8ves, crossing, spacing
│   ├── ternary.rs      # TernaryVector → chord candidate mapping
│   ├── cost.rs         # CostFunction: weighted scoring of voicing candidates
│   └── progression.rs  # ChordProgression: sequence of steps with cost tracking
├── examples/
│   └── basic.rs
└── Cargo.toml
```

### Data Flow

```
Ternary Vector {-1, 0, +1}
        │
        ▼
   TernaryToChord ──────► Chord Candidates
        │                        │
        │                        ▼
        │              CounterpointRules ──► Valid voicings
        │                        │
        │                        ▼
        └──────────────── CostFunction ──► Best voicing (min cost)
                                         │
                                         ▼
                                  ProgressionStep
                                  (Chord + Voice + Cost)
```

## Quick Start

```rust
use fleet_midi_harmonizer::*;
use fleet_midi_harmonizer::ternary::TernaryVector;

// Set up harmonic context: C major
let ctx = HarmonicContext::new(0, Mode::Major); // 0 = C

// Create a ternary input vector
let ternary: TernaryVector = vec![1, 0, -1, 0]; // direction: up, neutral, down, neutral

// Map ternary to chord candidates
let mapper = TernaryToChord::new(ctx);
let candidates = mapper.generate_candidates(&ternary);
println!("{} chord candidates from ternary {:?}", candidates.len(), ternary);

// Score each candidate with cost function
let cost_fn = CostFunction::default();
for chord in &candidates {
    let cost = cost_fn.evaluate(chord, &ctx, &ternary);
    println!("  {} → cost = {:.2}", chord, cost);
}
```

## Tutorial: Building a Chord Progression

```rust
use fleet_midi_harmonizer::*;
use fleet_midi_harmonizer::ternary::TernaryVector;
use fleet_midi_harmonizer::progression::ChordProgression;

fn main() {
    let ctx = HarmonicContext::new(0, Mode::Major); // C major

    // Build a progression step by step
    let mut progression = ChordProgression::new(ctx.clone());

    // I → IV → V → I with ternary inputs
    let steps: Vec<(&str, TernaryVector)> = vec![
        ("I",   vec![1, 0, 0, 0]),   // Tonic, moving up
        ("IV",  vec![0, 1, 0, 0]),   // Subdominant, expanding
        ("V",   vec![0, 0, -1, 0]),  // Dominant, tension
        ("I",   vec![1, 0, 0, 1]),   // Tonic, resolving
    ];

    let mapper = TernaryToChord::new(ctx);
    let cost_fn = CostFunction::default();
    let rules = CounterpointRules::strict();

    for (roman, ternary) in &steps {
        let candidates = mapper.generate_candidates(ternary);
        // Find the best voicing that doesn't violate rules
        if let Some((best_chord, best_voice)) = rules.best_valid(
            &candidates,
            progression.last_voice(),
        ) {
            let cost = cost_fn.evaluate(&best_chord, &ctx, ternary);
            progression.add_step(best_chord, best_voice, ternary.clone(), cost);
            println!("{}: {} (cost: {:.2})", roman, best_chord, cost);
        }
    }

    println!("\nTotal cost: {:.2}", progression.total_cost());
    println!("Progression:\n{}", progression);
}
```

## Tutorial: Ternary-to-Harmony Mapping

The ternary vector maps to harmonic direction:
- `+1` = move up (tension, departure)
- `0` = stay (stable, rest)
- `-1` = move down (resolution, return)

```rust
use fleet_midi_harmonizer::*;
use fleet_midi_harmonizer::ternary::TernaryVector;

fn main() {
    let ctx = HarmonicContext::new(0, Mode::Major);
    let mapper = TernaryToChord::new(ctx);

    // Different ternary vectors produce different harmonic directions
    let vectors: Vec<(&str, TernaryVector)> = vec![
        ("Tension up",    vec![1, 1, 0, 0]),
        ("Stable",        vec![0, 0, 0, 0]),
        ("Resolution",    vec![-1, -1, 0, 0]),
        ("Mixed",         vec![1, -1, 1, -1]),
    ];

    for (label, vec) in &vectors {
        let candidates = mapper.generate_candidates(vec);
        println!("{} {:?} → {} candidates", label, vec, candidates.len());
    }
}
```

## Counterpoint Rules

The `CounterpointRules` module enforces classical four-part writing rules:

| Rule | Violation | Penalty |
|------|-----------|---------|
| No parallel fifths | Two voices move in parallel P5 | 10.0 |
| No parallel octaves | Two voices move in parallel P8 | 10.0 |
| No voice crossing | Lower voice exceeds upper | 10.0 |
| Voice range limits | Soprano > A5, Bass < C2 | 5.0 |
| Maximum spacing | > P8 between adjacent voices | 3.0 |

Rules can be strict (reject violations) or lenient (allow with penalty).

## Voice Ranges

```
Soprano: C4 (60) ──────── A5 (81)
Alto:    G3 (55) ──────── D5 (74)
Tenor:   C3 (48) ──────── A4 (69)
Bass:    C2 (36) ──────── C4 (60)
```

## API Reference

### Core Types
- `Chord` — root note + quality + inversion
- `ChordQuality` — Major, Minor, Diminished, Augmented, Dominant7, Major7, Minor7, HalfDim7
- `Voice` — SATB assignment (4 MIDI note numbers)
- `HarmonicContext` — key + mode for harmonic analysis
- `TernaryVector` — `Vec<i8>` where values are {-1, 0, +1}

### Key Functions
- `TernaryToChord::generate_candidates(&vec)` → chord options from ternary input
- `CostFunction::evaluate(&chord, &ctx, &vec)` → weighted cost score
- `CounterpointRules::check(&prev, &curr)` → list of violations
- `ChordProgression::add_step(...)` → extend progression with optimal voicing

## Ecosystem Role

In SuperInstance, `fleet-midi-harmonizer` is the bridge between:
- **ternary agents** (which output {-1, 0, +1} decisions) and **musical output** (MIDI harmonization)
- **conservation laws** (energy preservation) and **musical rules** (no parallel fifths)
- **spreadsheet-engine** cells (which can contain ternary values) and **audible music**

## License

MIT

//! Tutorial: Building harmonizations from ternary agent decisions

use fleet_midi_harmonizer::*;
use fleet_midi_harmonizer::ternary::TernaryVector;

fn main() {
    println!("=== Fleet MIDI Harmonizer Tutorial ===\n");

    // Part 1: Chord qualities and intervals
    println!("--- Chord Quality Interval Structures ---\n");
    let qualities = [
        (ChordQuality::Major, "Major"),
        (ChordQuality::Minor, "Minor"),
        (ChordQuality::Diminished, "Diminished"),
        (ChordQuality::Augmented, "Augmented"),
        (ChordQuality::Dominant7, "Dom7"),
        (ChordQuality::Major7, "Maj7"),
    ];

    for (quality, name) in &qualities {
        let chord = Chord::new(0, *quality);
        println!("C{}: intervals {:?}, pitch classes {:?}", name, quality.intervals(), chord.pitch_classes());
    }
    println!();

    // Part 2: Ternary-to-harmony mapping
    println!("--- Ternary Vector → Chord Mapping ---\n");
    let ctx = HarmonicContext::new(0, Mode::Major);
    let mapper = TernaryToChord::new(ctx);

    let patterns: Vec<(&str, TernaryVector)> = vec![
        ("Ascending",    vec![1, 1, 1, 1]),
        ("Stable",       vec![0, 0, 0, 0]),
        ("Descending",   vec![-1, -1, -1, -1]),
        ("Mixed",        vec![1, 0, -1, 0]),
        ("Push-pull",    vec![1, -1, 1, -1]),
    ];

    for (label, vec) in &patterns {
        let candidates = mapper.chord_candidates(vec);
        println!("{} {:?} → {} candidates", label, vec, candidates.len());
        for (i, chord) in candidates.iter().take(3).enumerate() {
            println!("  {}: {} pcs={:?}", i + 1, chord, chord.pitch_classes());
        }
        println!();
    }

    // Part 3: Voice leading
    println!("--- Voice Ranges ---\n");
    let v1 = Voice::new(72, 64, 60, 48); // C major in SATB
    let v2 = Voice::new(71, 62, 59, 43); // G chord
    println!("Voice 1: {:?}", v1.to_array_satb());
    println!("Voice 2: {:?}", v2.to_array_satb());
    println!("Voice leading distance: {}", v1.distance_to(&v2));
    println!("In range: {}", v1.in_range());
    println!("No crossing: {}", v1.no_crossing());
    println!();

    // Part 4: Ternary mapping to voice
    println!("--- Full Ternary → Chord + Voice ---\n");
    let ctx2 = HarmonicContext::new(0, Mode::Major);
    let mapper2 = TernaryToChord::new(ctx2);
    let start_voice = Voice::new(72, 64, 60, 48); // C major

    let ternary_steps: Vec<TernaryVector> = vec![
        vec![0, 0, 0, 0],   // stay on tonic
        vec![1, 0, -1, 0],  // move toward dominant
        vec![-1, 0, 1, 0],  // resolve back
    ];

    let mut prev_voice = Some(start_voice);
    for (i, tv) in ternary_steps.iter().enumerate() {
        match mapper2.map_to_voice(tv, prev_voice.as_ref()) {
            Some((chord, voice)) => {
                println!("Step {}: {:?} → {} voice={:?}", i + 1, tv, chord, voice.to_array_satb());
                prev_voice = Some(voice);
            }
            None => println!("Step {}: {:?} → no valid mapping", i + 1, tv),
        }
    }

    // Part 5: Diatonic triads in context
    println!("\n--- Diatonic Triads in C Major ---\n");
    let ctx3 = HarmonicContext::new(0, Mode::Major);
    for (i, triad) in ctx3.diatonic_triads().iter().enumerate() {
        let rn = triad.roman_numeral(0, Mode::Major);
        println!("  {}: {} ({})", i + 1, triad, rn);
    }
}

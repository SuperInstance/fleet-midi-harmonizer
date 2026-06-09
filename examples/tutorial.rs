//! Tutorial: Building harmonizations from ternary agent decisions
//!
//! Shows how fleet-midi-harmonizer converts ternary vectors
//! into four-part SATB harmonizations.

use fleet_midi_harmonizer::*;
use fleet_midi_harmonizer::ternary::TernaryVector;

fn main() {
    println!("=== Fleet MIDI Harmonizer Tutorial ===\n");

    // Part 1: Understanding ternary-to-harmony mapping
    println!("--- Ternary Vector → Chord Mapping ---\n");
    let ctx = HarmonicContext::new(0, Mode::Major); // C major
    let mapper = TernaryToChord::new(ctx);

    let patterns: Vec<(&str, TernaryVector)> = vec![
        ("Ascending (tension)",   vec![1, 1, 1, 1]),
        ("Stable (tonic)",        vec![0, 0, 0, 0]),
        ("Descending (resolve)",  vec![-1, -1, -1, -1]),
        ("Mixed energy",          vec![1, 0, -1, 0]),
        ("Push-pull",             vec![1, -1, 1, -1]),
    ];

    for (label, vec) in &patterns {
        let candidates = mapper.generate_candidates(vec);
        println!("{} {:?}:", label, vec);
        for (i, chord) in candidates.iter().take(3).enumerate() {
            println!("  Candidate {}: {}", i + 1, chord);
        }
        if candidates.len() > 3 {
            println!("  ... and {} more", candidates.len() - 3);
        }
        println!();
    }

    // Part 2: Chord qualities and intervals
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
        let chord = Chord::new(0, *quality); // C chord
        println!("C{}: {:?}", name, chord.intervals());
    }
    println!();

    // Part 3: Voice ranges
    println!("--- SATB Voice Ranges ---\n");
    println!("Soprano: MIDI {}–{}", SOPRANO_RANGE.0, SOPRANO_RANGE.1);
    println!("Alto:    MIDI {}–{}", ALTO_RANGE.0, ALTO_RANGE.1);
    println!("Tenor:   MIDI {}–{}", TENOR_RANGE.0, TENOR_RANGE.1);
    println!("Bass:    MIDI {}–{}", BASS_RANGE.0, BASS_RANGE.1);
    println!();

    // Part 4: Cost function
    println!("--- Cost Function Weights ---\n");
    let cost = CostFunction::default();
    println!("Voice leading:  {:.1}", cost.voice_leading_weight);
    println!("Tension:        {:.1}", cost.tension_weight);
    println!("Ternary align:  {:.1}", cost.ternary_alignment_weight);
    println!("Rule violation: {:.1}", cost.rule_violation_penalty);
}

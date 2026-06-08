use crate::harmony::{Chord, ChordQuality, HarmonicContext};
use crate::voice::Voice;

/// A ternary vector: values in {-1, 0, +1}.
pub type TernaryVector = Vec<i8>;

/// Validates that all values are in {-1, 0, +1}.
pub fn validate_ternary(vec: &TernaryVector) -> bool {
    vec.iter().all(|&v| v == -1 || v == 0 || v == 1)
}

/// Maps ternary vectors to chord candidates.
#[derive(Debug, Clone)]
pub struct TernaryToChord {
    pub context: HarmonicContext,
}

impl TernaryToChord {
    pub fn new(context: HarmonicContext) -> Self {
        Self { context }
    }

    /// Interpret a ternary vector as a harmonic direction and generate chord candidates.
    ///
    /// Mapping scheme:
    /// - Sum > 0: tendency toward tension/dominant → V, vii°, V7
    /// - Sum < 0: tendency toward resolution/tonic → I, vi
    /// - Sum == 0: neutral → IV, ii, iii
    ///
    /// For 3-element vectors: [tension, color, bass_motion]
    pub fn chord_candidates(&self, vec: &TernaryVector) -> Vec<Chord> {
        if !validate_ternary(vec) || vec.is_empty() {
            return vec![];
        }

        let triads = self.context.diatonic_triads();
        let sum: i32 = vec.iter().map(|&v| v as i32).sum();

        if sum > 0 {
            // Tension: V, vii° (and V7 if tension budget allows)
            let mut candidates = Vec::new();
            // V chord (degree 4)
            if triads.len() > 4 {
                candidates.push(triads[4].clone());
            }
            // vii° (degree 6)
            if triads.len() > 6 {
                candidates.push(triads[6].clone());
            }
            // V7
            if triads.len() > 4 {
                candidates.push(Chord::new(triads[4].root, ChordQuality::Dominant7));
            }
            candidates
        } else if sum < 0 {
            // Resolution: I, vi
            let mut candidates = Vec::new();
            if !triads.is_empty() {
                candidates.push(triads[0].clone()); // I/i
            }
            if triads.len() > 5 {
                candidates.push(triads[5].clone()); // vi/VI
            }
            candidates
        } else {
            // Neutral: IV, ii, iii
            let mut candidates = Vec::new();
            if triads.len() > 3 {
                candidates.push(triads[3].clone()); // IV/iv
            }
            if triads.len() > 1 {
                candidates.push(triads[1].clone()); // ii/ii°
            }
            if triads.len() > 2 {
                candidates.push(triads[2].clone()); // iii/III
            }
            candidates
        }
    }

    /// Score how well a chord aligns with a ternary vector.
    /// Higher score = better alignment. Range [0.0, 1.0].
    pub fn alignment_score(&self, chord: &Chord, vec: &TernaryVector) -> f64 {
        if vec.is_empty() {
            return 0.5;
        }

        let triads = self.context.diatonic_triads();
        let sum: i32 = vec.iter().map(|&v| v as i32).sum();

        // Find the chord's degree
        let degree = triads.iter().position(|t| t.root == chord.root);
        let is_tonic = degree == Some(0);
        let is_dominant = degree == Some(4);
        let is_predominant = degree == Some(1) || degree == Some(3);

        let score = if sum > 0 && is_dominant {
            0.9
        } else if sum > 0 && chord.quality.is_dissonant() {
            0.8
        } else if sum < 0 && is_tonic {
            0.9
        } else if sum < 0 && degree == Some(5) {
            0.7 // vi deceptive
        } else if sum == 0 && is_predominant {
            0.8
        } else if self.context.is_diatonic(chord) {
            0.4
        } else {
            0.1 // chromatic
        };

        // Ternary vector length bonus: more dimensions = more specific guidance
        let specificity_bonus = (vec.len() as f64).min(4.0) / 20.0;
        (score + specificity_bonus).min(1.0)
    }

    /// Map a ternary vector to a voiced chord (bass note, voicing) given previous voice state.
    pub fn map_to_voice(&self, vec: &TernaryVector, prev_voice: Option<&Voice>) -> Option<(Chord, Voice)> {
        let candidates = self.chord_candidates(vec);
        if candidates.is_empty() {
            return None;
        }

        let bass_center = match prev_voice {
            Some(v) => v.bass,
            None => 48, // Default bass center C3
        };

        let chord = candidates.into_iter().next()?;
        let notes = chord.voice_around(bass_center);

        // Clamp to voice ranges
        let voice = Voice::new(
            notes[3].clamp(crate::voice::SOPRANO_RANGE.0, crate::voice::SOPRANO_RANGE.1),
            notes[2].clamp(crate::voice::ALTO_RANGE.0, crate::voice::ALTO_RANGE.1),
            notes[1].clamp(crate::voice::TENOR_RANGE.0, crate::voice::TENOR_RANGE.1),
            notes[0].clamp(crate::voice::BASS_RANGE.0, crate::voice::BASS_RANGE.1),
        );

        Some((chord, voice))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mode;

    fn ctx_c_major() -> HarmonicContext {
        HarmonicContext::new(0, Mode::Major)
    }

    #[test]
    fn validate_good_ternary() {
        assert!(validate_ternary(&vec![1, 0, -1]));
    }

    #[test]
    fn validate_bad_ternary() {
        assert!(!validate_ternary(&vec![1, 2, -1]));
    }

    #[test]
    fn validate_empty() {
        assert!(validate_ternary(&vec![]));
    }

    #[test]
    fn tension_vector_gives_dominant() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let candidates = t2c.chord_candidates(&vec![1, 1, 1]);
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].root, 7); // G = V in C
        assert_eq!(candidates[0].quality, ChordQuality::Major);
    }

    #[test]
    fn resolution_vector_gives_tonic() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let candidates = t2c.chord_candidates(&vec![-1, -1, -1]);
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].root, 0); // C = I
    }

    #[test]
    fn neutral_vector_gives_predominant() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let candidates = t2c.chord_candidates(&vec![0, 0, 0]);
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].root, 5); // F = IV in C
    }

    #[test]
    fn alignment_score_tonic_with_resolution() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let cmaj = Chord::new(0, ChordQuality::Major);
        let score = t2c.alignment_score(&cmaj, &vec![-1, -1, -1]);
        assert!(score > 0.8);
    }

    #[test]
    fn alignment_score_dominant_with_tension() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let gmaj = Chord::new(7, ChordQuality::Major);
        let score = t2c.alignment_score(&gmaj, &vec![1, 1, 0]);
        assert!(score > 0.8);
    }

    #[test]
    fn map_to_voice_no_previous() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let result = t2c.map_to_voice(&vec![1, 0, -1], None);
        assert!(result.is_some());
        let (chord, voice) = result.unwrap();
        let _chord = chord;
        assert!(voice.in_range());
    }

    #[test]
    fn map_to_voice_with_previous() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let prev = Voice::new(72, 64, 55, 48);
        let result = t2c.map_to_voice(&vec![0, 0, 0], Some(&prev));
        assert!(result.is_some());
    }

    #[test]
    fn invalid_vector_no_candidates() {
        let t2c = TernaryToChord::new(ctx_c_major());
        let candidates = t2c.chord_candidates(&vec![2, 3, 4]);
        assert!(candidates.is_empty());
    }

    #[test]
    fn minor_key_candidates() {
        let ctx = HarmonicContext::new(9, Mode::Minor); // A minor
        let t2c = TernaryToChord::new(ctx);
        let candidates = t2c.chord_candidates(&vec![1, 1, 1]);
        assert!(!candidates.is_empty());
        // V in A minor = E (pitch class 4)
        assert_eq!(candidates[0].root, 4);
    }
}

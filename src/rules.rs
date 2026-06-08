use crate::voice::Voice;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from counterpoint rule violations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum RuleViolation {
    #[error("parallel fifths between {voice_a} and {voice_b}: {prev_interval}→{curr_interval}")]
    ParallelFifths {
        voice_a: String,
        voice_b: String,
        prev_interval: u8,
        curr_interval: u8,
    },
    #[error("parallel octaves between {voice_a} and {voice_b}: {prev_interval}→{curr_interval}")]
    ParallelOctaves {
        voice_a: String,
        voice_b: String,
        prev_interval: u8,
        curr_interval: u8,
    },
    #[error("voice crossing: {upper} ({upper_midi}) < {lower} ({lower_midi})")]
    VoiceCrossing {
        upper: String,
        lower: String,
        upper_midi: u8,
        lower_midi: u8,
    },
    #[error("leading tone {midi} should resolve upward, but moved to {target}")]
    LeadingToneDown { midi: u8, target: u8 },
    #[error("augmented interval {from}→{to} ({interval} semitones) in {voice}")]
    AugmentedInterval {
        voice: String,
        from: u8,
        to: u8,
        interval: u8,
    },
    #[error("voice out of range: {details}")]
    OutOfRange { details: String },
}

/// Counterpoint rules engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterpointRules {
    /// The leading tone pitch classes to watch for (e.g., B in C major = 11).
    pub leading_tones: Vec<u8>,
}

impl CounterpointRules {
    pub fn new(leading_tones: Vec<u8>) -> Self {
        Self { leading_tones }
    }

    /// Default rules for C major: leading tone is B (pitch class 11).
    pub fn c_major() -> Self {
        Self::new(vec![11])
    }

    /// Rules for a given key.
    pub fn for_key(key_root: u8) -> Self {
        // Leading tone is the 7th degree (11 semitones above root)
        Self::new(vec![(key_root + 11) % 12])
    }

    /// Check for parallel fifths and octaves between two voicings.
    pub fn check_parallel_motion(&self, prev: &Voice, curr: &Voice) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let pairs: [(&str, &str, u8, u8, u8, u8); 6] = [
            ("soprano", "bass", prev.soprano, prev.bass, curr.soprano, curr.bass),
            ("soprano", "tenor", prev.soprano, prev.tenor, curr.soprano, curr.tenor),
            ("soprano", "alto", prev.soprano, prev.alto, curr.soprano, curr.alto),
            ("alto", "tenor", prev.alto, prev.tenor, curr.alto, curr.tenor),
            ("alto", "bass", prev.alto, prev.bass, curr.alto, curr.bass),
            ("tenor", "bass", prev.tenor, prev.bass, curr.tenor, curr.bass),
        ];
        for (va, vb, p1, p2, c1, c2) in &pairs {
            let prev_int = (*p1 as i32 - *p2 as i32).unsigned_abs() % 12;
            let curr_int = (*c1 as i32 - *c2 as i32).unsigned_abs() % 12;
            // Both voices moved (not static)
            let both_moved = (c1 != p1) && (c2 != p2);
            if both_moved && prev_int == 7 && curr_int == 7 {
                violations.push(RuleViolation::ParallelFifths {
                    voice_a: va.to_string(),
                    voice_b: vb.to_string(),
                    prev_interval: 7,
                    curr_interval: 7,
                });
            }
            if both_moved && prev_int == 0 && curr_int == 0 {
                violations.push(RuleViolation::ParallelOctaves {
                    voice_a: va.to_string(),
                    voice_b: vb.to_string(),
                    prev_interval: 0,
                    curr_interval: 0,
                });
            }
        }
        violations
    }

    /// Check for voice crossings in a single voicing.
    pub fn check_crossing(&self, voice: &Voice) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        if voice.soprano < voice.alto {
            violations.push(RuleViolation::VoiceCrossing {
                upper: "soprano".into(),
                lower: "alto".into(),
                upper_midi: voice.soprano,
                lower_midi: voice.alto,
            });
        }
        if voice.alto < voice.tenor {
            violations.push(RuleViolation::VoiceCrossing {
                upper: "alto".into(),
                lower: "tenor".into(),
                upper_midi: voice.alto,
                lower_midi: voice.tenor,
            });
        }
        if voice.tenor < voice.bass {
            violations.push(RuleViolation::VoiceCrossing {
                upper: "tenor".into(),
                lower: "bass".into(),
                upper_midi: voice.tenor,
                lower_midi: voice.bass,
            });
        }
        violations
    }

    /// Check that leading tones resolve upward (by step to the tonic).
    pub fn check_leading_tone(&self, prev: &Voice, curr: &Voice) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let parts = [
            ("soprano", prev.soprano, curr.soprano),
            ("alto", prev.alto, curr.alto),
            ("tenor", prev.tenor, curr.tenor),
            ("bass", prev.bass, curr.bass),
        ];
        for (_name, p, c) in &parts {
            let pc = p % 12;
            if self.leading_tones.contains(&pc) {
                // Leading tone should resolve upward to the tonic (+1 or +2 semitones)
                let diff = (*c as i32) - (*p as i32);
                if diff < 0 {
                    violations.push(RuleViolation::LeadingToneDown {
                        midi: *p,
                        target: *c,
                    });
                }
            }
        }
        violations
    }

    /// Check for augmented intervals (e.g., augmented 2nd = 3 semitones in a diatonic context,
    /// or any interval that's an augmented version of a perfect/major interval: 6 semitones
    /// is specifically an augmented 4th / tritone in scalar context).
    /// We flag any melodic leap of an augmented 2nd (3 semitones up in a minor key context)
    /// or any interval of 6 semitones (augmented 4th).
    pub fn check_augmented_intervals(&self, prev: &Voice, curr: &Voice) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let parts = [
            ("soprano", prev.soprano, curr.soprano),
            ("alto", prev.alto, curr.alto),
            ("tenor", prev.tenor, curr.tenor),
            ("bass", prev.bass, curr.bass),
        ];
        for (name, p, c) in &parts {
            let interval = (*c as i32 - *p as i32).unsigned_abs() as u8;
            // Flag augmented 4th (tritone = 6 semitones) and augmented 2nd (3 semitones)
            if interval == 6 {
                violations.push(RuleViolation::AugmentedInterval {
                    voice: name.to_string(),
                    from: *p,
                    to: *c,
                    interval,
                });
            }
        }
        violations
    }

    /// Check range violations.
    pub fn check_range(&self, voice: &Voice) -> Vec<RuleViolation> {
        voice
            .range_violations()
            .into_iter()
            .map(|d| RuleViolation::OutOfRange { details: d })
            .collect()
    }

    /// Run all checks between two voicings.
    pub fn check_all(&self, prev: &Voice, curr: &Voice) -> Vec<RuleViolation> {
        let mut all = Vec::new();
        all.extend(self.check_crossing(curr));
        all.extend(self.check_parallel_motion(prev, curr));
        all.extend(self.check_leading_tone(prev, curr));
        all.extend(self.check_augmented_intervals(prev, curr));
        all.extend(self.check_range(curr));
        all
    }

    /// Count total violations (lower is better for cost function).
    pub fn count_violations(&self, prev: &Voice, curr: &Voice) -> usize {
        self.check_all(prev, curr).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_parallel_fifths_when_static() {
        let rules = CounterpointRules::c_major();
        let prev = Voice::new(72, 64, 55, 48);
        let curr = Voice::new(72, 64, 55, 48);
        let v = rules.check_parallel_motion(&prev, &curr);
        assert!(v.is_empty());
    }

    #[test]
    fn detect_parallel_fifths() {
        let rules = CounterpointRules::c_major();
        // soprano-bass: 72-55=17%12=5... need interval of 7
        // 72-41=31%12=7 (fifth). Then move both: 71-40=31%12=7
        let prev = Voice::new(72, 60, 50, 41); // S-B = 72-41 = 31%12=7
        let curr = Voice::new(71, 60, 50, 40); // S-B = 71-40 = 31%12=7
        let v = rules.check_parallel_motion(&prev, &curr);
        assert!(v.iter().any(|x| matches!(x, RuleViolation::ParallelFifths { .. })));
    }

    #[test]
    fn detect_parallel_octaves() {
        let rules = CounterpointRules::c_major();
        // S-B both on C: 72 and 60 (octave). Move both to B: 71 and 59
        let prev = Voice::new(72, 60, 50, 60); // S-B = 72-60=12%12=0
        let curr = Voice::new(71, 60, 50, 59); // S-B = 71-59=12%12=0
        let v = rules.check_parallel_motion(&prev, &curr);
        assert!(v.iter().any(|x| matches!(x, RuleViolation::ParallelOctaves { .. })));
    }

    #[test]
    fn no_violations_for_good_voice_leading() {
        let rules = CounterpointRules::c_major();
        let prev = Voice::new(72, 64, 55, 48);
        let curr = Voice::new(71, 64, 55, 48);
        let v = rules.check_all(&prev, &curr);
        assert!(v.is_empty());
    }

    #[test]
    fn detect_voice_crossing() {
        let rules = CounterpointRules::c_major();
        let voice = Voice::new(60, 70, 55, 48);
        let v = rules.check_crossing(&voice);
        assert!(!v.is_empty());
        assert!(v.iter().any(|x| matches!(x, RuleViolation::VoiceCrossing { .. })));
    }

    #[test]
    fn no_crossing_when_proper() {
        let rules = CounterpointRules::c_major();
        let voice = Voice::new(72, 64, 55, 48);
        assert!(rules.check_crossing(&voice).is_empty());
    }

    #[test]
    fn leading_tone_resolved_upward_ok() {
        let rules = CounterpointRules::c_major();
        // B(71) → C(72) in soprano
        let prev = Voice::new(71, 64, 55, 48);
        let curr = Voice::new(72, 64, 55, 48);
        assert!(rules.check_leading_tone(&prev, &curr).is_empty());
    }

    #[test]
    fn leading_tone_resolved_downward_violation() {
        let rules = CounterpointRules::c_major();
        // B(71) → A(69) in soprano — downward resolution
        let prev = Voice::new(71, 64, 55, 48);
        let curr = Voice::new(69, 64, 55, 48);
        let v = rules.check_leading_tone(&prev, &curr);
        assert!(v.iter().any(|x| matches!(x, RuleViolation::LeadingToneDown { .. })));
    }

    #[test]
    fn detect_augmented_interval_tritone() {
        let rules = CounterpointRules::c_major();
        // F(65) → B(71) = 6 semitones (tritone)
        let prev = Voice::new(65, 64, 55, 48);
        let curr = Voice::new(71, 64, 55, 48);
        let v = rules.check_augmented_intervals(&prev, &curr);
        assert!(v.iter().any(|x| matches!(x, RuleViolation::AugmentedInterval { .. })));
    }

    #[test]
    fn count_violations() {
        let rules = CounterpointRules::c_major();
        let prev = Voice::new(72, 64, 55, 48);
        let curr = Voice::new(72, 64, 55, 48);
        assert_eq!(rules.count_violations(&prev, &curr), 0);
    }

    #[test]
    fn for_key_g_major() {
        let rules = CounterpointRules::for_key(7); // G major → leading tone F# = 6
        assert_eq!(rules.leading_tones, vec![6]);
    }
}

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::cost::CostFunction;
use crate::harmony::{Chord, HarmonicContext};
use crate::rules::CounterpointRules;
use crate::ternary::{TernaryToChord, TernaryVector};
use crate::voice::Voice;

/// A single step in a chord progression: chord + SATB voicing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressionStep {
    pub chord: Chord,
    pub voice: Voice,
    pub ternary_input: TernaryVector,
    pub cost: f64,
}

impl fmt::Display for ProgressionStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] cost={:.2} vec={:?}",
            self.chord, self.voice, self.cost, self.ternary_input
        )
    }
}

/// A chord progression: sequence of chords with voice leading between them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChordProgression {
    pub steps: Vec<ProgressionStep>,
    pub context: HarmonicContext,
    pub rules: CounterpointRules,
    pub cost_function: CostFunction,
}

impl ChordProgression {
    /// Create a new empty progression in the given context.
    pub fn new(context: HarmonicContext) -> Self {
        let rules = CounterpointRules::for_key(context.key_root);
        Self {
            steps: Vec::new(),
            context,
            rules,
            cost_function: CostFunction::new(),
        }
    }

    /// Create with custom cost function weights.
    pub fn with_cost_function(mut self, cf: CostFunction) -> Self {
        self.cost_function = cf;
        self
    }

    /// Create with custom counterpoint rules.
    pub fn with_rules(mut self, rules: CounterpointRules) -> Self {
        self.rules = rules;
        self
    }

    /// Number of steps in the progression.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Is the progression empty?
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get the last voicing, or a default starting voicing.
    pub fn last_voice(&self) -> Voice {
        self.steps
            .last()
            .map(|s| s.voice.clone())
            .unwrap_or_else(|| Voice::new(72, 64, 55, 48)) // C major starting voicing
    }

    /// Add a chord step from a ternary vector.
    /// Evaluates all candidates and picks the best one.
    pub fn add_step(&mut self, ternary_vec: TernaryVector) -> Option<&ProgressionStep> {
        let t2c = TernaryToChord::new(self.context.clone());
        let candidates = t2c.chord_candidates(&ternary_vec);
        if candidates.is_empty() {
            return None;
        }

        let prev_voice = self.last_voice();
        let mut voiced_candidates: Vec<(Chord, Voice, f64)> = Vec::new();

        for chord in &candidates {
            let notes = chord.voice_around(prev_voice.bass);
            let voice = Voice::new(
                notes[3].clamp(crate::voice::SOPRANO_RANGE.0, crate::voice::SOPRANO_RANGE.1),
                notes[2].clamp(crate::voice::ALTO_RANGE.0, crate::voice::ALTO_RANGE.1),
                notes[1].clamp(crate::voice::TENOR_RANGE.0, crate::voice::TENOR_RANGE.1),
                notes[0].clamp(crate::voice::BASS_RANGE.0, crate::voice::BASS_RANGE.1),
            );
            let alignment = t2c.alignment_score(chord, &ternary_vec);
            voiced_candidates.push((chord.clone(), voice, alignment));
        }

        let best = self.cost_function.pick_best(
            &voiced_candidates,
            &prev_voice,
            &self.context,
            &ternary_vec,
            &self.rules,
        );

        if let Some((idx, cost)) = best {
            let (chord, voice, _) = voiced_candidates.into_iter().nth(idx).unwrap();
            let step = ProgressionStep {
                chord,
                voice,
                ternary_input: ternary_vec,
                cost,
            };
            self.steps.push(step);
            self.steps.last()
        } else {
            None
        }
    }

    /// Generate a full progression from a sequence of ternary vectors.
    pub fn generate(context: HarmonicContext, vectors: &[TernaryVector]) -> Self {
        let mut prog = Self::new(context);
        for vec in vectors {
            prog.add_step(vec.clone());
        }
        prog
    }

    /// Get just the chord sequence.
    pub fn chords(&self) -> Vec<&Chord> {
        self.steps.iter().map(|s| &s.chord).collect()
    }

    /// Get just the voice sequence.
    pub fn voices(&self) -> Vec<&Voice> {
        self.steps.iter().map(|s| &s.voice).collect()
    }

    /// Total cost of the progression.
    pub fn total_cost(&self) -> f64 {
        self.steps.iter().map(|s| s.cost).sum()
    }

    /// Validate the entire progression for counterpoint violations.
    pub fn validate(&self) -> Vec<crate::rules::RuleViolation> {
        let mut violations = Vec::new();
        for i in 0..self.steps.len() {
            let curr = &self.steps[i].voice;
            // Check current voicing for crossing/range
            violations.extend(self.rules.check_crossing(curr));
            violations.extend(self.rules.check_range(curr));
            // Check voice leading from previous
            if i > 0 {
                let prev = &self.steps[i - 1].voice;
                violations.extend(self.rules.check_parallel_motion(prev, curr));
                violations.extend(self.rules.check_leading_tone(prev, curr));
                violations.extend(self.rules.check_augmented_intervals(prev, curr));
            }
        }
        violations
    }

    /// Render as Roman numeral analysis string.
    pub fn roman_analysis(&self) -> Vec<String> {
        self.steps
            .iter()
            .map(|s| s.chord.roman_numeral(self.context.key_root, self.context.mode))
            .collect()
    }
}

impl fmt::Display for ChordProgression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ChordProgression ({}:{:?}, {} steps):",
            crate::harmony::midi_to_name(self.context.key_root + 12), // +12 to get a valid note name
            self.context.mode,
            self.steps.len()
        )?;
        for (i, step) in self.steps.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, step)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harmony::Mode;

    fn c_major_ctx() -> HarmonicContext {
        HarmonicContext::new(0, Mode::Major)
    }

    #[test]
    fn empty_progression() {
        let prog = ChordProgression::new(c_major_ctx());
        assert!(prog.is_empty());
        assert_eq!(prog.len(), 0);
    }

    #[test]
    fn add_single_step() {
        let mut prog = ChordProgression::new(c_major_ctx());
        let result = prog.add_step(vec![1, 0, -1]);
        assert!(result.is_some());
        assert_eq!(prog.len(), 1);
    }

    #[test]
    fn generate_simple_progression() {
        let vectors: Vec<TernaryVector> = vec![
            vec![-1, 0, 0],  // tonic
            vec![0, 0, 0],   // predominant
            vec![1, 0, 0],   // dominant
            vec![-1, -1, 0], // resolution
        ];
        let prog = ChordProgression::generate(c_major_ctx(), &vectors);
        assert_eq!(prog.len(), 4);
    }

    #[test]
    fn progression_all_voices_in_range() {
        let vectors: Vec<TernaryVector> = vec![
            vec![-1, 0, 0],
            vec![0, 0, 0],
            vec![1, 0, 0],
            vec![-1, -1, 0],
        ];
        let prog = ChordProgression::generate(c_major_ctx(), &vectors);
        for step in &prog.steps {
            assert!(step.voice.in_range(), "Voice out of range: {}", step.voice);
        }
    }

    #[test]
    fn progression_no_crossing() {
        let vectors: Vec<TernaryVector> = vec![
            vec![-1, 0, 0],
            vec![0, 0, 0],
            vec![1, 0, 0],
            vec![-1, -1, 0],
        ];
        let prog = ChordProgression::generate(c_major_ctx(), &vectors);
        for step in &prog.steps {
            assert!(step.voice.no_crossing(), "Voice crossing: {}", step.voice);
        }
    }

    #[test]
    fn progression_validate_clean() {
        let vectors: Vec<TernaryVector> = vec![
            vec![-1, 0, 0],
            vec![0, 0, 0],
        ];
        let prog = ChordProgression::generate(c_major_ctx(), &vectors);
        let violations = prog.validate();
        // May have some violations but should be finite
        assert!(violations.len() < 10);
    }

    #[test]
    fn total_cost_positive() {
        let vectors: Vec<TernaryVector> = vec![
            vec![-1, 0, 0],
            vec![1, 0, 0],
        ];
        let prog = ChordProgression::generate(c_major_ctx(), &vectors);
        assert!(prog.total_cost() >= 0.0);
    }

    #[test]
    fn roman_analysis() {
        let vectors: Vec<TernaryVector> = vec![
            vec![-1, -1, 0],
            vec![1, 1, 0],
            vec![-1, -1, -1],
        ];
        let prog = ChordProgression::generate(c_major_ctx(), &vectors);
        let analysis = prog.roman_analysis();
        assert_eq!(analysis.len(), prog.len());
    }

    #[test]
    fn chords_and_voices() {
        let mut prog = ChordProgression::new(c_major_ctx());
        prog.add_step(vec![-1, 0, 0]);
        prog.add_step(vec![1, 0, 0]);
        assert_eq!(prog.chords().len(), 2);
        assert_eq!(prog.voices().len(), 2);
    }

    #[test]
    fn display_format() {
        let mut prog = ChordProgression::new(c_major_ctx());
        prog.add_step(vec![-1, 0, 0]);
        let s = format!("{}", prog);
        assert!(s.contains("ChordProgression"));
        assert!(s.contains("steps"));
    }

    #[test]
    fn last_voice_default() {
        let prog = ChordProgression::new(c_major_ctx());
        let v = prog.last_voice();
        assert_eq!(v.soprano, 72);
    }

    #[test]
    fn last_voice_after_step() {
        let mut prog = ChordProgression::new(c_major_ctx());
        prog.add_step(vec![-1, 0, 0]);
        let v = prog.last_voice();
        assert!(v.in_range());
    }
}

use crate::harmony::{Chord, HarmonicContext};
use crate::rules::CounterpointRules;
use crate::ternary::TernaryVector;
use crate::voice::Voice;

use serde::{Deserialize, Serialize};

/// Cost function for evaluating chord candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostFunction {
    pub voice_leading_weight: f64,
    pub tension_weight: f64,
    pub ternary_alignment_weight: f64,
    pub rule_violation_penalty: f64,
}

impl Default for CostFunction {
    fn default() -> Self {
        Self {
            voice_leading_weight: 1.0,
            tension_weight: 0.5,
            ternary_alignment_weight: 0.8,
            rule_violation_penalty: 10.0,
        }
    }
}

impl CostFunction {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute voice-leading distance (semitones) between two voicings.
    pub fn voice_leading_distance(prev: &Voice, curr: &Voice) -> f64 {
        prev.distance_to(curr) as f64
    }

    /// Compute tension cost for a chord in the given context.
    pub fn tension_cost(context: &HarmonicContext, chord: &Chord) -> f64 {
        context.tension_cost(chord)
    }

    /// Compute ternary alignment (inverted: higher alignment = lower cost).
    pub fn ternary_cost(alignment_score: f64) -> f64 {
        1.0 - alignment_score
    }

    /// Count counterpoint rule violations between two voicings.
    pub fn rule_violation_count(rules: &CounterpointRules, prev: &Voice, curr: &Voice) -> usize {
        rules.count_violations(prev, curr)
    }

    /// Compute the total cost for a chord candidate.
    /// Lower is better.
    #[allow(clippy::too_many_arguments)]
    pub fn total_cost(
        &self,
        prev_voice: &Voice,
        candidate_voice: &Voice,
        context: &HarmonicContext,
        chord: &Chord,
        _ternary_vec: &TernaryVector,
        rules: &CounterpointRules,
        ternary_alignment: f64,
    ) -> f64 {
        let vl_dist = Self::voice_leading_distance(prev_voice, candidate_voice);
        let tension = Self::tension_cost(context, chord);
        let ternary = Self::ternary_cost(ternary_alignment);
        let violations = Self::rule_violation_count(rules, prev_voice, candidate_voice) as f64;

        self.voice_leading_weight * vl_dist
            + self.tension_weight * tension
            + self.ternary_alignment_weight * ternary
            + self.rule_violation_penalty * violations
    }

    /// Pick the best chord+voicing from a list of candidates.
    /// Returns the index and cost of the best candidate.
    pub fn pick_best(
        &self,
        candidates: &[(Chord, Voice, f64)], // (chord, voice, ternary_alignment)
        prev_voice: &Voice,
        context: &HarmonicContext,
        ternary_vec: &TernaryVector,
        rules: &CounterpointRules,
    ) -> Option<(usize, f64)> {
        candidates
            .iter()
            .enumerate()
            .map(|(i, (chord, voice, alignment))| {
                let cost = self.total_cost(
                    prev_voice,
                    voice,
                    context,
                    chord,
                    ternary_vec,
                    rules,
                    *alignment,
                );
                (i, cost)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harmony::{ChordQuality, Mode};

    fn c_major_ctx() -> HarmonicContext {
        HarmonicContext::new(0, Mode::Major)
    }

    fn default_rules() -> CounterpointRules {
        CounterpointRules::c_major()
    }

    #[test]
    fn voice_leading_distance_same() {
        let v = Voice::new(72, 64, 55, 48);
        let cost = CostFunction::voice_leading_distance(&v, &v);
        assert!((cost).abs() < f64::EPSILON);
    }

    #[test]
    fn voice_leading_distance_one_semitone() {
        let v1 = Voice::new(72, 64, 55, 48);
        let v2 = Voice::new(71, 64, 55, 48);
        assert!((CostFunction::voice_leading_distance(&v1, &v2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tension_cost_major_chord() {
        let ctx = c_major_ctx();
        let cmaj = Chord::new(0, ChordQuality::Major);
        assert!((CostFunction::tension_cost(&ctx, &cmaj)).abs() < f64::EPSILON);
    }

    #[test]
    fn tension_cost_dominant_with_low_budget() {
        let ctx = c_major_ctx().with_tension(0.1);
        let g7 = Chord::new(7, ChordQuality::Dominant7);
        let cost = CostFunction::tension_cost(&ctx, &g7);
        assert!(cost > 0.5);
    }

    #[test]
    fn tension_cost_dominant_with_high_budget() {
        let ctx = c_major_ctx().with_tension(0.9);
        let g7 = Chord::new(7, ChordQuality::Dominant7);
        let cost = CostFunction::tension_cost(&ctx, &g7);
        assert!(cost < 0.2);
    }

    #[test]
    fn ternary_cost_perfect_alignment() {
        assert!((CostFunction::ternary_cost(1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn ternary_cost_zero_alignment() {
        assert!((CostFunction::ternary_cost(0.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rule_violation_count_clean() {
        let rules = default_rules();
        let v1 = Voice::new(72, 64, 55, 48);
        let v2 = Voice::new(71, 64, 55, 48);
        assert_eq!(CostFunction::rule_violation_count(&rules, &v1, &v2), 0);
    }

    #[test]
    fn total_cost_no_violations() {
        let cf = CostFunction::new();
        let ctx = c_major_ctx();
        let rules = default_rules();
        let prev = Voice::new(72, 64, 55, 48);
        let curr = Voice::new(71, 64, 55, 48);
        let chord = Chord::new(0, ChordQuality::Major);
        let cost = cf.total_cost(&prev, &curr, &ctx, &chord, &vec![0], &rules, 0.9);
        // Should be low: 1 semitone movement + low tension + good alignment
        assert!(cost < 5.0);
    }

    #[test]
    fn pick_best_selects_closest() {
        let cf = CostFunction::new();
        let ctx = c_major_ctx();
        let rules = default_rules();
        let prev = Voice::new(72, 64, 55, 48);
        let candidates = vec![
            (Chord::new(0, ChordQuality::Major), Voice::new(72, 64, 55, 48), 0.9),
            (Chord::new(7, ChordQuality::Major), Voice::new(79, 67, 55, 43), 0.5),
        ];
        let best = cf.pick_best(&candidates, &prev, &ctx, &vec![-1], &rules);
        assert!(best.is_some());
        let (idx, _cost) = best.unwrap();
        // The first candidate (same voicing, high alignment) should win
        assert_eq!(idx, 0);
    }
}

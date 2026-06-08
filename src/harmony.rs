use serde::{Deserialize, Serialize};
use std::fmt;

/// Musical mode (major or minor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Major,
    Minor,
}

/// Chord quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dominant7,
    Major7,
    Minor7,
    HalfDim7,
}

/// Interval structure (semitones from root) for each chord quality.
impl ChordQuality {
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            ChordQuality::Major => &[0, 4, 7],
            ChordQuality::Minor => &[0, 3, 7],
            ChordQuality::Diminished => &[0, 3, 6],
            ChordQuality::Augmented => &[0, 4, 8],
            ChordQuality::Dominant7 => &[0, 4, 7, 10],
            ChordQuality::Major7 => &[0, 4, 7, 11],
            ChordQuality::Minor7 => &[0, 3, 7, 10],
            ChordQuality::HalfDim7 => &[0, 3, 6, 10],
        }
    }

    /// Does this quality contain a tritone (interval of 6 semitones from some chord tone)?
    pub fn has_tritone(&self) -> bool {
        self.intervals().iter().any(|&i| i == 6 || i == 10)
    }

    /// Is this a dissonant chord (dominant 7, half-dim, diminished)?
    pub fn is_dissonant(&self) -> bool {
        matches!(
            self,
            ChordQuality::Dominant7 | ChordQuality::HalfDim7 | ChordQuality::Diminished
        )
    }
}

impl fmt::Display for ChordQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChordQuality::Major => write!(f, "maj"),
            ChordQuality::Minor => write!(f, "min"),
            ChordQuality::Diminished => write!(f, "dim"),
            ChordQuality::Augmented => write!(f, "aug"),
            ChordQuality::Dominant7 => write!(f, "7"),
            ChordQuality::Major7 => write!(f, "maj7"),
            ChordQuality::Minor7 => write!(f, "min7"),
            ChordQuality::HalfDim7 => write!(f, "m7b5"),
        }
    }
}

/// Pitch class names (C=0, C#=1, ... B=11).
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// MIDI note number to note name + octave.
pub fn midi_to_name(midi: u8) -> String {
    let name = NOTE_NAMES[(midi % 12) as usize];
    let octave = (midi as i32 / 12) - 1;
    format!("{}{}", name, octave)
}

/// Note name to pitch class (0–11). Returns None if unrecognized.
pub fn name_to_pc(name: &str) -> Option<u8> {
    NOTE_NAMES.iter().position(|&n| n == name).map(|i| i as u8)
}

/// A chord: root (pitch class 0–11), quality, and optional voicing offset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chord {
    pub root: u8,
    pub quality: ChordQuality,
    /// Optional bass note pitch class for inversions (None = root position).
    pub bass_pc: Option<u8>,
}

impl Chord {
    pub fn new(root: u8, quality: ChordQuality) -> Self {
        Self {
            root: root % 12,
            quality,
            bass_pc: None,
        }
    }

    pub fn with_inversion(root: u8, quality: ChordQuality, bass_pc: u8) -> Self {
        Self {
            root: root % 12,
            quality,
            bass_pc: Some(bass_pc % 12),
        }
    }

    /// MIDI pitch classes in this chord (0–11), accounting for inversion.
    pub fn pitch_classes(&self) -> Vec<u8> {
        let mut pcs: Vec<u8> = self.quality.intervals()
            .iter()
            .map(|&i| (self.root + i) % 12)
            .collect();
        if let Some(bp) = self.bass_pc {
            // Ensure bass is lowest
            pcs.insert(0, bp);
        }
        pcs
    }

    /// Construct concrete MIDI notes in a four-part voicing around the given center.
    pub fn voice_around(&self, center: u8) -> [u8; 4] {
        let intervals = self.quality.intervals();
        let _n_notes = intervals.len().max(4);
        // Build a 4-note voicing by duplicating/root-position as needed
        let mut notes = Vec::new();
        for i in 0..4 {
            let interval = intervals[i % intervals.len()];
            let octave_shift = (i / intervals.len()) as u8 * 12;
            let candidate = self.root + interval + octave_shift;
            notes.push(candidate);
        }
        // Shift to center around `center` (bass)
        let bass_midi = notes[0];
        if bass_midi < center {
            let shift = ((center - bass_midi) / 12) * 12;
            for n in notes.iter_mut() {
                *n += shift;
            }
        } else if bass_midi > center + 12 {
            let shift = ((bass_midi - center) / 12) * 12;
            for n in notes.iter_mut() {
                *n -= shift;
            }
        }
        [notes[0], notes[1], notes[2], notes[3]]
    }

    /// Roman numeral analysis: degree of root in the given key.
    pub fn roman_numeral(&self, _key_root: u8, mode: Mode) -> String {
        let scale: Vec<u8> = match mode {
            Mode::Major => vec![0, 2, 4, 5, 7, 9, 11],
            Mode::Minor => vec![0, 2, 3, 5, 7, 8, 10],
        };
        let degree = scale.iter().position(|&pc| pc == self.root % 12);
        let num = match degree {
            Some(0) => "I",
            Some(1) => "II",
            Some(2) => "III",
            Some(3) => "IV",
            Some(4) => "V",
            Some(5) => "VI",
            Some(6) => "VII",
            _ => return format!("N/C({})", self.root),
        };
        let quality_str = if self.quality == ChordQuality::Minor
            || self.quality == ChordQuality::Minor7
        {
            num.to_lowercase()
        } else {
            num.to_string()
        };
        format!("{}{}", quality_str, self.quality)
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let root_name = NOTE_NAMES[self.root as usize];
        if let Some(bp) = self.bass_pc {
            let bass_name = NOTE_NAMES[bp as usize];
            write!(f, "{}{}/{}", root_name, self.quality, bass_name)
        } else {
            write!(f, "{}{}", root_name, self.quality)
        }
    }
}

/// Harmonic context: current key, mode, tension budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonicContext {
    pub key_root: u8,
    pub mode: Mode,
    /// Tension budget 0.0–1.0: how much dissonance is allowed.
    pub tension_budget: f64,
}

impl HarmonicContext {
    pub fn new(key_root: u8, mode: Mode) -> Self {
        Self {
            key_root: key_root % 12,
            mode,
            tension_budget: 0.5,
        }
    }

    pub fn with_tension(mut self, budget: f64) -> Self {
        self.tension_budget = budget.clamp(0.0, 1.0);
        self
    }

    /// Scale degrees for the current key/mode (pitch classes 0–11, relative to key_root).
    pub fn scale_degrees(&self) -> Vec<u8> {
        let raw: Vec<u8> = match self.mode {
            Mode::Major => vec![0, 2, 4, 5, 7, 9, 11],
            Mode::Minor => vec![0, 2, 3, 5, 7, 8, 10],
        };
        raw.iter().map(|&d| (self.key_root + d) % 12).collect()
    }

    /// Diatonic triads in the current key (7 chords).
    pub fn diatonic_triads(&self) -> Vec<Chord> {
        let scale = self.scale_degrees();
        let qualities: Vec<ChordQuality> = match self.mode {
            Mode::Major => vec![
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Minor,
                ChordQuality::Major,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Diminished,
            ],
            Mode::Minor => vec![
                ChordQuality::Minor,
                ChordQuality::Diminished,
                ChordQuality::Major,
                ChordQuality::Minor,
                ChordQuality::Minor,
                ChordQuality::Major,
                ChordQuality::Major,
            ],
        };
        scale
            .iter()
            .zip(qualities.iter())
            .map(|(&root, &q)| Chord::new(root, q))
            .collect()
    }

    /// Is the given chord diatonic to this key?
    pub fn is_diatonic(&self, chord: &Chord) -> bool {
        self.diatonic_triads().iter().any(|c| {
            c.root == chord.root && c.quality == chord.quality
        })
    }

    /// Tension cost: dissonant chords in a low-tension context cost more.
    pub fn tension_cost(&self, chord: &Chord) -> f64 {
        if chord.quality.is_dissonant() {
            (1.0 - self.tension_budget).max(0.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_major_intervals() {
        assert_eq!(ChordQuality::Major.intervals(), &[0, 4, 7]);
    }

    #[test]
    fn chord_minor_intervals() {
        assert_eq!(ChordQuality::Minor.intervals(), &[0, 3, 7]);
    }

    #[test]
    fn chord_diminished_intervals() {
        assert_eq!(ChordQuality::Diminished.intervals(), &[0, 3, 6]);
    }

    #[test]
    fn chord_dominant7_intervals() {
        assert_eq!(ChordQuality::Dominant7.intervals(), &[0, 4, 7, 10]);
    }

    #[test]
    fn chord_pitch_classes_c_major() {
        let c = Chord::new(0, ChordQuality::Major);
        assert_eq!(c.pitch_classes(), vec![0, 4, 7]);
    }

    #[test]
    fn chord_pitch_classes_g_major() {
        let g = Chord::new(7, ChordQuality::Major);
        assert_eq!(g.pitch_classes(), vec![7, 11, 2]);
    }

    #[test]
    fn chord_display() {
        let c = Chord::new(0, ChordQuality::Major);
        assert_eq!(format!("{}", c), "Cmaj");
        let dm = Chord::new(2, ChordQuality::Minor);
        assert_eq!(format!("{}", dm), "Dmin");
    }

    #[test]
    fn chord_with_inversion() {
        let c = Chord::with_inversion(0, ChordQuality::Major, 4);
        assert_eq!(c.bass_pc, Some(4));
        assert_eq!(format!("{}", c), "Cmaj/E");
    }

    #[test]
    fn harmonic_context_c_major() {
        let ctx = HarmonicContext::new(0, Mode::Major);
        assert_eq!(ctx.key_root, 0);
        assert_eq!(ctx.scale_degrees(), vec![0, 2, 4, 5, 7, 9, 11]);
    }

    #[test]
    fn harmonic_context_a_minor() {
        let ctx = HarmonicContext::new(9, Mode::Minor);
        assert_eq!(ctx.scale_degrees(), vec![9, 11, 0, 2, 4, 5, 7]);
    }

    #[test]
    fn diatonic_triads_c_major() {
        let ctx = HarmonicContext::new(0, Mode::Major);
        let triads = ctx.diatonic_triads();
        assert_eq!(triads.len(), 7);
        assert_eq!(triads[0].root, 0); // C
        assert_eq!(triads[0].quality, ChordQuality::Major);
        assert_eq!(triads[1].root, 2); // D
        assert_eq!(triads[1].quality, ChordQuality::Minor);
        assert_eq!(triads[6].root, 11); // B
        assert_eq!(triads[6].quality, ChordQuality::Diminished);
    }

    #[test]
    fn is_diatonic() {
        let ctx = HarmonicContext::new(0, Mode::Major);
        let cmaj = Chord::new(0, ChordQuality::Major);
        let dmin = Chord::new(2, ChordQuality::Minor);
        let cmin = Chord::new(0, ChordQuality::Minor);
        assert!(ctx.is_diatonic(&cmaj));
        assert!(ctx.is_diatonic(&dmin));
        assert!(!ctx.is_diatonic(&cmin));
    }

    #[test]
    fn tension_budget_clamped() {
        let ctx = HarmonicContext::new(0, Mode::Major).with_tension(2.0);
        assert!((ctx.tension_budget - 1.0).abs() < f64::EPSILON);
        let ctx2 = HarmonicContext::new(0, Mode::Major).with_tension(-1.0);
        assert!((ctx2.tension_budget).abs() < f64::EPSILON);
    }

    #[test]
    fn roman_numeral_i_in_c() {
        let ctx = HarmonicContext::new(0, Mode::Major);
        let cmaj = Chord::new(0, ChordQuality::Major);
        let rn = cmaj.roman_numeral(ctx.key_root, ctx.mode);
        assert!(rn.contains('I'));
    }

    #[test]
    fn tritone_detection() {
        assert!(ChordQuality::Diminished.has_tritone());
        assert!(ChordQuality::Dominant7.has_tritone());
        assert!(!ChordQuality::Major.has_tritone());
        assert!(!ChordQuality::Minor.has_tritone());
    }

    #[test]
    fn dissonant_quality() {
        assert!(ChordQuality::Dominant7.is_dissonant());
        assert!(ChordQuality::HalfDim7.is_dissonant());
        assert!(!ChordQuality::Major.is_dissonant());
        assert!(!ChordQuality::Minor7.is_dissonant());
    }
}

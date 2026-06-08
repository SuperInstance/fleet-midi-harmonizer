use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard voice ranges (MIDI note numbers).
pub const SOPRANO_RANGE: (u8, u8) = (60, 81); // C4–A5
pub const ALTO_RANGE: (u8, u8) = (55, 74);    // G3–D5
pub const TENOR_RANGE: (u8, u8) = (48, 69);   // C3–A4
pub const BASS_RANGE: (u8, u8) = (36, 60);    // C2–C4

/// Four-part voice assignment with MIDI note numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voice {
    pub soprano: u8,
    pub alto: u8,
    pub tenor: u8,
    pub bass: u8,
}

impl Voice {
    pub fn new(soprano: u8, alto: u8, tenor: u8, bass: u8) -> Self {
        Self { soprano, alto, tenor, bass }
    }

    /// All four parts as an array [bass, tenor, alto, soprano] (low to high).
    pub fn to_array_low_high(&self) -> [u8; 4] {
        [self.bass, self.tenor, self.alto, self.soprano]
    }

    /// All four parts as an array [soprano, alto, tenor, bass].
    pub fn to_array_satb(&self) -> [u8; 4] {
        [self.soprano, self.alto, self.tenor, self.bass]
    }

    /// Check if all voices are within their standard ranges.
    pub fn in_range(&self) -> bool {
        self.soprano >= SOPRANO_RANGE.0
            && self.soprano <= SOPRANO_RANGE.1
            && self.alto >= ALTO_RANGE.0
            && self.alto <= ALTO_RANGE.1
            && self.tenor >= TENOR_RANGE.0
            && self.tenor <= TENOR_RANGE.1
            && self.bass >= BASS_RANGE.0
            && self.bass <= BASS_RANGE.1
    }

    /// Check for voice crossings: each voice should be ≤ the one above it.
    pub fn no_crossing(&self) -> bool {
        self.soprano >= self.alto && self.alto >= self.tenor && self.tenor >= self.bass
    }

    /// Total semitone distance to another voicing (sum of absolute differences).
    pub fn distance_to(&self, other: &Voice) -> u32 {
        let d = |a: u8, b: u8| (a as i32 - b as i32).unsigned_abs();
        d(self.soprano, other.soprano)
            + d(self.alto, other.alto)
            + d(self.tenor, other.tenor)
            + d(self.bass, other.bass)
    }

    /// List any range violations as strings.
    pub fn range_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        if self.soprano < SOPRANO_RANGE.0 {
            violations.push(format!("soprano {} < {}", self.soprano, SOPRANO_RANGE.0));
        }
        if self.soprano > SOPRANO_RANGE.1 {
            violations.push(format!("soprano {} > {}", self.soprano, SOPRANO_RANGE.1));
        }
        if self.alto < ALTO_RANGE.0 {
            violations.push(format!("alto {} < {}", self.alto, ALTO_RANGE.0));
        }
        if self.alto > ALTO_RANGE.1 {
            violations.push(format!("alto {} > {}", self.alto, ALTO_RANGE.1));
        }
        if self.tenor < TENOR_RANGE.0 {
            violations.push(format!("tenor {} < {}", self.tenor, TENOR_RANGE.0));
        }
        if self.tenor > TENOR_RANGE.1 {
            violations.push(format!("tenor {} > {}", self.tenor, TENOR_RANGE.1));
        }
        if self.bass < BASS_RANGE.0 {
            violations.push(format!("bass {} < {}", self.bass, BASS_RANGE.0));
        }
        if self.bass > BASS_RANGE.1 {
            violations.push(format!("bass {} > {}", self.bass, BASS_RANGE.1));
        }
        violations
    }
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "S={} A={} T={} B={}",
            self.soprano, self.alto, self.tenor, self.bass
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_in_range_valid() {
        let v = Voice::new(72, 64, 55, 48);
        assert!(v.in_range());
    }

    #[test]
    fn voice_in_range_soprano_too_high() {
        let v = Voice::new(90, 64, 55, 48);
        assert!(!v.in_range());
    }

    #[test]
    fn voice_in_range_bass_too_low() {
        let v = Voice::new(72, 64, 55, 20);
        assert!(!v.in_range());
    }

    #[test]
    fn no_crossing_valid() {
        let v = Voice::new(72, 64, 55, 48);
        assert!(v.no_crossing());
    }

    #[test]
    fn crossing_detected_alto_above_soprano() {
        let v = Voice::new(60, 70, 55, 48);
        assert!(!v.no_crossing());
    }

    #[test]
    fn crossing_detected_tenor_above_alto() {
        let v = Voice::new(72, 55, 65, 48);
        assert!(!v.no_crossing());
    }

    #[test]
    fn voice_distance() {
        let v1 = Voice::new(72, 64, 55, 48);
        let v2 = Voice::new(71, 64, 55, 48);
        assert_eq!(v1.distance_to(&v2), 1);
    }

    #[test]
    fn voice_distance_large() {
        let v1 = Voice::new(72, 64, 55, 48);
        let v2 = Voice::new(60, 60, 48, 36);
        let expected: u32 = (72 - 60) + (64 - 60) + (55 - 48) + (48 - 36);
        assert_eq!(v1.distance_to(&v2), expected);
    }

    #[test]
    fn range_violations_empty_when_valid() {
        let v = Voice::new(72, 64, 55, 48);
        assert!(v.range_violations().is_empty());
    }

    #[test]
    fn range_violations_reports_issues() {
        let v = Voice::new(90, 30, 75, 20);
        let violations = v.range_violations();
        assert!(!violations.is_empty());
    }

    #[test]
    fn display_format() {
        let v = Voice::new(72, 64, 55, 48);
        assert_eq!(format!("{}", v), "S=72 A=64 T=55 B=48");
    }

    #[test]
    fn to_array_ordering() {
        let v = Voice::new(72, 64, 55, 48);
        assert_eq!(v.to_array_low_high(), [48, 55, 64, 72]);
        assert_eq!(v.to_array_satb(), [72, 64, 55, 48]);
    }
}

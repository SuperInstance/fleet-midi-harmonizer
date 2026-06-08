pub mod harmony;
pub mod voice;
pub mod rules;
pub mod ternary;
pub mod cost;
pub mod progression;

pub use harmony::{Chord, ChordQuality, HarmonicContext, Mode};
pub use voice::Voice;
pub use rules::CounterpointRules;
pub use ternary::TernaryToChord;
pub use cost::CostFunction;
pub use progression::ChordProgression;

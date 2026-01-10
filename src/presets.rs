use crate::sequencer::Step;

/// A complete preset with pattern and synth settings
pub struct Preset {
    pub name: &'static str,
    pub steps: [Step; 16],
    pub tempo: f32,
    pub cutoff: f32,
    pub resonance: f32,
    pub env_mod: f32,
    pub decay: f32,
    pub saw: bool,
}

// Helper to create steps more easily
const fn step(note: u8, accent: bool, slide: bool, active: bool) -> Step {
    Step { note, accent, slide, active }
}

const fn rest() -> Step {
    Step { note: 36, accent: false, slide: false, active: false }
}

/// Classic 90s acid house patterns
pub static PRESETS: &[Preset] = &[
    // 1. Acid Tracks - Phuture (1987)
    // The original acid track that started it all
    Preset {
        name: "Acid Tracks",
        steps: [
            step(36, true, false, true),   // C2 accent
            step(36, false, false, true),  // C2
            step(48, false, true, true),   // C3 slide
            step(36, false, false, true),  // C2
            step(38, true, false, true),   // D2 accent
            rest(),
            step(36, false, false, true),  // C2
            step(43, false, true, true),   // G2 slide
            step(36, true, false, true),   // C2 accent
            rest(),
            step(48, false, false, true),  // C3
            step(36, false, true, true),   // C2 slide
            step(41, true, false, true),   // F2 accent
            step(36, false, false, true),  // C2
            rest(),
            step(36, false, false, true),  // C2
        ],
        tempo: 126.0,
        cutoff: 400.0,
        resonance: 0.75,
        env_mod: 0.8,
        decay: 150.0,
        saw: true,
    },

    // 2. Higher State - Josh Wink style (1995)
    // Hypnotic rising pattern
    Preset {
        name: "Higher State",
        steps: [
            step(36, true, false, true),   // C2
            step(36, false, false, true),
            step(38, false, true, true),   // D2 slide
            step(40, false, true, true),   // E2 slide
            step(41, true, false, true),   // F2 accent
            step(41, false, false, true),
            step(43, false, true, true),   // G2 slide
            step(45, false, true, true),   // A2 slide
            step(48, true, false, true),   // C3 accent
            step(48, false, false, true),
            step(45, false, true, true),   // A2 slide
            step(43, false, true, true),   // G2 slide
            step(41, true, false, true),   // F2 accent
            step(38, false, true, true),   // D2 slide
            step(36, false, true, true),   // C2 slide
            rest(),
        ],
        tempo: 132.0,
        cutoff: 300.0,
        resonance: 0.85,
        env_mod: 0.9,
        decay: 120.0,
        saw: true,
    },

    // 3. Acperience - Hardfloor style (1992)
    // Fast, hypnotic, minimal
    Preset {
        name: "Acperience",
        steps: [
            step(36, true, false, true),   // C2
            step(36, false, false, true),
            step(36, false, false, true),
            step(48, false, true, true),   // C3 slide
            step(36, true, false, true),   // C2
            step(36, false, false, true),
            step(43, false, true, true),   // G2 slide
            step(36, false, false, true),
            step(36, true, false, true),   // C2
            step(36, false, false, true),
            step(36, false, false, true),
            step(41, false, true, true),   // F2 slide
            step(36, true, false, true),   // C2
            step(36, false, false, true),
            step(38, false, true, true),   // D2 slide
            step(36, false, false, true),
        ],
        tempo: 138.0,
        cutoff: 350.0,
        resonance: 0.8,
        env_mod: 0.7,
        decay: 100.0,
        saw: true,
    },

    // 4. Voodoo Ray - A Guy Called Gerald style (1988)
    // Bouncy, melodic acid
    Preset {
        name: "Voodoo Ray",
        steps: [
            step(41, true, false, true),   // F2
            rest(),
            step(43, false, false, true),  // G2
            step(45, false, true, true),   // A2 slide
            step(48, true, false, true),   // C3 accent
            rest(),
            step(45, false, true, true),   // A2 slide
            step(43, false, false, true),  // G2
            step(41, true, false, true),   // F2 accent
            rest(),
            step(38, false, false, true),  // D2
            step(36, false, true, true),   // C2 slide
            step(38, true, false, true),   // D2 accent
            rest(),
            step(41, false, true, true),   // F2 slide
            rest(),
        ],
        tempo: 118.0,
        cutoff: 500.0,
        resonance: 0.65,
        env_mod: 0.6,
        decay: 200.0,
        saw: true,
    },

    // 5. Mentasm Stab - Second Phase (1991)
    // Classic hoover-style
    Preset {
        name: "Mentasm",
        steps: [
            step(36, true, false, true),
            step(36, false, false, true),
            step(36, false, false, true),
            rest(),
            step(36, true, false, true),
            step(43, false, true, true),   // G2 slide
            step(48, false, true, true),   // C3 slide
            rest(),
            step(36, true, false, true),
            step(36, false, false, true),
            step(36, false, false, true),
            rest(),
            step(36, true, false, true),
            step(41, false, true, true),   // F2 slide
            step(36, false, true, true),   // C2 slide
            rest(),
        ],
        tempo: 128.0,
        cutoff: 600.0,
        resonance: 0.7,
        env_mod: 0.75,
        decay: 180.0,
        saw: false, // Square wave for hoover-ish sound
    },

    // 6. Energy Flash - Joey Beltram style (1990)
    // Driving, relentless
    Preset {
        name: "Energy Flash",
        steps: [
            step(36, true, false, true),
            step(36, false, false, true),
            step(36, true, false, true),
            step(36, false, false, true),
            step(43, true, false, true),
            step(43, false, false, true),
            step(36, true, false, true),
            step(36, false, false, true),
            step(36, true, false, true),
            step(36, false, false, true),
            step(41, true, false, true),
            step(41, false, false, true),
            step(38, true, false, true),
            step(38, false, false, true),
            step(36, true, false, true),
            step(36, false, false, true),
        ],
        tempo: 130.0,
        cutoff: 450.0,
        resonance: 0.72,
        env_mod: 0.65,
        decay: 140.0,
        saw: true,
    },

    // 7. Squelch Classic
    // Pure resonant filter workout
    Preset {
        name: "Squelch Classic",
        steps: [
            step(36, true, false, true),
            step(48, false, true, true),
            step(36, false, true, true),
            step(48, false, true, true),
            step(36, true, false, true),
            step(43, false, true, true),
            step(36, false, true, true),
            step(41, false, true, true),
            step(36, true, false, true),
            step(48, false, true, true),
            step(36, false, true, true),
            step(45, false, true, true),
            step(36, true, false, true),
            step(43, false, true, true),
            step(36, false, true, true),
            step(38, false, true, true),
        ],
        tempo: 125.0,
        cutoff: 250.0,
        resonance: 0.9,
        env_mod: 0.95,
        decay: 100.0,
        saw: true,
    },

    // 8. Minimal Techno
    // Less is more - Detroit influenced
    Preset {
        name: "Minimal Techno",
        steps: [
            step(36, true, false, true),
            rest(),
            rest(),
            step(36, false, false, true),
            rest(),
            step(36, true, false, true),
            rest(),
            rest(),
            step(36, false, false, true),
            rest(),
            step(43, true, true, true),
            rest(),
            step(36, false, true, true),
            rest(),
            rest(),
            rest(),
        ],
        tempo: 135.0,
        cutoff: 800.0,
        resonance: 0.5,
        env_mod: 0.4,
        decay: 250.0,
        saw: true,
    },

    // 9. Rave Anthem
    // Big room energy
    Preset {
        name: "Rave Anthem",
        steps: [
            step(36, true, false, true),
            step(36, false, false, true),
            step(43, true, false, true),
            step(43, false, false, true),
            step(48, true, false, true),
            step(48, false, false, true),
            step(43, true, false, true),
            step(43, false, false, true),
            step(41, true, false, true),
            step(41, false, false, true),
            step(43, true, false, true),
            step(48, false, true, true),
            step(53, true, false, true),   // F3
            step(48, false, true, true),
            step(43, false, true, true),
            step(36, false, true, true),
        ],
        tempo: 140.0,
        cutoff: 550.0,
        resonance: 0.68,
        env_mod: 0.7,
        decay: 130.0,
        saw: true,
    },

    // 10. Warehouse
    // Dark, underground Chicago
    Preset {
        name: "Warehouse",
        steps: [
            step(33, true, false, true),   // A1
            rest(),
            step(33, false, false, true),
            step(36, false, true, true),   // C2 slide
            step(33, true, false, true),
            rest(),
            step(40, false, true, true),   // E2 slide
            step(33, false, true, true),
            step(33, true, false, true),
            rest(),
            step(33, false, false, true),
            step(45, false, true, true),   // A2 slide
            step(33, true, false, true),
            rest(),
            step(33, false, false, true),
            rest(),
        ],
        tempo: 122.0,
        cutoff: 380.0,
        resonance: 0.78,
        env_mod: 0.82,
        decay: 170.0,
        saw: true,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_exist() {
        assert!(!PRESETS.is_empty());
    }

    #[test]
    fn test_preset_names_unique() {
        let mut names: Vec<_> = PRESETS.iter().map(|p| p.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), PRESETS.len());
    }

    #[test]
    fn test_preset_parameters_valid() {
        for preset in PRESETS.iter() {
            assert!(preset.tempo >= 60.0 && preset.tempo <= 300.0);
            assert!(preset.cutoff >= 20.0 && preset.cutoff <= 20000.0);
            assert!(preset.resonance >= 0.0 && preset.resonance <= 1.0);
            assert!(preset.env_mod >= 0.0 && preset.env_mod <= 1.0);
            assert!(preset.decay >= 10.0 && preset.decay <= 5000.0);
        }
    }

    #[test]
    fn test_preset_patterns_have_notes() {
        for preset in PRESETS.iter() {
            let active_steps = preset.steps.iter().filter(|s| s.active).count();
            assert!(active_steps > 0, "Preset {} has no active steps", preset.name);
        }
    }

    #[test]
    fn test_acid_tracks_pattern() {
        let acid = &PRESETS[0];
        assert_eq!(acid.name, "Acid Tracks");
        assert!(acid.steps[0].accent); // First step accented
        assert!(acid.steps[2].slide);  // Third step slides
    }
}

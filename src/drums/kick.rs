use std::f32::consts::PI;

/// 808-style kick drum synthesizer
/// Uses a sine wave with pitch envelope for that deep boom
pub struct Kick {
    sample_rate: f32,
    phase: f32,

    // Envelope state
    amp_env: f32,
    pitch_env: f32,

    // Parameters
    base_freq: f32,      // Base frequency (40-80 Hz typical)
    pitch_decay: f32,    // How fast pitch drops
    amp_decay: f32,      // How fast amplitude drops
    pitch_amount: f32,   // How much pitch sweeps (in Hz)

    active: bool,
}

impl Kick {
    pub fn new(sample_rate: f32) -> Self {
        let mut kick = Self {
            sample_rate,
            phase: 0.0,
            amp_env: 0.0,
            pitch_env: 0.0,
            base_freq: 50.0,
            pitch_decay: 0.0,
            amp_decay: 0.0,
            pitch_amount: 150.0,
            active: false,
        };
        kick.set_decay(0.5);
        kick
    }

    pub fn trigger(&mut self) {
        self.phase = 0.0;
        self.amp_env = 1.0;
        self.pitch_env = 1.0;
        self.active = true;
    }

    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Calculate current frequency (base + pitch envelope)
        let freq = self.base_freq + (self.pitch_env * self.pitch_amount);

        // Generate sine wave
        let output = (self.phase * 2.0 * PI).sin();

        // Advance phase
        self.phase += freq / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Apply amplitude envelope
        let output = output * self.amp_env;

        // Decay envelopes
        self.amp_env *= self.amp_decay;
        self.pitch_env *= self.pitch_decay;

        // Stop when quiet enough
        if self.amp_env < 0.001 {
            self.active = false;
        }

        // Soft clip for extra punch
        soft_clip(output * 1.5)
    }

    /// Set decay time (0.0 = short, 1.0 = long boomy)
    pub fn set_decay(&mut self, decay: f32) {
        let decay = decay.clamp(0.0, 1.0);

        // Map to useful decay rates
        // Short: ~50ms, Long: ~500ms
        let amp_ms = 50.0 + decay * 450.0;
        let pitch_ms = 10.0 + decay * 40.0;

        let amp_samples = (amp_ms / 1000.0) * self.sample_rate;
        let pitch_samples = (pitch_ms / 1000.0) * self.sample_rate;

        self.amp_decay = 0.001_f32.powf(1.0 / amp_samples);
        self.pitch_decay = 0.001_f32.powf(1.0 / pitch_samples);
    }

    /// Set base pitch (0.0 = low 40Hz, 1.0 = high 80Hz)
    pub fn set_pitch(&mut self, pitch: f32) {
        let pitch = pitch.clamp(0.0, 1.0);
        self.base_freq = 40.0 + pitch * 40.0;
    }
}

fn soft_clip(x: f32) -> f32 {
    x.tanh()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kick_creation() {
        let kick = Kick::new(44100.0);
        assert!(!kick.active);
    }

    #[test]
    fn test_kick_trigger() {
        let mut kick = Kick::new(44100.0);
        kick.trigger();
        assert!(kick.active);

        // Should produce sound (process a few samples since sin(0) = 0)
        let mut has_sound = false;
        for _ in 0..100 {
            if kick.process().abs() > 0.01 {
                has_sound = true;
                break;
            }
        }
        assert!(has_sound);
    }

    #[test]
    fn test_kick_decays() {
        let mut kick = Kick::new(44100.0);
        kick.set_decay(0.0); // Short decay
        kick.trigger();

        // Process until silent
        for _ in 0..10000 {
            kick.process();
        }

        assert!(!kick.active);
    }

    #[test]
    fn test_kick_output_range() {
        let mut kick = Kick::new(44100.0);
        kick.trigger();

        for _ in 0..1000 {
            let sample = kick.process();
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }
}

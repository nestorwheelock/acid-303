use std::f32::consts::PI;

/// 909-style snare drum synthesizer
/// Combines a pitched tone with filtered noise for that crisp snap
pub struct Snare {
    sample_rate: f32,

    // Tone oscillator (body of the snare)
    tone_phase: f32,
    tone_freq: f32,
    tone_env: f32,
    tone_decay: f32,

    // Noise generator (snare wires)
    noise_state: u32,  // LFSR for noise
    noise_env: f32,
    noise_decay: f32,

    // Noise filter state (bandpass-ish)
    noise_hp_state: f32,
    noise_lp_state: f32,

    // Mix parameters
    tone_mix: f32,     // How much tone vs noise
    snap: f32,         // Attack sharpness

    active: bool,
}

impl Snare {
    pub fn new(sample_rate: f32) -> Self {
        let mut snare = Self {
            sample_rate,
            tone_phase: 0.0,
            tone_freq: 180.0,  // ~180Hz tone
            tone_env: 0.0,
            tone_decay: 0.0,
            noise_state: 0xACE1,
            noise_env: 0.0,
            noise_decay: 0.0,
            noise_hp_state: 0.0,
            noise_lp_state: 0.0,
            tone_mix: 0.4,
            snap: 0.7,
            active: false,
        };
        snare.set_decay(0.3);
        snare
    }

    pub fn trigger(&mut self) {
        self.tone_phase = 0.0;
        self.tone_env = 1.0;
        self.noise_env = 1.0;
        self.active = true;
    }

    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // === Tone (body) ===
        // Sine wave with quick pitch drop
        let tone_freq = self.tone_freq * (1.0 + self.tone_env * 0.5);
        let tone = (self.tone_phase * 2.0 * PI).sin();

        self.tone_phase += tone_freq / self.sample_rate;
        if self.tone_phase >= 1.0 {
            self.tone_phase -= 1.0;
        }

        // === Noise (snare wires) ===
        let noise = self.generate_noise();

        // Bandpass filter the noise (around 5-8kHz for snare sizzle)
        // Simple one-pole HP + LP cascade
        let hp_coeff = 0.95;  // High pass ~500Hz
        let lp_coeff = 0.3;   // Low pass ~8kHz

        self.noise_hp_state = hp_coeff * (self.noise_hp_state + noise - self.noise_lp_state);
        let hp_out = self.noise_hp_state;

        self.noise_lp_state += lp_coeff * (hp_out - self.noise_lp_state);
        let filtered_noise = self.noise_lp_state;

        // === Mix ===
        let tone_out = tone * self.tone_env * self.tone_mix;
        let noise_out = filtered_noise * self.noise_env * (1.0 - self.tone_mix * 0.5);

        let output = tone_out + noise_out;

        // === Decay ===
        self.tone_env *= self.tone_decay;
        self.noise_env *= self.noise_decay;

        if self.tone_env < 0.001 && self.noise_env < 0.001 {
            self.active = false;
        }

        // Add some drive/saturation
        soft_clip(output * 2.0) * 0.7
    }

    /// Generate white noise using LFSR
    fn generate_noise(&mut self) -> f32 {
        // 16-bit LFSR with taps at 16, 14, 13, 11
        let bit = ((self.noise_state >> 0) ^ (self.noise_state >> 2)
                 ^ (self.noise_state >> 3) ^ (self.noise_state >> 5)) & 1;
        self.noise_state = (self.noise_state >> 1) | (bit << 15);

        // Convert to float in range -1 to 1
        (self.noise_state as f32 / 32768.0) - 1.0
    }

    /// Set overall decay (0.0 = tight, 1.0 = long)
    pub fn set_decay(&mut self, decay: f32) {
        let decay = decay.clamp(0.0, 1.0);

        // Tone decays faster than noise
        let tone_ms = 30.0 + decay * 100.0;
        let noise_ms = 50.0 + decay * 200.0;

        let tone_samples = (tone_ms / 1000.0) * self.sample_rate;
        let noise_samples = (noise_ms / 1000.0) * self.sample_rate;

        self.tone_decay = 0.001_f32.powf(1.0 / tone_samples);
        self.noise_decay = 0.001_f32.powf(1.0 / noise_samples);
    }

    /// Set tone amount (0.0 = all noise, 1.0 = more body)
    pub fn set_tone(&mut self, tone: f32) {
        self.tone_mix = tone.clamp(0.0, 1.0);
    }

    /// Set snap/attack (0.0 = soft, 1.0 = sharp transient)
    pub fn set_snap(&mut self, snap: f32) {
        self.snap = snap.clamp(0.0, 1.0);
        // Snap affects the initial noise burst
    }
}

fn soft_clip(x: f32) -> f32 {
    x.tanh()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snare_creation() {
        let snare = Snare::new(44100.0);
        assert!(!snare.active);
    }

    #[test]
    fn test_snare_trigger() {
        let mut snare = Snare::new(44100.0);
        snare.trigger();
        assert!(snare.active);

        let sample = snare.process();
        assert!(sample.abs() > 0.0);
    }

    #[test]
    fn test_snare_decays() {
        let mut snare = Snare::new(44100.0);
        snare.set_decay(0.0);
        snare.trigger();

        for _ in 0..20000 {
            snare.process();
        }

        assert!(!snare.active);
    }

    #[test]
    fn test_snare_output_range() {
        let mut snare = Snare::new(44100.0);
        snare.trigger();

        for _ in 0..1000 {
            let sample = snare.process();
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_noise_varies() {
        let mut snare = Snare::new(44100.0);
        let n1 = snare.generate_noise();
        let n2 = snare.generate_noise();
        let n3 = snare.generate_noise();

        // Noise should produce different values
        assert!(n1 != n2 || n2 != n3);
    }
}

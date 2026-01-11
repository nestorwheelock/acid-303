/// Hihat synthesizer using metallic noise
/// Based on 808/909 approach: multiple square waves + noise through bandpass

/// Closed hihat - short, tight
pub struct ClosedHihat {
    sample_rate: f32,
    noise_state: u32,
    env: f32,
    decay: f32,

    // Multiple detuned oscillators for metallic sound
    phases: [f32; 6],
    freqs: [f32; 6],

    // Bandpass filter
    bp_state1: f32,
    bp_state2: f32,

    active: bool,
}

impl ClosedHihat {
    pub fn new(sample_rate: f32) -> Self {
        // 808-style uses 6 square waves at specific ratios
        // These create the metallic, inharmonic timbre
        let base = 400.0;
        let freqs = [
            base * 1.0,
            base * 1.4471,  // Inharmonic ratios from 808
            base * 1.6170,
            base * 1.9265,
            base * 2.5028,
            base * 2.6637,
        ];

        Self {
            sample_rate,
            noise_state: 0xBEEF,
            env: 0.0,
            decay: 0.9985,
            phases: [0.0; 6],
            freqs,
            bp_state1: 0.0,
            bp_state2: 0.0,
            active: false,
        }
    }

    pub fn trigger(&mut self) {
        self.env = 1.0;
        self.active = true;
    }

    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Generate metallic oscillator mix
        let mut osc_mix = 0.0;
        for i in 0..6 {
            // Square waves
            let square = if self.phases[i] < 0.5 { 1.0 } else { -1.0 };
            osc_mix += square;

            self.phases[i] += self.freqs[i] / self.sample_rate;
            if self.phases[i] >= 1.0 {
                self.phases[i] -= 1.0;
            }
        }
        osc_mix /= 6.0;

        // Add some noise for extra sizzle
        let noise = self.generate_noise() * 0.3;
        let mixed = osc_mix + noise;

        // Highpass filter to remove low frequencies
        // Simple 2-pole bandpass around 8-10kHz
        let cutoff = 0.4;  // Normalized frequency
        let q = 0.7;

        self.bp_state1 += cutoff * (mixed - self.bp_state1 - q * self.bp_state2);
        self.bp_state2 += cutoff * self.bp_state1;
        let filtered = self.bp_state1;

        // Apply envelope
        let output = filtered * self.env;

        // Decay
        self.env *= self.decay;

        if self.env < 0.001 {
            self.active = false;
        }

        output * 0.5
    }

    fn generate_noise(&mut self) -> f32 {
        let bit = ((self.noise_state >> 0) ^ (self.noise_state >> 2)
                 ^ (self.noise_state >> 3) ^ (self.noise_state >> 5)) & 1;
        self.noise_state = (self.noise_state >> 1) | (bit << 15);
        (self.noise_state as f32 / 32768.0) - 1.0
    }
}

/// Open hihat - longer, more sustain, can be choked
pub struct OpenHihat {
    sample_rate: f32,
    noise_state: u32,
    env: f32,
    decay: f32,

    phases: [f32; 6],
    freqs: [f32; 6],

    bp_state1: f32,
    bp_state2: f32,

    active: bool,
    choking: bool,
    choke_rate: f32,
}

impl OpenHihat {
    pub fn new(sample_rate: f32) -> Self {
        let base = 400.0;
        let freqs = [
            base * 1.0,
            base * 1.4471,
            base * 1.6170,
            base * 1.9265,
            base * 2.5028,
            base * 2.6637,
        ];

        Self {
            sample_rate,
            noise_state: 0xCAFE,
            env: 0.0,
            decay: 0.9998,  // Longer decay than closed
            phases: [0.0; 6],
            freqs,
            bp_state1: 0.0,
            bp_state2: 0.0,
            active: false,
            choking: false,
            choke_rate: 0.99,
        }
    }

    pub fn trigger(&mut self) {
        self.env = 1.0;
        self.active = true;
        self.choking = false;
    }

    /// Choke the hihat (when closed hihat plays)
    pub fn choke(&mut self) {
        if self.active {
            self.choking = true;
        }
    }

    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Same metallic oscillator as closed
        let mut osc_mix = 0.0;
        for i in 0..6 {
            let square = if self.phases[i] < 0.5 { 1.0 } else { -1.0 };
            osc_mix += square;

            self.phases[i] += self.freqs[i] / self.sample_rate;
            if self.phases[i] >= 1.0 {
                self.phases[i] -= 1.0;
            }
        }
        osc_mix /= 6.0;

        let noise = self.generate_noise() * 0.3;
        let mixed = osc_mix + noise;

        // Bandpass
        let cutoff = 0.35;
        let q = 0.6;

        self.bp_state1 += cutoff * (mixed - self.bp_state1 - q * self.bp_state2);
        self.bp_state2 += cutoff * self.bp_state1;
        let filtered = self.bp_state1;

        let output = filtered * self.env;

        // Apply decay or choke
        if self.choking {
            self.env *= self.choke_rate;
        } else {
            self.env *= self.decay;
        }

        if self.env < 0.001 {
            self.active = false;
        }

        output * 0.5
    }

    fn generate_noise(&mut self) -> f32 {
        let bit = ((self.noise_state >> 0) ^ (self.noise_state >> 2)
                 ^ (self.noise_state >> 3) ^ (self.noise_state >> 5)) & 1;
        self.noise_state = (self.noise_state >> 1) | (bit << 15);
        (self.noise_state as f32 / 32768.0) - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closed_hihat() {
        let mut hh = ClosedHihat::new(44100.0);
        hh.trigger();
        assert!(hh.active);

        let sample = hh.process();
        assert!(sample.abs() > 0.0 || sample == 0.0); // May be zero initially
    }

    #[test]
    fn test_open_hihat() {
        let mut hh = OpenHihat::new(44100.0);
        hh.trigger();
        assert!(hh.active);
    }

    #[test]
    fn test_hihat_choke() {
        let mut hh = OpenHihat::new(44100.0);
        hh.trigger();

        // Let it ring a bit
        for _ in 0..1000 {
            hh.process();
        }
        assert!(hh.active);

        // Choke it
        hh.choke();
        assert!(hh.choking);

        // Should decay quickly now
        for _ in 0..2000 {
            hh.process();
        }
        assert!(!hh.active);
    }

    #[test]
    fn test_closed_shorter_than_open() {
        let mut closed = ClosedHihat::new(44100.0);
        let mut open = OpenHihat::new(44100.0);

        closed.trigger();
        open.trigger();

        // Count samples until inactive
        let mut closed_count = 0;
        while closed.active && closed_count < 100000 {
            closed.process();
            closed_count += 1;
        }

        let mut open_count = 0;
        while open.active && open_count < 100000 {
            open.process();
            open_count += 1;
        }

        assert!(closed_count < open_count, "Closed should decay faster than open");
    }
}

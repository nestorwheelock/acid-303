/// Decay-only envelope generator
/// The 303 uses a simple decay envelope for the filter
pub struct Envelope {
    sample_rate: f32,
    value: f32,
    decay_rate: f32,
    peak: f32,
}

impl Envelope {
    pub fn new(sample_rate: f32) -> Self {
        let mut env = Self {
            sample_rate,
            value: 0.0,
            decay_rate: 0.0,
            peak: 1.0,
        };
        env.set_decay(200.0); // Default 200ms decay
        env
    }

    /// Set decay time in milliseconds
    pub fn set_decay(&mut self, ms: f32) {
        let ms = ms.clamp(10.0, 5000.0);
        // Calculate decay rate for exponential decay
        // After `ms` milliseconds, value should be at ~1% of peak
        let samples = (ms / 1000.0) * self.sample_rate;
        // decay^samples = 0.01 (1% remaining)
        // decay = 0.01^(1/samples)
        self.decay_rate = 0.01_f32.powf(1.0 / samples);
    }

    /// Trigger the envelope with optional accent multiplier
    pub fn trigger(&mut self, accent_mult: f32) {
        self.peak = accent_mult.clamp(0.5, 2.0);
        self.value = self.peak;
    }

    /// Process one sample
    pub fn process(&mut self) -> f32 {
        let output = self.value;

        // Exponential decay
        self.value *= self.decay_rate;

        // Floor very small values to zero
        if self.value < 0.0001 {
            self.value = 0.0;
        }

        output
    }

    /// Get current envelope value without advancing
    pub fn current(&self) -> f32 {
        self.value
    }

    /// Check if envelope is active
    pub fn is_active(&self) -> bool {
        self.value > 0.0001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_creation() {
        let env = Envelope::new(44100.0);
        assert_eq!(env.value, 0.0);
    }

    #[test]
    fn test_envelope_trigger() {
        let mut env = Envelope::new(44100.0);
        env.trigger(1.0);
        assert_eq!(env.value, 1.0);
    }

    #[test]
    fn test_envelope_decay() {
        let mut env = Envelope::new(44100.0);
        env.set_decay(100.0); // 100ms decay
        env.trigger(1.0);

        let initial = env.process();
        assert_eq!(initial, 1.0);

        // After some samples, value should be lower
        for _ in 0..1000 {
            env.process();
        }

        assert!(env.current() < 0.5);
    }

    #[test]
    fn test_envelope_reaches_zero() {
        let mut env = Envelope::new(44100.0);
        env.set_decay(50.0); // Short decay
        env.trigger(1.0);

        // Process enough samples for decay to complete
        for _ in 0..10000 {
            env.process();
        }

        assert!(env.current() < 0.001);
        assert!(!env.is_active());
    }

    #[test]
    fn test_accent_boost() {
        let mut env = Envelope::new(44100.0);

        env.trigger(1.0);
        let normal_peak = env.current();

        env.trigger(1.5);
        let accent_peak = env.current();

        assert!(accent_peak > normal_peak);
    }
}

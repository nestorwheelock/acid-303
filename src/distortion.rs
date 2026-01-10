/// Soft clipping distortion/overdrive
/// Adds warmth and grit to the 303 sound
pub struct Distortion {
    drive: f32,
    mix: f32,
}

impl Distortion {
    pub fn new() -> Self {
        Self {
            drive: 0.3,
            mix: 1.0,
        }
    }

    /// Set drive amount (0.0 to 1.0)
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.clamp(0.0, 1.0);
    }

    /// Set wet/dry mix (0.0 = dry, 1.0 = wet)
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        if self.drive < 0.01 {
            return input;
        }

        // Scale input by drive amount
        // Higher drive = more gain before clipping
        let gain = 1.0 + self.drive * 10.0;
        let driven = input * gain;

        // Soft clipping using tanh
        // This gives a warm, tube-like saturation
        let clipped = driven.tanh();

        // Compensate for volume increase
        let compensated = clipped / (1.0 + self.drive * 0.5);

        // Mix dry and wet
        input * (1.0 - self.mix) + compensated * self.mix
    }
}

impl Default for Distortion {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distortion_creation() {
        let dist = Distortion::new();
        assert_eq!(dist.drive, 0.3);
    }

    #[test]
    fn test_no_distortion_passthrough() {
        let mut dist = Distortion::new();
        dist.set_drive(0.0);

        let input = 0.5;
        let output = dist.process(input);

        assert!((output - input).abs() < 0.01);
    }

    #[test]
    fn test_distortion_clips() {
        let mut dist = Distortion::new();
        dist.set_drive(1.0);

        // Large input should be clipped
        let output = dist.process(1.0);
        assert!(output < 1.0);

        // Very large input should still be bounded
        let output2 = dist.process(10.0);
        assert!(output2 < 2.0);
    }

    #[test]
    fn test_distortion_symmetric() {
        let mut dist = Distortion::new();
        dist.set_drive(0.5);

        let pos = dist.process(0.5);
        let neg = dist.process(-0.5);

        assert!((pos + neg).abs() < 0.01);
    }

    #[test]
    fn test_drive_range() {
        let mut dist = Distortion::new();

        dist.set_drive(-1.0);
        assert_eq!(dist.drive, 0.0);

        dist.set_drive(5.0);
        assert_eq!(dist.drive, 1.0);
    }
}

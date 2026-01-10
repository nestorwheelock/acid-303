use std::f32::consts::PI;

/// 18dB/octave (3-pole) resonant lowpass filter
/// Emulates the distinctive TB-303 filter sound
pub struct Filter {
    sample_rate: f32,
    cutoff: f32,
    resonance: f32,

    // 3 cascaded one-pole filter states
    s1: f32,
    s2: f32,
    s3: f32,

    // Coefficients
    g: f32,  // filter coefficient
    k: f32,  // resonance coefficient
}

impl Filter {
    pub fn new(sample_rate: f32) -> Self {
        let mut filter = Self {
            sample_rate,
            cutoff: 1000.0,
            resonance: 0.0,
            s1: 0.0,
            s2: 0.0,
            s3: 0.0,
            g: 0.0,
            k: 0.0,
        };
        filter.update_coefficients();
        filter
    }

    pub fn set_cutoff(&mut self, freq: f32) {
        self.cutoff = freq.clamp(20.0, self.sample_rate * 0.49);
        self.update_coefficients();
    }

    pub fn set_resonance(&mut self, res: f32) {
        self.resonance = res.clamp(0.0, 1.0);
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        // Compute filter coefficient using tan approximation for stability
        let wc = 2.0 * PI * self.cutoff / self.sample_rate;
        self.g = wc.tan();

        // Resonance: map 0-1 to useful range (0 to ~4 for self-oscillation)
        // The 303 can self-oscillate at high resonance
        self.k = self.resonance * 4.0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // 3-pole ladder filter with resonance feedback
        // Based on simplified Moog ladder topology adapted for 3 poles

        // Feedback path - take from output of 3rd stage
        let feedback = self.s3;

        // Saturate the feedback for analog-like behavior
        let saturated_feedback = (self.k * feedback).tanh();

        // Input with resonance feedback
        let u = input - saturated_feedback;

        // Cascade of 3 one-pole lowpass filters
        // Each stage: y = g * (x - y) + y, simplified to y += g * (x - y)
        let g_factor = self.g / (1.0 + self.g);

        // Stage 1
        self.s1 += g_factor * (u - self.s1);

        // Stage 2
        self.s2 += g_factor * (self.s1 - self.s2);

        // Stage 3
        self.s3 += g_factor * (self.s2 - self.s3);

        // Output from 3rd pole gives us 18dB/octave
        // Apply soft clipping to prevent harsh clipping at high resonance
        soft_clip(self.s3)
    }

    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
        self.s3 = 0.0;
    }
}

/// Soft clipping function for analog-like saturation
fn soft_clip(x: f32) -> f32 {
    if x > 1.0 {
        1.0 - (-x + 1.0).exp() * 0.5
    } else if x < -1.0 {
        -1.0 + (x + 1.0).exp() * 0.5
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_creation() {
        let filter = Filter::new(44100.0);
        assert_eq!(filter.cutoff, 1000.0);
        assert_eq!(filter.resonance, 0.0);
    }

    #[test]
    fn test_filter_attenuates_highs() {
        let mut filter = Filter::new(44100.0);
        filter.set_cutoff(200.0);

        // Feed in a high frequency square wave
        let mut sum_output = 0.0f32;
        let mut sum_input = 0.0f32;

        for i in 0..1000 {
            let input: f32 = if i % 5 < 2 { 1.0 } else { -1.0 }; // ~8820 Hz
            sum_input += input.abs();
            sum_output += filter.process(input).abs();
        }

        // Output should be much smaller than input (attenuated)
        assert!(sum_output < sum_input * 0.5);
    }

    #[test]
    fn test_filter_passes_lows() {
        let mut filter = Filter::new(44100.0);
        filter.set_cutoff(5000.0);

        // Feed in a low frequency wave
        let mut sum_output = 0.0f32;
        let mut sum_input = 0.0f32;

        for i in 0..1000 {
            let input = ((i as f32) * 0.01).sin(); // Very low frequency
            sum_input += input.abs();
            sum_output += filter.process(input).abs();
        }

        // Output should be close to input (passed through)
        assert!(sum_output > sum_input * 0.5);
    }

    #[test]
    fn test_resonance_boost() {
        let mut filter_no_res = Filter::new(44100.0);
        filter_no_res.set_cutoff(1000.0);
        filter_no_res.set_resonance(0.0);

        let mut filter_res = Filter::new(44100.0);
        filter_res.set_cutoff(1000.0);
        filter_res.set_resonance(0.8);

        // At cutoff frequency, resonance should boost
        let mut max_no_res = 0.0f32;
        let mut max_res = 0.0f32;

        for i in 0..1000 {
            let freq = 1000.0;
            let input = (2.0 * PI * freq * (i as f32) / 44100.0).sin();
            max_no_res = max_no_res.max(filter_no_res.process(input).abs());
            max_res = max_res.max(filter_res.process(input).abs());
        }

        // Resonant filter should have higher peak
        assert!(max_res > max_no_res);
    }

    #[test]
    fn test_soft_clip() {
        assert!((soft_clip(0.5) - 0.5).abs() < 0.01);
        assert!(soft_clip(2.0) < 1.1);
        assert!(soft_clip(-2.0) > -1.1);
    }
}

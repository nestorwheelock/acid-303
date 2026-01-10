#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Waveform {
    Saw,
    Square,
}

/// Band-limited oscillator using PolyBLEP for anti-aliasing
pub struct Oscillator {
    sample_rate: f32,
    phase: f32,
    frequency: f32,
    waveform: Waveform,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            frequency: 440.0,
            waveform: Waveform::Saw,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq.clamp(20.0, 20000.0);
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn process(&mut self) -> f32 {
        let phase_inc = self.frequency / self.sample_rate;

        let output = match self.waveform {
            Waveform::Saw => self.saw_polyblep(phase_inc),
            Waveform::Square => self.square_polyblep(phase_inc),
        };

        // Advance phase
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }

    /// Sawtooth wave with PolyBLEP anti-aliasing
    fn saw_polyblep(&self, phase_inc: f32) -> f32 {
        // Naive sawtooth: goes from -1 to 1 over one period
        let naive = 2.0 * self.phase - 1.0;

        // Apply PolyBLEP correction at discontinuity (phase = 0/1)
        naive - self.polyblep(self.phase, phase_inc)
    }

    /// Square wave with PolyBLEP anti-aliasing
    fn square_polyblep(&self, phase_inc: f32) -> f32 {
        // Naive square: +1 for first half, -1 for second half
        let naive = if self.phase < 0.5 { 1.0 } else { -1.0 };

        // Apply PolyBLEP at both transitions
        let mut output = naive;
        output += self.polyblep(self.phase, phase_inc);
        output -= self.polyblep((self.phase + 0.5) % 1.0, phase_inc);

        output
    }

    /// PolyBLEP (polynomial band-limited step) correction
    /// Smooths discontinuities to reduce aliasing
    fn polyblep(&self, t: f32, dt: f32) -> f32 {
        if t < dt {
            // Right after discontinuity
            let t = t / dt;
            2.0 * t - t * t - 1.0
        } else if t > 1.0 - dt {
            // Right before discontinuity
            let t = (t - 1.0) / dt;
            t * t + 2.0 * t + 1.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = Oscillator::new(44100.0);
        assert_eq!(osc.frequency, 440.0);
        assert_eq!(osc.waveform, Waveform::Saw);
    }

    #[test]
    fn test_saw_output_range() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);

        for _ in 0..1000 {
            let sample = osc.process();
            assert!(sample >= -1.5 && sample <= 1.5);
        }
    }

    #[test]
    fn test_square_output_range() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Square);
        osc.set_frequency(440.0);

        for _ in 0..1000 {
            let sample = osc.process();
            assert!(sample >= -1.5 && sample <= 1.5);
        }
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency, 880.0);

        // Test clamping
        osc.set_frequency(5.0);
        assert_eq!(osc.frequency, 20.0);

        osc.set_frequency(25000.0);
        assert_eq!(osc.frequency, 20000.0);
    }
}

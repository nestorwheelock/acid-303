const STEPS: usize = 16;
const SAMPLE_RATE: f32 = 44100.0;

/// A single step in the sequencer
#[derive(Clone, Copy, Debug, Default)]
pub struct Step {
    pub note: u8,     // MIDI note number
    pub accent: bool, // Accent this step
    pub slide: bool,  // Slide to this note from previous
    pub active: bool, // Step is on/off
}

/// 16-step sequencer
pub struct Sequencer {
    steps: [Step; STEPS],
    current: usize,
    sample_counter: u32,
    samples_per_step: u32,
    playing: bool,
    tempo: f32,
}

impl Sequencer {
    pub fn new() -> Self {
        // Initialize with default pattern (all C2, no accents/slides)
        let default_step = Step {
            note: 36, // C2
            accent: false,
            slide: false,
            active: false,
        };

        Self {
            steps: [default_step; STEPS],
            current: 0,
            sample_counter: 0,
            samples_per_step: 0,
            playing: false,
            tempo: 120.0,
        }
    }

    pub fn set_tempo(&mut self, bpm: f32) {
        self.tempo = bpm.clamp(60.0, 300.0);
        // 16th notes at given BPM
        // beats per second = bpm / 60
        // 16th notes per second = (bpm / 60) * 4
        // samples per 16th = sample_rate / (16ths per second)
        let sixteenths_per_second = (self.tempo / 60.0) * 4.0;
        self.samples_per_step = (SAMPLE_RATE / sixteenths_per_second) as u32;
    }

    pub fn set_step(&mut self, index: usize, step: Step) {
        if index < STEPS {
            self.steps[index] = step;
        }
    }

    pub fn get_step(&self, index: usize) -> Option<&Step> {
        self.steps.get(index)
    }

    pub fn start(&mut self) {
        self.playing = true;
        self.current = 0;
        self.sample_counter = 0;
        self.set_tempo(self.tempo); // Recalculate samples_per_step
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn current_step(&self) -> usize {
        self.current
    }

    /// Tick the sequencer. Returns Some(Step) when advancing to a new step.
    pub fn tick(&mut self) -> Option<Step> {
        if !self.playing || self.samples_per_step == 0 {
            return None;
        }

        self.sample_counter += 1;

        if self.sample_counter >= self.samples_per_step {
            self.sample_counter = 0;
            let step = self.steps[self.current];
            self.current = (self.current + 1) % STEPS;
            Some(step)
        } else {
            None
        }
    }

    /// Load a pattern from an array of steps
    pub fn load_pattern(&mut self, pattern: &[Step; STEPS]) {
        self.steps = *pattern;
    }

    /// Clear the pattern
    pub fn clear(&mut self) {
        for step in &mut self.steps {
            step.active = false;
        }
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_creation() {
        let seq = Sequencer::new();
        assert!(!seq.is_playing());
        assert_eq!(seq.current_step(), 0);
    }

    #[test]
    fn test_sequencer_start_stop() {
        let mut seq = Sequencer::new();
        seq.start();
        assert!(seq.is_playing());
        seq.stop();
        assert!(!seq.is_playing());
    }

    #[test]
    fn test_sequencer_advances() {
        let mut seq = Sequencer::new();
        seq.set_tempo(120.0);
        seq.set_step(0, Step { note: 48, accent: true, slide: false, active: true });
        seq.start();

        // Tick until we get a step
        let mut step_received = false;
        for _ in 0..50000 {
            if let Some(step) = seq.tick() {
                step_received = true;
                assert_eq!(step.note, 48);
                assert!(step.accent);
                break;
            }
        }
        assert!(step_received);
    }

    #[test]
    fn test_sequencer_wraps() {
        let mut seq = Sequencer::new();
        seq.set_tempo(300.0); // Fast tempo for quick test
        seq.start();

        let mut wrap_count = 0;
        let mut last_step = 0;

        for _ in 0..500000 {
            if seq.tick().is_some() {
                if seq.current_step() < last_step {
                    wrap_count += 1;
                }
                last_step = seq.current_step();
            }
            if wrap_count >= 2 {
                break;
            }
        }

        assert!(wrap_count >= 2, "Sequencer should wrap around");
    }

    #[test]
    fn test_tempo_change() {
        let mut seq = Sequencer::new();

        seq.set_tempo(60.0);
        let slow_samples = seq.samples_per_step;

        seq.set_tempo(120.0);
        let fast_samples = seq.samples_per_step;

        // Faster tempo = fewer samples per step
        assert!(fast_samples < slow_samples);
    }
}

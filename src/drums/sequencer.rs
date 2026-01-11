const STEPS: usize = 16;
const SAMPLE_RATE: f32 = 44100.0;

/// Which drums are active on a step
#[derive(Clone, Copy, Debug, Default)]
pub struct DrumStep {
    pub kick: bool,
    pub snare: bool,
    pub closed_hh: bool,
    pub open_hh: bool,
}

/// Which track we're editing
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrumTrack {
    Kick,
    Snare,
    ClosedHH,
    OpenHH,
}

/// 16-step drum sequencer with 4 tracks
pub struct DrumSequencer {
    steps: [DrumStep; STEPS],
    current: usize,
    sample_counter: u32,
    samples_per_step: u32,
    playing: bool,
    tempo: f32,
}

impl DrumSequencer {
    pub fn new() -> Self {
        let mut seq = Self {
            steps: [DrumStep::default(); STEPS],
            current: 0,
            sample_counter: 0,
            samples_per_step: 0,
            playing: false,
            tempo: 120.0,
        };
        seq.set_tempo(120.0);

        // Initialize with a basic 4/4 beat
        seq.load_pattern(&BASIC_BEAT);

        seq
    }

    pub fn set_tempo(&mut self, bpm: f32) {
        self.tempo = bpm.clamp(60.0, 300.0);
        let sixteenths_per_second = (self.tempo / 60.0) * 4.0;
        self.samples_per_step = (SAMPLE_RATE / sixteenths_per_second) as u32;
    }

    pub fn set_step(&mut self, index: usize, track: DrumTrack, active: bool) {
        if index < STEPS {
            match track {
                DrumTrack::Kick => self.steps[index].kick = active,
                DrumTrack::Snare => self.steps[index].snare = active,
                DrumTrack::ClosedHH => self.steps[index].closed_hh = active,
                DrumTrack::OpenHH => self.steps[index].open_hh = active,
            }
        }
    }

    pub fn toggle_step(&mut self, index: usize, track: DrumTrack) {
        if index < STEPS {
            match track {
                DrumTrack::Kick => self.steps[index].kick = !self.steps[index].kick,
                DrumTrack::Snare => self.steps[index].snare = !self.steps[index].snare,
                DrumTrack::ClosedHH => self.steps[index].closed_hh = !self.steps[index].closed_hh,
                DrumTrack::OpenHH => self.steps[index].open_hh = !self.steps[index].open_hh,
            }
        }
    }

    pub fn get_step(&self, index: usize) -> Option<&DrumStep> {
        self.steps.get(index)
    }

    pub fn start(&mut self) {
        self.playing = true;
        self.current = 0;
        self.sample_counter = 0;
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

    /// Tick the sequencer. Returns Some(DrumStep) when advancing.
    pub fn tick(&mut self) -> Option<DrumStep> {
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

    pub fn load_pattern(&mut self, pattern: &[DrumStep; STEPS]) {
        self.steps = *pattern;
    }

    pub fn clear(&mut self) {
        self.steps = [DrumStep::default(); STEPS];
    }
}

impl Default for DrumSequencer {
    fn default() -> Self {
        Self::new()
    }
}

// ============== PRESET PATTERNS ==============

/// Helper to create drum steps
const fn d(kick: bool, snare: bool, closed_hh: bool, open_hh: bool) -> DrumStep {
    DrumStep { kick, snare, closed_hh, open_hh }
}

/// Basic 4/4 house beat
pub static BASIC_BEAT: [DrumStep; 16] = [
    d(true,  false, true,  false),  // 1
    d(false, false, true,  false),
    d(false, false, true,  false),
    d(false, false, true,  false),
    d(true,  true,  true,  false),  // 2
    d(false, false, true,  false),
    d(false, false, true,  false),
    d(false, false, true,  false),
    d(true,  false, true,  false),  // 3
    d(false, false, true,  false),
    d(false, false, true,  false),
    d(false, false, true,  false),
    d(true,  true,  true,  false),  // 4
    d(false, false, true,  false),
    d(false, false, true,  false),
    d(false, false, true,  false),
];

/// Breakbeat pattern
pub static BREAKBEAT: [DrumStep; 16] = [
    d(true,  false, true,  false),  // 1
    d(false, false, false, true ),
    d(false, true,  true,  false),
    d(false, false, true,  false),
    d(false, false, true,  false),  // 2
    d(false, true,  false, true ),
    d(true,  false, true,  false),
    d(false, false, true,  false),
    d(false, false, true,  false),  // 3
    d(false, false, false, true ),
    d(false, true,  true,  false),
    d(false, false, true,  false),
    d(true,  false, true,  false),  // 4
    d(false, true,  false, true ),
    d(false, false, true,  false),
    d(true,  false, true,  false),
];

/// Classic 909 house
pub static HOUSE_909: [DrumStep; 16] = [
    d(true,  false, true,  false),
    d(false, false, true,  false),
    d(false, false, false, true ),
    d(false, false, true,  false),
    d(true,  true,  true,  false),
    d(false, false, true,  false),
    d(false, false, false, true ),
    d(false, false, true,  false),
    d(true,  false, true,  false),
    d(false, false, true,  false),
    d(false, false, false, true ),
    d(false, false, true,  false),
    d(true,  true,  true,  false),
    d(false, false, true,  false),
    d(false, false, false, true ),
    d(true,  false, true,  false),
];

/// Minimal techno
pub static MINIMAL: [DrumStep; 16] = [
    d(true,  false, true,  false),
    d(false, false, false, false),
    d(false, false, true,  false),
    d(false, false, false, false),
    d(true,  false, true,  false),
    d(false, false, false, false),
    d(false, false, true,  false),
    d(false, true,  false, false),
    d(true,  false, true,  false),
    d(false, false, false, false),
    d(false, false, true,  false),
    d(false, false, false, false),
    d(true,  false, true,  false),
    d(false, false, false, false),
    d(false, true,  true,  false),
    d(false, false, false, false),
];

/// Driving acid
pub static ACID_DRIVE: [DrumStep; 16] = [
    d(true,  false, true,  false),
    d(false, false, true,  false),
    d(true,  false, true,  false),
    d(false, false, true,  false),
    d(true,  true,  true,  false),
    d(false, false, true,  false),
    d(true,  false, true,  false),
    d(false, false, false, true ),
    d(true,  false, true,  false),
    d(false, false, true,  false),
    d(true,  false, true,  false),
    d(false, false, true,  false),
    d(true,  true,  true,  false),
    d(false, false, true,  false),
    d(true,  false, false, true ),
    d(false, false, true,  false),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_creation() {
        let seq = DrumSequencer::new();
        assert!(!seq.is_playing());
    }

    #[test]
    fn test_sequencer_start_stop() {
        let mut seq = DrumSequencer::new();
        seq.start();
        assert!(seq.is_playing());
        seq.stop();
        assert!(!seq.is_playing());
    }

    #[test]
    fn test_toggle_step() {
        let mut seq = DrumSequencer::new();
        seq.clear();

        assert!(!seq.steps[0].kick);
        seq.toggle_step(0, DrumTrack::Kick);
        assert!(seq.steps[0].kick);
        seq.toggle_step(0, DrumTrack::Kick);
        assert!(!seq.steps[0].kick);
    }

    #[test]
    fn test_basic_beat_has_kicks() {
        let kick_count = BASIC_BEAT.iter().filter(|s| s.kick).count();
        assert_eq!(kick_count, 4, "4/4 should have 4 kicks");
    }

    #[test]
    fn test_sequencer_advances() {
        let mut seq = DrumSequencer::new();
        seq.set_tempo(120.0);
        seq.start();

        let mut step_received = false;
        for _ in 0..50000 {
            if seq.tick().is_some() {
                step_received = true;
                break;
            }
        }
        assert!(step_received);
    }
}

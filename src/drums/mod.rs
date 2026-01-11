mod kick;
mod snare;
mod hihat;
pub mod sequencer;

pub use kick::Kick;
pub use snare::Snare;
pub use hihat::{ClosedHihat, OpenHihat};
pub use sequencer::{DrumSequencer, DrumTrack};
pub use sequencer::{BASIC_BEAT, BREAKBEAT, HOUSE_909, MINIMAL, ACID_DRIVE};
pub use sequencer::{
    INTRO_KICK, INTRO_HATS, BUILD_SNARE, BUILD_ROLL,
    BREAKDOWN, BREAKDOWN_KICK, FILL_SNARE, FILL_STOMP,
    FILL_OPEN_HAT, DROP_FULL, OFFBEAT_HOUSE, SHUFFLE
};

/// Complete drum machine with 808/909 style sounds
pub struct DrumMachine {
    pub kick: Kick,
    pub snare: Snare,
    pub closed_hh: ClosedHihat,
    pub open_hh: OpenHihat,
    pub sequencer: DrumSequencer,

    // Volumes (0.0 - 1.0)
    kick_vol: f32,
    snare_vol: f32,
    hh_vol: f32,
    master_vol: f32,
}

impl DrumMachine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            kick: Kick::new(sample_rate),
            snare: Snare::new(sample_rate),
            closed_hh: ClosedHihat::new(sample_rate),
            open_hh: OpenHihat::new(sample_rate),
            sequencer: DrumSequencer::new(),
            kick_vol: 0.8,
            snare_vol: 0.7,
            hh_vol: 0.5,
            master_vol: 0.8,
        }
    }

    /// Process one sample of audio
    pub fn process(&mut self) -> f32 {
        let kick = self.kick.process() * self.kick_vol;
        let snare = self.snare.process() * self.snare_vol;
        let closed = self.closed_hh.process() * self.hh_vol;
        let open = self.open_hh.process() * self.hh_vol;

        (kick + snare + closed + open) * self.master_vol
    }

    /// Tick the sequencer, trigger drums as needed
    pub fn tick(&mut self) -> Option<usize> {
        if let Some(step) = self.sequencer.tick() {
            if step.kick {
                self.kick.trigger();
            }
            if step.snare {
                self.snare.trigger();
            }
            if step.closed_hh {
                // Close open hihat when closed hihat plays
                self.open_hh.choke();
                self.closed_hh.trigger();
            }
            if step.open_hh {
                self.open_hh.trigger();
            }
            return Some(self.sequencer.current_step());
        }
        None
    }

    pub fn start(&mut self) {
        self.sequencer.start();
    }

    pub fn stop(&mut self) {
        self.sequencer.stop();
    }

    pub fn is_playing(&self) -> bool {
        self.sequencer.is_playing()
    }

    pub fn set_tempo(&mut self, bpm: f32) {
        self.sequencer.set_tempo(bpm);
    }

    // Volume setters
    pub fn set_kick_volume(&mut self, vol: f32) {
        self.kick_vol = vol.clamp(0.0, 1.0);
    }

    pub fn set_snare_volume(&mut self, vol: f32) {
        self.snare_vol = vol.clamp(0.0, 1.0);
    }

    pub fn set_hihat_volume(&mut self, vol: f32) {
        self.hh_vol = vol.clamp(0.0, 1.0);
    }

    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_vol = vol.clamp(0.0, 1.0);
    }

    // Sound parameter setters
    pub fn set_kick_decay(&mut self, decay: f32) {
        self.kick.set_decay(decay);
    }

    pub fn set_kick_pitch(&mut self, pitch: f32) {
        self.kick.set_pitch(pitch);
    }

    pub fn set_snare_tone(&mut self, tone: f32) {
        self.snare.set_tone(tone);
    }

    pub fn set_snare_snap(&mut self, snap: f32) {
        self.snare.set_snap(snap);
    }
}

impl Default for DrumMachine {
    fn default() -> Self {
        Self::new(44100.0)
    }
}

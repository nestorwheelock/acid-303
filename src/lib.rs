use wasm_bindgen::prelude::*;

mod oscillator;
mod filter;
mod envelope;
mod sequencer;
mod distortion;
mod presets;
mod drums;

pub use oscillator::{Oscillator, Waveform};
pub use filter::Filter;
pub use envelope::Envelope;
pub use sequencer::{Sequencer, Step};
pub use distortion::Distortion;
pub use presets::PRESETS;
pub use drums::{DrumMachine, DrumSequencer, DrumTrack};

const SAMPLE_RATE: f32 = 44100.0;

/// Main synthesizer engine - TB-303 style acid synth
#[wasm_bindgen]
pub struct Synth {
    oscillator: Oscillator,
    filter: Filter,
    envelope: Envelope,
    sequencer: Sequencer,
    distortion: Distortion,

    // Parameters
    cutoff: f32,
    resonance: f32,
    env_mod: f32,
    accent_amount: f32,

    // State
    current_note: f32,
    target_note: f32,
    slide_rate: f32,
    is_sliding: bool,
    gate: bool,
}

#[wasm_bindgen]
impl Synth {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            oscillator: Oscillator::new(SAMPLE_RATE),
            filter: Filter::new(SAMPLE_RATE),
            envelope: Envelope::new(SAMPLE_RATE),
            sequencer: Sequencer::new(),
            distortion: Distortion::new(),

            cutoff: 1000.0,
            resonance: 0.5,
            env_mod: 0.5,
            accent_amount: 0.7,

            current_note: 36.0, // C2
            target_note: 36.0,
            slide_rate: 0.001,
            is_sliding: false,
            gate: false,
        }
    }

    /// Process a block of audio samples
    #[wasm_bindgen]
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            // Handle note sliding (portamento)
            if self.is_sliding {
                if (self.current_note - self.target_note).abs() > 0.01 {
                    self.current_note += (self.target_note - self.current_note) * self.slide_rate;
                } else {
                    self.current_note = self.target_note;
                    self.is_sliding = false;
                }
            }

            // Convert MIDI note to frequency
            let freq = midi_to_freq(self.current_note);
            self.oscillator.set_frequency(freq);

            // Generate oscillator
            let osc_out = self.oscillator.process();

            // Get envelope value
            let env = self.envelope.process();

            // Calculate filter cutoff with envelope modulation
            let env_scaled = env * self.env_mod * 10000.0;
            let filter_freq = (self.cutoff + env_scaled).clamp(20.0, 20000.0);
            self.filter.set_cutoff(filter_freq);

            // Apply filter
            let filtered = self.filter.process(osc_out);

            // Apply VCA (envelope also controls amplitude)
            let vca_out = filtered * (0.3 + env * 0.7);

            // Apply distortion
            let distorted = self.distortion.process(vca_out);

            *sample = distorted * 0.5; // Master volume
        }
    }

    /// Trigger a note
    #[wasm_bindgen]
    pub fn note_on(&mut self, note: f32, accent: bool, slide: bool) {
        if slide && self.gate {
            // Slide to new note
            self.target_note = note;
            self.is_sliding = true;
        } else {
            // Immediate note change
            self.current_note = note;
            self.target_note = note;
            self.is_sliding = false;
        }

        self.gate = true;

        // Trigger envelope with accent
        let accent_mult = if accent { 1.0 + self.accent_amount } else { 1.0 };
        self.envelope.trigger(accent_mult);

        // Accent also boosts resonance temporarily
        if accent {
            self.filter.set_resonance((self.resonance + 0.2).min(1.0));
        } else {
            self.filter.set_resonance(self.resonance);
        }
    }

    /// Release a note
    #[wasm_bindgen]
    pub fn note_off(&mut self) {
        self.gate = false;
    }

    // Parameter setters

    #[wasm_bindgen]
    pub fn set_waveform(&mut self, saw: bool) {
        self.oscillator.set_waveform(if saw { Waveform::Saw } else { Waveform::Square });
    }

    #[wasm_bindgen]
    pub fn set_cutoff(&mut self, freq: f32) {
        self.cutoff = freq.clamp(20.0, 20000.0);
    }

    #[wasm_bindgen]
    pub fn set_resonance(&mut self, res: f32) {
        self.resonance = res.clamp(0.0, 1.0);
        self.filter.set_resonance(self.resonance);
    }

    #[wasm_bindgen]
    pub fn set_env_mod(&mut self, depth: f32) {
        self.env_mod = depth.clamp(0.0, 1.0);
    }

    #[wasm_bindgen]
    pub fn set_decay(&mut self, ms: f32) {
        self.envelope.set_decay(ms);
    }

    #[wasm_bindgen]
    pub fn set_accent(&mut self, amount: f32) {
        self.accent_amount = amount.clamp(0.0, 1.0);
    }

    #[wasm_bindgen]
    pub fn set_slide_time(&mut self, ms: f32) {
        let samples = (ms / 1000.0) * SAMPLE_RATE;
        self.slide_rate = 1.0 / samples.max(1.0);
    }

    #[wasm_bindgen]
    pub fn set_distortion(&mut self, amount: f32) {
        self.distortion.set_drive(amount);
    }

    // Sequencer controls

    #[wasm_bindgen]
    pub fn set_step(&mut self, index: usize, note: u8, accent: bool, slide: bool, active: bool) {
        self.sequencer.set_step(index, Step { note, accent, slide, active });
    }

    #[wasm_bindgen]
    pub fn set_tempo(&mut self, bpm: f32) {
        self.sequencer.set_tempo(bpm);
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) -> i32 {
        if let Some(step) = self.sequencer.tick() {
            if step.active {
                self.note_on(step.note as f32, step.accent, step.slide);
            }
            return self.sequencer.current_step() as i32;
        }
        -1
    }

    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.sequencer.start();
    }

    #[wasm_bindgen]
    pub fn stop(&mut self) {
        self.sequencer.stop();
        self.note_off();
    }

    #[wasm_bindgen]
    pub fn is_playing(&self) -> bool {
        self.sequencer.is_playing()
    }

    /// Load a preset pattern by index
    #[wasm_bindgen]
    pub fn load_preset(&mut self, index: usize) {
        if let Some(preset) = PRESETS.get(index) {
            for (i, step) in preset.steps.iter().enumerate() {
                self.sequencer.set_step(i, *step);
            }
            self.set_tempo(preset.tempo);
            self.set_cutoff(preset.cutoff);
            self.set_resonance(preset.resonance);
            self.set_env_mod(preset.env_mod);
            self.set_decay(preset.decay);
            self.set_waveform(preset.saw);
        }
    }

    /// Get number of available presets
    #[wasm_bindgen]
    pub fn preset_count() -> usize {
        PRESETS.len()
    }

    /// Get preset name
    #[wasm_bindgen]
    pub fn preset_name(index: usize) -> String {
        PRESETS.get(index)
            .map(|p| p.name.to_string())
            .unwrap_or_default()
    }
}

impl Default for Synth {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert MIDI note number to frequency in Hz
fn midi_to_freq(note: f32) -> f32 {
    440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
}

// ============== STUDIO (Synth + Drums Combined) ==============

/// Complete studio with 303 bass synth and 808/909 drum machine
#[wasm_bindgen]
pub struct Studio {
    synth: Synth,
    drums: DrumMachine,

    // Mixer levels
    synth_vol: f32,
    drum_vol: f32,
    master_vol: f32,

    // Sync state
    playing: bool,
    tempo: f32,

    // Step tracking for UI
    last_synth_step: i32,
    last_drum_step: i32,
    synth_step_changed: bool,
    drum_step_changed: bool,
}

#[wasm_bindgen]
impl Studio {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            synth: Synth::new(),
            drums: DrumMachine::new(SAMPLE_RATE),
            synth_vol: 0.7,
            drum_vol: 0.8,
            master_vol: 0.8,
            playing: false,
            tempo: 120.0,
            last_synth_step: -1,
            last_drum_step: -1,
            synth_step_changed: false,
            drum_step_changed: false,
        }
    }

    /// Process audio - combines synth and drums with integrated sequencer timing
    #[wasm_bindgen]
    pub fn process(&mut self, output: &mut [f32]) {
        // Reset step change flags at start of buffer
        self.synth_step_changed = false;
        self.drum_step_changed = false;

        for sample in output.iter_mut() {
            // Tick sequencers if playing
            if self.playing {
                // Synth sequencer
                if let Some(step) = self.synth.sequencer.tick() {
                    let new_step = self.synth.sequencer.current_step() as i32;
                    if new_step != self.last_synth_step {
                        self.last_synth_step = new_step;
                        self.synth_step_changed = true;
                    }
                    if step.active {
                        self.synth.note_on(step.note as f32, step.accent, step.slide);
                    }
                }

                // Drum sequencer
                if let Some(step) = self.drums.sequencer.tick() {
                    let new_step = self.drums.sequencer.current_step() as i32;
                    if new_step != self.last_drum_step {
                        self.last_drum_step = new_step;
                        self.drum_step_changed = true;
                    }
                    // Trigger drum sounds
                    if step.kick {
                        self.drums.kick.trigger();
                    }
                    if step.snare {
                        self.drums.snare.trigger();
                    }
                    if step.closed_hh {
                        self.drums.open_hh.choke();
                        self.drums.closed_hh.trigger();
                    }
                    if step.open_hh {
                        self.drums.open_hh.trigger();
                    }
                }
            }

            // Handle synth note sliding
            if self.synth.is_sliding {
                if (self.synth.current_note - self.synth.target_note).abs() > 0.01 {
                    self.synth.current_note += (self.synth.target_note - self.synth.current_note) * self.synth.slide_rate;
                } else {
                    self.synth.current_note = self.synth.target_note;
                    self.synth.is_sliding = false;
                }
            }

            let freq = midi_to_freq(self.synth.current_note);
            self.synth.oscillator.set_frequency(freq);

            let osc_out = self.synth.oscillator.process();
            let env = self.synth.envelope.process();

            let env_scaled = env * self.synth.env_mod * 10000.0;
            let filter_freq = (self.synth.cutoff + env_scaled).clamp(20.0, 20000.0);
            self.synth.filter.set_cutoff(filter_freq);

            let filtered = self.synth.filter.process(osc_out);
            let vca_out = filtered * (0.3 + env * 0.7);
            let synth_sample = self.synth.distortion.process(vca_out);

            // Process drums (sound generation)
            let drum_sample = self.drums.process();

            // Mix and output
            let mixed = (synth_sample * self.synth_vol) + (drum_sample * self.drum_vol);
            *sample = mixed * self.master_vol;
        }
    }

    /// Get current synth step (for UI), returns -1 if stopped
    #[wasm_bindgen]
    pub fn get_synth_step(&self) -> i32 {
        if self.playing { self.last_synth_step } else { -1 }
    }

    /// Get current drum step (for UI), returns -1 if stopped
    #[wasm_bindgen]
    pub fn get_drum_step(&self) -> i32 {
        if self.playing { self.last_drum_step } else { -1 }
    }

    /// Check if synth step changed during last process() call
    #[wasm_bindgen]
    pub fn synth_step_changed(&self) -> bool {
        self.synth_step_changed
    }

    /// Check if drum step changed during last process() call
    #[wasm_bindgen]
    pub fn drum_step_changed(&self) -> bool {
        self.drum_step_changed
    }

    // ===== Transport =====

    #[wasm_bindgen]
    pub fn start(&mut self) {
        self.playing = true;
        self.synth.sequencer.start();
        self.drums.start();
    }

    #[wasm_bindgen]
    pub fn stop(&mut self) {
        self.playing = false;
        self.synth.sequencer.stop();
        self.synth.note_off();
        self.drums.stop();
    }

    #[wasm_bindgen]
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    #[wasm_bindgen]
    pub fn set_tempo(&mut self, bpm: f32) {
        self.tempo = bpm.clamp(60.0, 300.0);
        self.synth.sequencer.set_tempo(self.tempo);
        self.drums.set_tempo(self.tempo);
    }

    // ===== Mixer =====

    #[wasm_bindgen]
    pub fn set_synth_volume(&mut self, vol: f32) {
        self.synth_vol = vol.clamp(0.0, 1.0);
    }

    #[wasm_bindgen]
    pub fn set_drum_volume(&mut self, vol: f32) {
        self.drum_vol = vol.clamp(0.0, 1.0);
    }

    #[wasm_bindgen]
    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_vol = vol.clamp(0.0, 1.0);
    }

    // ===== Synth controls (delegated) =====

    #[wasm_bindgen]
    pub fn synth_note_on(&mut self, note: f32, accent: bool, slide: bool) {
        self.synth.note_on(note, accent, slide);
    }

    #[wasm_bindgen]
    pub fn synth_note_off(&mut self) {
        self.synth.note_off();
    }

    #[wasm_bindgen]
    pub fn set_synth_waveform(&mut self, saw: bool) {
        self.synth.set_waveform(saw);
    }

    #[wasm_bindgen]
    pub fn set_synth_cutoff(&mut self, freq: f32) {
        self.synth.set_cutoff(freq);
    }

    #[wasm_bindgen]
    pub fn set_synth_resonance(&mut self, res: f32) {
        self.synth.set_resonance(res);
    }

    #[wasm_bindgen]
    pub fn set_synth_env_mod(&mut self, depth: f32) {
        self.synth.set_env_mod(depth);
    }

    #[wasm_bindgen]
    pub fn set_synth_decay(&mut self, ms: f32) {
        self.synth.set_decay(ms);
    }

    #[wasm_bindgen]
    pub fn set_synth_accent(&mut self, amount: f32) {
        self.synth.set_accent(amount);
    }

    #[wasm_bindgen]
    pub fn set_synth_slide_time(&mut self, ms: f32) {
        self.synth.set_slide_time(ms);
    }

    #[wasm_bindgen]
    pub fn set_synth_distortion(&mut self, amount: f32) {
        self.synth.set_distortion(amount);
    }

    #[wasm_bindgen]
    pub fn set_synth_step(&mut self, index: usize, note: u8, accent: bool, slide: bool, active: bool) {
        self.synth.set_step(index, note, accent, slide, active);
    }

    #[wasm_bindgen]
    pub fn load_synth_preset(&mut self, index: usize) {
        self.synth.load_preset(index);
    }

    // ===== Drum controls =====

    /// Set a drum step with all 4 tracks at once
    #[wasm_bindgen]
    pub fn set_drum_step(&mut self, index: usize, kick: bool, snare: bool, closed_hh: bool, open_hh: bool) {
        self.drums.sequencer.set_step(index, drums::DrumTrack::Kick, kick);
        self.drums.sequencer.set_step(index, drums::DrumTrack::Snare, snare);
        self.drums.sequencer.set_step(index, drums::DrumTrack::ClosedHH, closed_hh);
        self.drums.sequencer.set_step(index, drums::DrumTrack::OpenHH, open_hh);
    }

    /// Set a single drum track step
    #[wasm_bindgen]
    pub fn set_drum_track_step(&mut self, index: usize, track: u8, active: bool) {
        let track = match track {
            0 => drums::DrumTrack::Kick,
            1 => drums::DrumTrack::Snare,
            2 => drums::DrumTrack::ClosedHH,
            3 => drums::DrumTrack::OpenHH,
            _ => return,
        };
        self.drums.sequencer.set_step(index, track, active);
    }

    #[wasm_bindgen]
    pub fn toggle_drum_step(&mut self, index: usize, track: u8) {
        let track = match track {
            0 => drums::DrumTrack::Kick,
            1 => drums::DrumTrack::Snare,
            2 => drums::DrumTrack::ClosedHH,
            3 => drums::DrumTrack::OpenHH,
            _ => return,
        };
        self.drums.sequencer.toggle_step(index, track);
    }

    /// Get drum step data (all 4 tracks) for a specific step index
    #[wasm_bindgen]
    pub fn get_drum_step_data(&self, index: usize) -> Vec<u8> {
        if let Some(step) = self.drums.sequencer.get_step(index) {
            vec![
                step.kick as u8,
                step.snare as u8,
                step.closed_hh as u8,
                step.open_hh as u8,
            ]
        } else {
            vec![0, 0, 0, 0]
        }
    }

    #[wasm_bindgen]
    pub fn set_kick_volume(&mut self, vol: f32) {
        self.drums.set_kick_volume(vol);
    }

    #[wasm_bindgen]
    pub fn set_snare_volume(&mut self, vol: f32) {
        self.drums.set_snare_volume(vol);
    }

    #[wasm_bindgen]
    pub fn set_hihat_volume(&mut self, vol: f32) {
        self.drums.set_hihat_volume(vol);
    }

    #[wasm_bindgen]
    pub fn set_kick_decay(&mut self, decay: f32) {
        self.drums.set_kick_decay(decay);
    }

    #[wasm_bindgen]
    pub fn set_kick_pitch(&mut self, pitch: f32) {
        self.drums.set_kick_pitch(pitch);
    }

    #[wasm_bindgen]
    pub fn set_snare_tone(&mut self, tone: f32) {
        self.drums.set_snare_tone(tone);
    }

    #[wasm_bindgen]
    pub fn set_snare_snap(&mut self, snap: f32) {
        self.drums.set_snare_snap(snap);
    }

    #[wasm_bindgen]
    pub fn load_drum_pattern(&mut self, index: usize) {
        let pattern = match index {
            // Main patterns
            0 => &drums::BASIC_BEAT,
            1 => &drums::BREAKBEAT,
            2 => &drums::HOUSE_909,
            3 => &drums::MINIMAL,
            4 => &drums::ACID_DRIVE,
            // Arrangement patterns
            5 => &drums::INTRO_KICK,
            6 => &drums::INTRO_HATS,
            7 => &drums::BUILD_SNARE,
            8 => &drums::BUILD_ROLL,
            9 => &drums::BREAKDOWN,
            10 => &drums::BREAKDOWN_KICK,
            11 => &drums::FILL_SNARE,
            12 => &drums::FILL_STOMP,
            13 => &drums::FILL_OPEN_HAT,
            14 => &drums::DROP_FULL,
            15 => &drums::OFFBEAT_HOUSE,
            16 => &drums::SHUFFLE,
            _ => &drums::BASIC_BEAT,
        };
        self.drums.sequencer.load_pattern(pattern);
    }

    #[wasm_bindgen]
    pub fn drum_pattern_count() -> usize {
        17
    }

    #[wasm_bindgen]
    pub fn drum_pattern_name(index: usize) -> String {
        match index {
            // Main patterns
            0 => "Basic 4/4".to_string(),
            1 => "Breakbeat".to_string(),
            2 => "House 909".to_string(),
            3 => "Minimal Techno".to_string(),
            4 => "Acid Drive".to_string(),
            // Arrangement patterns
            5 => "-- INTRO --".to_string(),
            6 => "Intro + Hats".to_string(),
            7 => "Build (Snare)".to_string(),
            8 => "Build (Roll)".to_string(),
            9 => "Breakdown".to_string(),
            10 => "Breakdown + Kick".to_string(),
            11 => "Fill: Snare".to_string(),
            12 => "Fill: Stomp".to_string(),
            13 => "Fill: Open Hat".to_string(),
            14 => "DROP!".to_string(),
            15 => "Offbeat House".to_string(),
            16 => "Shuffle".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    // ===== Presets =====

    #[wasm_bindgen]
    pub fn synth_preset_count() -> usize {
        Synth::preset_count()
    }

    #[wasm_bindgen]
    pub fn synth_preset_name(index: usize) -> String {
        Synth::preset_name(index)
    }
}

impl Default for Studio {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_freq() {
        assert!((midi_to_freq(69.0) - 440.0).abs() < 0.01);
        assert!((midi_to_freq(57.0) - 220.0).abs() < 0.01);
        assert!((midi_to_freq(60.0) - 261.63).abs() < 0.1);
    }

    #[test]
    fn test_synth_creation() {
        let synth = Synth::new();
        assert!(!synth.is_playing());
    }

    #[test]
    fn test_synth_process() {
        let mut synth = Synth::new();
        let mut buffer = [0.0f32; 128];
        synth.note_on(48.0, false, false);
        synth.process(&mut buffer);
        assert!(buffer.iter().any(|&s| s.abs() > 0.001));
    }

    #[test]
    fn test_presets_exist() {
        assert!(Synth::preset_count() > 0);
        assert!(!Synth::preset_name(0).is_empty());
    }

    #[test]
    fn test_studio_creation() {
        let studio = Studio::new();
        assert!(!studio.is_playing());
    }

    #[test]
    fn test_studio_process() {
        let mut studio = Studio::new();
        let mut buffer = [0.0f32; 128];
        studio.synth_note_on(48.0, false, false);
        studio.process(&mut buffer);
        assert!(buffer.iter().any(|&s| s.abs() > 0.001));
    }

    #[test]
    fn test_drum_patterns_exist() {
        assert!(Studio::drum_pattern_count() > 0);
        assert!(!Studio::drum_pattern_name(0).is_empty());
    }
}

use wasm_bindgen::prelude::*;

mod oscillator;
mod filter;
mod envelope;
mod sequencer;
mod distortion;
mod presets;

pub use oscillator::{Oscillator, Waveform};
pub use filter::Filter;
pub use envelope::Envelope;
pub use sequencer::{Sequencer, Step};
pub use distortion::Distortion;
pub use presets::PRESETS;

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
}

# Acid-303: TB-303 Style Synthesizer in WebAssembly

## What
A browser-based acid synthesizer inspired by the Roland TB-303, implemented in Rust compiled to WebAssembly for near-native audio performance.

## Why
The TB-303's distinctive squelchy, resonant sound defines acid house music. Building one in WASM provides:
- Real-time audio synthesis at native speed in the browser
- No plugins or downloads required
- Educational platform for understanding subtractive synthesis
- Fun instrument for making acid basslines

## How
- **Rust** for audio DSP (oscillators, filters, envelopes)
- **WebAssembly** for browser execution
- **AudioWorklet** for real-time audio output
- **Vanilla JS/HTML/CSS** for UI

## Success Criteria
- [ ] Produces authentic 303-style acid sound
- [ ] Runs at 44.1kHz without audio glitches
- [ ] Sub-10ms latency from keypress to sound
- [ ] Works in Chrome, Firefox, Safari
- [ ] 16-step sequencer with accent/slide per step

## Risks
| Risk | Mitigation |
|------|------------|
| AudioWorklet browser support | Fallback to ScriptProcessorNode |
| Filter instability at high resonance | Clamp resonance, use stable filter topology |
| WASM size bloat | LTO optimization, no_std where possible |

---

## Scope

### IN SCOPE (v0.1.0)
- Single sawtooth/square oscillator
- 18dB/octave resonant lowpass filter
- Decay-only filter envelope
- Accent (boosts filter + volume)
- Slide (portamento between notes)
- 16-step sequencer
- Tempo control (60-200 BPM)
- Distortion/overdrive
- Keyboard input (computer keyboard)

### OUT OF SCOPE (Future)
- MIDI input
- Pattern save/load to file
- Multiple patterns/song mode
- Effects (delay, reverb)
- Mobile touch UI
- Audio export/recording

---

## Architecture

### Signal Flow
```
                    ┌─────────────────┐
                    │   SEQUENCER     │
                    │  (16 steps)     │
                    └────────┬────────┘
                             │ note, accent, slide
                             ▼
┌─────────┐   ┌─────────────────────────────┐   ┌──────────┐
│   VCO   │──▶│      18dB LP FILTER         │──▶│   VCA    │
│ Saw/Sqr │   │  cutoff + resonance         │   │  volume  │
└─────────┘   └──────────────▲──────────────┘   └────┬─────┘
                             │                       │
                    ┌────────┴────────┐              ▼
                    │  FILTER ENVELOPE │        ┌─────────┐
                    │  (decay only)    │        │  DIST   │
                    │  + env mod depth │        └────┬────┘
                    └─────────────────┘              │
                             ▲                       ▼
                             │                    OUTPUT
                          accent
```

### Module Structure
```
src/
├── lib.rs          # WASM exports, main synth struct
├── oscillator.rs   # Sawtooth and square wave generators
├── filter.rs       # 18dB resonant lowpass (3-pole)
├── envelope.rs     # Decay envelope generator
├── sequencer.rs    # 16-step pattern sequencer
├── distortion.rs   # Soft clipping overdrive
└── utils.rs        # Math helpers, MIDI note conversion

web/
├── index.html      # Main page
├── style.css       # 303-inspired UI styling
├── app.js          # Main application logic
├── worklet.js      # AudioWorkletProcessor wrapper
└── sequencer.js    # Sequencer UI controls
```

---

## User Stories

### S-001: Basic Sound Generation
**As a** user
**I want** to hear a bass sound when I press a key
**So that** I can play notes on the synth

**Acceptance Criteria:**
- [ ] Pressing keys A-L plays notes C3-C4
- [ ] Sound plays within 10ms of keypress
- [ ] Sound stops when key is released
- [ ] Can switch between saw and square waveforms

### S-002: Filter Control
**As a** user
**I want** to adjust cutoff and resonance
**So that** I can shape the tone from mellow to squelchy

**Acceptance Criteria:**
- [ ] Cutoff knob sweeps from 20Hz to 20kHz
- [ ] Resonance knob goes from 0 to self-oscillation
- [ ] Filter responds to envelope modulation
- [ ] Sound doesn't clip or distort unexpectedly

### S-003: Sequencer Playback
**As a** user
**I want** to program a 16-step pattern
**So that** I can create repeating acid basslines

**Acceptance Criteria:**
- [ ] 16 steps displayed in UI
- [ ] Click step to toggle note on/off
- [ ] Right-click to set accent
- [ ] Shift-click to set slide
- [ ] Pattern loops continuously
- [ ] Tempo adjustable 60-200 BPM

### S-004: Accent and Slide
**As a** user
**I want** accent and slide per step
**So that** I can create dynamic, expressive patterns

**Acceptance Criteria:**
- [ ] Accented notes are louder and brighter
- [ ] Slide smoothly glides to next note
- [ ] Accent indicator visible in sequencer UI
- [ ] Slide indicator visible in sequencer UI

---

## Technical Specifications

### Audio Parameters
| Parameter | Range | Default | Description |
|-----------|-------|---------|-------------|
| Sample Rate | 44100 Hz | 44100 | Audio sample rate |
| Buffer Size | 128 samples | 128 | AudioWorklet buffer |
| Waveform | saw/square | saw | Oscillator shape |
| Cutoff | 20-20000 Hz | 1000 | Filter cutoff frequency |
| Resonance | 0.0-1.0 | 0.5 | Filter resonance (Q) |
| Env Mod | 0.0-1.0 | 0.5 | Envelope to filter depth |
| Decay | 10-2000 ms | 200 | Envelope decay time |
| Accent | 0.0-1.0 | 0.7 | Accent intensity |
| Slide Time | 10-200 ms | 60 | Portamento time |
| Distortion | 0.0-1.0 | 0.3 | Overdrive amount |
| Tempo | 60-200 BPM | 120 | Sequencer tempo |

### Filter Design
The 303 uses a unique 18dB/octave (3-pole) lowpass filter. We'll implement this as a cascade of 3 one-pole filters with resonance feedback:

```
Input ──▶ [LP1] ──▶ [LP2] ──▶ [LP3] ──▶ Output
              ◀──────────────────────────┘
                    feedback (resonance)
```

### Envelope Design
Simple decay-only envelope:
- Trigger: Instant attack to peak
- Decay: Exponential falloff to zero
- Accent: Multiplies peak level and speeds decay

---

## Definition of Done

A feature is DONE when:
1. Rust code compiles to WASM without warnings
2. Unit tests pass for the module
3. Audio output is glitch-free at 44.1kHz
4. UI control works as specified
5. Works in Chrome and Firefox

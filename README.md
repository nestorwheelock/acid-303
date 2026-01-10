# Acid-303

A TB-303 style acid synthesizer built with Rust and WebAssembly.

![Acid-303 Screenshot](docs/screenshot.png)

## Features

- **Authentic 303 Sound**: Sawtooth/square oscillator, 18dB resonant filter, decay envelope
- **Classic Presets**: 10 preset patterns inspired by 90s acid house classics
- **Real-time Synthesis**: WebAssembly for near-native audio performance
- **16-Step Sequencer**: Program basslines with accent and slide per step
- **Full Control**: Cutoff, resonance, envelope mod, decay, accent, slide time, distortion
- **Keyboard Input**: Play notes with your computer keyboard (A-L = C2-C3)

## Preset Patterns

Includes patterns inspired by:
- **Acid Tracks** - Phuture (1987) - The original acid track
- **Higher State** - Josh Wink style (1995)
- **Acperience** - Hardfloor (1992)
- **Voodoo Ray** - A Guy Called Gerald style (1988)
- **Energy Flash** - Joey Beltram style (1990)
- And more classic rave patterns

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- A modern web browser (Chrome, Firefox, Safari)

### Build

```bash
# Clone the repo
git clone https://github.com/yourusername/acid-303
cd acid-303

# Build WASM
wasm-pack build --target web

# Serve the web directory
python3 -m http.server 8080 --directory .
# Or use any static file server
```

### Run

Open http://localhost:8080/web/ in your browser.

## Controls

### Synth Parameters

| Control | Description |
|---------|-------------|
| **VCO** | Sawtooth or Square waveform |
| **Cutoff** | Filter cutoff frequency (20-20000 Hz) |
| **Reso** | Filter resonance (0-100%) |
| **Env Mod** | Envelope modulation depth |
| **Decay** | Envelope decay time (10-2000 ms) |
| **Accent** | Accent intensity |
| **Slide** | Portamento time between notes |
| **Dist** | Distortion/overdrive amount |

### Sequencer

- **Click** step to toggle note on/off
- **Right-click** to toggle accent
- **Shift+click** to toggle slide
- **Double-click** to change note pitch

### Keyboard

```
 W E   T Y U
A S D F G H J K
C C# D D# E F F# G G# A A# B C
```

## Architecture

```
src/
├── lib.rs          # Main synth engine
├── oscillator.rs   # PolyBLEP oscillators
├── filter.rs       # 18dB resonant lowpass
├── envelope.rs     # Decay envelope
├── sequencer.rs    # 16-step sequencer
├── distortion.rs   # Soft clipping
└── presets.rs      # Classic patterns

web/
├── index.html      # Main page
├── style.css       # 303-style UI
├── app.js          # Application logic
└── worklet.js      # AudioWorklet processor
```

## Technical Details

### The 303 Sound

The TB-303's distinctive acid sound comes from:

1. **18dB/octave filter** - Unusual 3-pole design (most synths use 12 or 24dB)
2. **High resonance** - Creates the signature "squelch"
3. **Decay envelope** - Short attack, variable decay modulating the filter
4. **Accent** - Boosts filter and volume on specific steps
5. **Slide** - Glides smoothly between notes

### WebAssembly Audio

Audio is processed in the browser using:
- Rust compiled to WebAssembly for DSP
- ScriptProcessorNode for audio output (AudioWorklet in development)
- Real-time synthesis at 44.1kHz sample rate

## Development

```bash
# Run tests
cargo test

# Build for development
wasm-pack build --target web --dev

# Build for production
wasm-pack build --target web --release
```

## License

MIT

## Credits

Inspired by the Roland TB-303 and the pioneers of acid house music.

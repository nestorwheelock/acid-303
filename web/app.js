// Acid-303 Main Application
// Handles UI and WASM synth communication

import init, { Synth } from '../pkg/acid_303.js';

class Acid303App {
    constructor() {
        this.synth = null;
        this.audioContext = null;
        this.scriptProcessor = null;
        this.isPlaying = false;
        this.currentStep = -1;

        // Step data (local copy for UI)
        this.steps = Array(16).fill(null).map(() => ({
            note: 36, // C2
            accent: false,
            slide: false,
            active: false
        }));

        // Keyboard mapping: A-L keys to MIDI notes
        this.keyMap = {
            'a': 36, 'w': 37, 's': 38, 'e': 39, 'd': 40,
            'f': 41, 't': 42, 'g': 43, 'y': 44, 'h': 45,
            'u': 46, 'j': 47, 'k': 48
        };

        this.pressedKeys = new Set();
    }

    async init() {
        try {
            // Initialize WASM
            await init();
            this.synth = new Synth();

            console.log('Acid-303 WASM initialized');

            // Setup UI
            this.setupUI();
            this.setupKeyboard();
            this.loadPresets();

            // Load first preset
            this.loadPreset(0);

        } catch (error) {
            console.error('Failed to initialize:', error);
            alert('Failed to load synthesizer. Please refresh the page.');
        }
    }

    async startAudio() {
        if (this.audioContext) return;

        // Create audio context (must be triggered by user gesture)
        this.audioContext = new (window.AudioContext || window.webkitAudioContext)({
            sampleRate: 44100
        });

        // Use ScriptProcessorNode for compatibility
        // (AudioWorklet requires HTTPS in some browsers)
        const bufferSize = 2048;
        this.scriptProcessor = this.audioContext.createScriptProcessor(bufferSize, 0, 1);

        this.scriptProcessor.onaudioprocess = (event) => {
            const output = event.outputBuffer.getChannelData(0);

            if (this.synth) {
                // Process audio
                this.synth.process(output);

                // Tick sequencer (called per sample internally, but we check for step changes)
                if (this.isPlaying) {
                    for (let i = 0; i < output.length; i++) {
                        const step = this.synth.tick();
                        if (step >= 0 && step !== this.currentStep) {
                            this.currentStep = step;
                            this.updateStepDisplay();
                        }
                    }
                }
            } else {
                output.fill(0);
            }
        };

        this.scriptProcessor.connect(this.audioContext.destination);
        console.log('Audio started');
    }

    setupUI() {
        // Preset selector
        const presetSelect = document.getElementById('preset-select');
        presetSelect.addEventListener('change', () => {
            this.loadPreset(parseInt(presetSelect.value));
        });

        // Waveform buttons
        document.getElementById('wave-saw').addEventListener('click', () => {
            this.setWaveform(true);
        });
        document.getElementById('wave-square').addEventListener('click', () => {
            this.setWaveform(false);
        });

        // Knobs
        this.setupKnob('cutoff', (v) => this.synth?.set_cutoff(v));
        this.setupKnob('resonance', (v) => this.synth?.set_resonance(v / 100));
        this.setupKnob('env-mod', (v) => this.synth?.set_env_mod(v / 100));
        this.setupKnob('decay', (v) => this.synth?.set_decay(v));
        this.setupKnob('accent', (v) => this.synth?.set_accent(v / 100));
        this.setupKnob('slide-time', (v) => this.synth?.set_slide_time(v));
        this.setupKnob('distortion', (v) => this.synth?.set_distortion(v / 100));

        // Tempo
        const tempoInput = document.getElementById('tempo');
        tempoInput.addEventListener('change', () => {
            const bpm = parseFloat(tempoInput.value);
            this.synth?.set_tempo(bpm);
        });

        // Transport
        document.getElementById('play-btn').addEventListener('click', () => this.togglePlay());
        document.getElementById('stop-btn').addEventListener('click', () => this.stop());

        // Create step grid
        this.createStepGrid();
    }

    setupKnob(id, callback) {
        const input = document.getElementById(id);
        const valueDisplay = document.getElementById(`${id}-value`);

        input.addEventListener('input', () => {
            const value = parseFloat(input.value);
            valueDisplay.textContent = Math.round(value);
            callback(value);
        });

        // Also trigger on first interaction to ensure audio context
        input.addEventListener('mousedown', () => this.startAudio());
    }

    setWaveform(saw) {
        this.synth?.set_waveform(saw);
        document.getElementById('wave-saw').classList.toggle('active', saw);
        document.getElementById('wave-square').classList.toggle('active', !saw);
    }

    createStepGrid() {
        const grid = document.getElementById('step-grid');
        grid.innerHTML = '';

        for (let i = 0; i < 16; i++) {
            const step = document.createElement('div');
            step.className = 'step';
            step.dataset.index = i;

            step.innerHTML = `
                <span class="note-display">C2</span>
                <div class="indicators">
                    <span class="accent-dot"></span>
                    <span class="slide-dot"></span>
                </div>
                <span class="step-number">${i + 1}</span>
            `;

            // Click = toggle active
            step.addEventListener('click', (e) => {
                if (!e.shiftKey) {
                    this.toggleStepActive(i);
                } else {
                    this.toggleStepSlide(i);
                }
            });

            // Right-click = toggle accent
            step.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                this.toggleStepAccent(i);
            });

            // Double-click = change note
            step.addEventListener('dblclick', () => {
                this.cycleStepNote(i);
            });

            grid.appendChild(step);
        }
    }

    toggleStepActive(index) {
        this.startAudio();
        this.steps[index].active = !this.steps[index].active;
        this.updateStep(index);
    }

    toggleStepAccent(index) {
        this.startAudio();
        this.steps[index].accent = !this.steps[index].accent;
        this.updateStep(index);
    }

    toggleStepSlide(index) {
        this.startAudio();
        this.steps[index].slide = !this.steps[index].slide;
        this.updateStep(index);
    }

    cycleStepNote(index) {
        // Cycle through common bass notes
        const notes = [36, 38, 40, 41, 43, 45, 48]; // C2, D2, E2, F2, G2, A2, C3
        const currentIndex = notes.indexOf(this.steps[index].note);
        const nextIndex = (currentIndex + 1) % notes.length;
        this.steps[index].note = notes[nextIndex];
        this.updateStep(index);
    }

    updateStep(index) {
        const step = this.steps[index];
        this.synth?.set_step(index, step.note, step.accent, step.slide, step.active);
        this.updateStepUI(index);
    }

    updateStepUI(index) {
        const stepEl = document.querySelector(`.step[data-index="${index}"]`);
        if (!stepEl) return;

        const step = this.steps[index];

        stepEl.classList.toggle('active', step.active);
        stepEl.classList.toggle('has-accent', step.accent);
        stepEl.classList.toggle('has-slide', step.slide);

        const noteDisplay = stepEl.querySelector('.note-display');
        noteDisplay.textContent = this.midiToNoteName(step.note);
    }

    updateStepDisplay() {
        document.querySelectorAll('.step').forEach((el, i) => {
            el.classList.toggle('current', i === this.currentStep);
        });
    }

    midiToNoteName(midi) {
        const notes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
        const octave = Math.floor(midi / 12) - 1;
        const note = notes[midi % 12];
        return `${note}${octave}`;
    }

    loadPresets() {
        const select = document.getElementById('preset-select');
        const count = Synth.preset_count();

        for (let i = 0; i < count; i++) {
            const option = document.createElement('option');
            option.value = i;
            option.textContent = Synth.preset_name(i);
            select.appendChild(option);
        }
    }

    loadPreset(index) {
        this.synth?.load_preset(index);

        // Update UI from synth state
        // Since we can't read back from WASM easily, we'll re-sync manually
        // For now, just update the step display based on preset patterns
        this.syncStepsFromPreset(index);

        // Update all step UIs
        for (let i = 0; i < 16; i++) {
            this.updateStepUI(i);
        }

        // Update preset selector
        document.getElementById('preset-select').value = index;
    }

    syncStepsFromPreset(index) {
        // Hardcoded preset data to match Rust presets
        const presets = this.getPresetData();
        if (presets[index]) {
            this.steps = presets[index].steps.map(s => ({ ...s }));

            // Update UI controls
            const p = presets[index];
            document.getElementById('tempo').value = p.tempo;
            document.getElementById('cutoff').value = p.cutoff;
            document.getElementById('resonance').value = p.resonance * 100;
            document.getElementById('env-mod').value = p.envMod * 100;
            document.getElementById('decay').value = p.decay;

            // Update value displays
            document.getElementById('cutoff-value').textContent = Math.round(p.cutoff);
            document.getElementById('resonance-value').textContent = Math.round(p.resonance * 100);
            document.getElementById('env-mod-value').textContent = Math.round(p.envMod * 100);
            document.getElementById('decay-value').textContent = Math.round(p.decay);

            this.setWaveform(p.saw);
        }
    }

    getPresetData() {
        // Mirror of Rust preset data for UI sync
        const s = (note, accent, slide, active) => ({ note, accent, slide, active });
        const r = () => ({ note: 36, accent: false, slide: false, active: false });

        return [
            // Acid Tracks
            {
                tempo: 126, cutoff: 400, resonance: 0.75, envMod: 0.8, decay: 150, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(48,false,true,true), s(36,false,false,true),
                    s(38,true,false,true), r(), s(36,false,false,true), s(43,false,true,true),
                    s(36,true,false,true), r(), s(48,false,false,true), s(36,false,true,true),
                    s(41,true,false,true), s(36,false,false,true), r(), s(36,false,false,true)
                ]
            },
            // Higher State
            {
                tempo: 132, cutoff: 300, resonance: 0.85, envMod: 0.9, decay: 120, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(38,false,true,true), s(40,false,true,true),
                    s(41,true,false,true), s(41,false,false,true), s(43,false,true,true), s(45,false,true,true),
                    s(48,true,false,true), s(48,false,false,true), s(45,false,true,true), s(43,false,true,true),
                    s(41,true,false,true), s(38,false,true,true), s(36,false,true,true), r()
                ]
            },
            // Acperience
            {
                tempo: 138, cutoff: 350, resonance: 0.8, envMod: 0.7, decay: 100, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), s(48,false,true,true),
                    s(36,true,false,true), s(36,false,false,true), s(43,false,true,true), s(36,false,false,true),
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), s(41,false,true,true),
                    s(36,true,false,true), s(36,false,false,true), s(38,false,true,true), s(36,false,false,true)
                ]
            },
            // Voodoo Ray
            {
                tempo: 118, cutoff: 500, resonance: 0.65, envMod: 0.6, decay: 200, saw: true,
                steps: [
                    s(41,true,false,true), r(), s(43,false,false,true), s(45,false,true,true),
                    s(48,true,false,true), r(), s(45,false,true,true), s(43,false,false,true),
                    s(41,true,false,true), r(), s(38,false,false,true), s(36,false,true,true),
                    s(38,true,false,true), r(), s(41,false,true,true), r()
                ]
            },
            // Mentasm
            {
                tempo: 128, cutoff: 600, resonance: 0.7, envMod: 0.75, decay: 180, saw: false,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), r(),
                    s(36,true,false,true), s(43,false,true,true), s(48,false,true,true), r(),
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), r(),
                    s(36,true,false,true), s(41,false,true,true), s(36,false,true,true), r()
                ]
            },
            // Energy Flash
            {
                tempo: 130, cutoff: 450, resonance: 0.72, envMod: 0.65, decay: 140, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(36,true,false,true), s(36,false,false,true),
                    s(43,true,false,true), s(43,false,false,true), s(36,true,false,true), s(36,false,false,true),
                    s(36,true,false,true), s(36,false,false,true), s(41,true,false,true), s(41,false,false,true),
                    s(38,true,false,true), s(38,false,false,true), s(36,true,false,true), s(36,false,false,true)
                ]
            },
            // Squelch Classic
            {
                tempo: 125, cutoff: 250, resonance: 0.9, envMod: 0.95, decay: 100, saw: true,
                steps: [
                    s(36,true,false,true), s(48,false,true,true), s(36,false,true,true), s(48,false,true,true),
                    s(36,true,false,true), s(43,false,true,true), s(36,false,true,true), s(41,false,true,true),
                    s(36,true,false,true), s(48,false,true,true), s(36,false,true,true), s(45,false,true,true),
                    s(36,true,false,true), s(43,false,true,true), s(36,false,true,true), s(38,false,true,true)
                ]
            },
            // Minimal Techno
            {
                tempo: 135, cutoff: 800, resonance: 0.5, envMod: 0.4, decay: 250, saw: true,
                steps: [
                    s(36,true,false,true), r(), r(), s(36,false,false,true),
                    r(), s(36,true,false,true), r(), r(),
                    s(36,false,false,true), r(), s(43,true,true,true), r(),
                    s(36,false,true,true), r(), r(), r()
                ]
            },
            // Rave Anthem
            {
                tempo: 140, cutoff: 550, resonance: 0.68, envMod: 0.7, decay: 130, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(43,true,false,true), s(43,false,false,true),
                    s(48,true,false,true), s(48,false,false,true), s(43,true,false,true), s(43,false,false,true),
                    s(41,true,false,true), s(41,false,false,true), s(43,true,false,true), s(48,false,true,true),
                    s(53,true,false,true), s(48,false,true,true), s(43,false,true,true), s(36,false,true,true)
                ]
            },
            // Warehouse
            {
                tempo: 122, cutoff: 380, resonance: 0.78, envMod: 0.82, decay: 170, saw: true,
                steps: [
                    s(33,true,false,true), r(), s(33,false,false,true), s(36,false,true,true),
                    s(33,true,false,true), r(), s(40,false,true,true), s(33,false,true,true),
                    s(33,true,false,true), r(), s(33,false,false,true), s(45,false,true,true),
                    s(33,true,false,true), r(), s(33,false,false,true), r()
                ]
            }
        ];
    }

    async togglePlay() {
        await this.startAudio();

        if (this.isPlaying) {
            this.stop();
        } else {
            this.isPlaying = true;
            this.synth?.start();
            document.getElementById('play-btn').classList.add('playing');
            document.getElementById('play-btn').textContent = '⏸ PAUSE';
        }
    }

    stop() {
        this.isPlaying = false;
        this.currentStep = -1;
        this.synth?.stop();
        document.getElementById('play-btn').classList.remove('playing');
        document.getElementById('play-btn').textContent = '▶ PLAY';
        this.updateStepDisplay();
    }

    setupKeyboard() {
        // Create visual keyboard
        const keyboard = document.getElementById('keyboard');
        const notes = [
            { note: 'C', midi: 36, black: false, key: 'A' },
            { note: 'C#', midi: 37, black: true, key: 'W' },
            { note: 'D', midi: 38, black: false, key: 'S' },
            { note: 'D#', midi: 39, black: true, key: 'E' },
            { note: 'E', midi: 40, black: false, key: 'D' },
            { note: 'F', midi: 41, black: false, key: 'F' },
            { note: 'F#', midi: 42, black: true, key: 'T' },
            { note: 'G', midi: 43, black: false, key: 'G' },
            { note: 'G#', midi: 44, black: true, key: 'Y' },
            { note: 'A', midi: 45, black: false, key: 'H' },
            { note: 'A#', midi: 46, black: true, key: 'U' },
            { note: 'B', midi: 47, black: false, key: 'J' },
            { note: 'C', midi: 48, black: false, key: 'K' }
        ];

        notes.forEach(n => {
            const key = document.createElement('div');
            key.className = `key ${n.black ? 'black' : ''}`;
            key.dataset.midi = n.midi;
            key.innerHTML = `<span>${n.key}</span>`;

            key.addEventListener('mousedown', () => {
                this.playNote(n.midi);
                key.classList.add('pressed');
            });

            key.addEventListener('mouseup', () => {
                this.stopNote();
                key.classList.remove('pressed');
            });

            key.addEventListener('mouseleave', () => {
                key.classList.remove('pressed');
            });

            keyboard.appendChild(key);
        });

        // Computer keyboard input
        document.addEventListener('keydown', (e) => {
            if (e.repeat) return;
            const midi = this.keyMap[e.key.toLowerCase()];
            if (midi !== undefined && !this.pressedKeys.has(e.key)) {
                this.pressedKeys.add(e.key);
                this.playNote(midi);

                // Highlight key
                const keyEl = document.querySelector(`.key[data-midi="${midi}"]`);
                keyEl?.classList.add('pressed');
            }
        });

        document.addEventListener('keyup', (e) => {
            const midi = this.keyMap[e.key.toLowerCase()];
            if (midi !== undefined) {
                this.pressedKeys.delete(e.key);
                if (this.pressedKeys.size === 0) {
                    this.stopNote();
                }

                // Remove highlight
                const keyEl = document.querySelector(`.key[data-midi="${midi}"]`);
                keyEl?.classList.remove('pressed');
            }
        });
    }

    async playNote(midi) {
        await this.startAudio();
        this.synth?.note_on(midi, false, false);
    }

    stopNote() {
        this.synth?.note_off();
    }
}

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    const app = new Acid303App();
    app.init();
});

// Acid Studio Main Application
// Handles UI and WASM synth/drums communication

import init, { Studio, Synth } from '../pkg/acid_303.js';

class AcidStudioApp {
    constructor() {
        this.studio = null;
        this.audioContext = null;
        this.scriptProcessor = null;
        this.isPlaying = false;
        this.currentStep = -1;
        this.currentDrumStep = -1;

        // Bass step data (local copy for UI)
        this.steps = Array(16).fill(null).map(() => ({
            note: 36, // C2
            accent: false,
            slide: false,
            active: false
        }));

        // Drum step data (4 tracks x 16 steps)
        this.drumSteps = {
            kick: Array(16).fill(false),
            snare: Array(16).fill(false),
            closedHH: Array(16).fill(false),
            openHH: Array(16).fill(false)
        };

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
            this.studio = new Studio();

            console.log('Acid Studio WASM initialized');

            // Setup UI
            this.setupUI();
            this.setupKeyboard();
            this.loadPresets();
            this.loadDrumPresets();

            // Load first preset
            this.loadPreset(0);
            this.loadDrumPreset(0);

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
        const bufferSize = 2048;
        this.scriptProcessor = this.audioContext.createScriptProcessor(bufferSize, 0, 1);

        this.scriptProcessor.onaudioprocess = (event) => {
            const output = event.outputBuffer.getChannelData(0);

            if (this.studio) {
                // Process audio (includes sequencer timing)
                this.studio.process(output);

                // Check for step changes (for UI updates)
                if (this.isPlaying) {
                    if (this.studio.synth_step_changed()) {
                        this.currentStep = this.studio.get_synth_step();
                        this.updateBassStepDisplay();
                    }
                    if (this.studio.drum_step_changed()) {
                        this.currentDrumStep = this.studio.get_drum_step();
                        this.updateDrumStepDisplay();
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

        // Drum preset selector
        const drumPresetSelect = document.getElementById('drum-preset-select');
        drumPresetSelect.addEventListener('change', () => {
            this.loadDrumPreset(parseInt(drumPresetSelect.value));
        });

        // Waveform buttons
        document.getElementById('wave-saw').addEventListener('click', () => {
            this.setWaveform(true);
        });
        document.getElementById('wave-square').addEventListener('click', () => {
            this.setWaveform(false);
        });

        // Synth knobs
        this.setupKnob('cutoff', (v) => this.studio?.set_synth_cutoff(v));
        this.setupKnob('resonance', (v) => this.studio?.set_synth_resonance(v / 100));
        this.setupKnob('env-mod', (v) => this.studio?.set_synth_env_mod(v / 100));
        this.setupKnob('decay', (v) => this.studio?.set_synth_decay(v));
        this.setupKnob('accent', (v) => this.studio?.set_synth_accent(v / 100));
        this.setupKnob('slide-time', (v) => this.studio?.set_synth_slide_time(v));
        this.setupKnob('distortion', (v) => this.studio?.set_synth_distortion(v / 100));

        // Tempo
        const tempoInput = document.getElementById('tempo');
        tempoInput.addEventListener('change', () => {
            const bpm = parseFloat(tempoInput.value);
            this.studio?.set_tempo(bpm);
        });

        // Transport
        document.getElementById('play-btn').addEventListener('click', () => this.togglePlay());
        document.getElementById('stop-btn').addEventListener('click', () => this.stop());

        // Mixer volumes
        this.setupVolumeSlider('synth-vol', (v) => this.studio?.set_synth_volume(v / 100));
        this.setupVolumeSlider('drum-vol', (v) => this.studio?.set_drum_volume(v / 100));

        // Drum individual volumes
        this.setupVolumeSlider('kick-vol', (v) => this.studio?.set_kick_volume(v / 100));
        this.setupVolumeSlider('snare-vol', (v) => this.studio?.set_snare_volume(v / 100));
        this.setupVolumeSlider('hh-vol', (v) => this.studio?.set_hihat_volume(v / 100));

        // Create step grids
        this.createBassStepGrid();
        this.createDrumGrid();
    }

    setupKnob(id, callback) {
        const input = document.getElementById(id);
        const valueDisplay = document.getElementById(`${id}-value`);

        input.addEventListener('input', () => {
            const value = parseFloat(input.value);
            valueDisplay.textContent = Math.round(value);
            callback(value);
        });

        input.addEventListener('mousedown', () => this.startAudio());
    }

    setupVolumeSlider(id, callback) {
        const input = document.getElementById(id);
        input.addEventListener('input', () => {
            const value = parseFloat(input.value);
            callback(value);
        });
        input.addEventListener('mousedown', () => this.startAudio());
    }

    setWaveform(saw) {
        this.studio?.set_synth_waveform(saw);
        document.getElementById('wave-saw').classList.toggle('active', saw);
        document.getElementById('wave-square').classList.toggle('active', !saw);
    }

    createBassStepGrid() {
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
                    this.toggleBassStepActive(i);
                } else {
                    this.toggleBassStepSlide(i);
                }
            });

            // Right-click = toggle accent
            step.addEventListener('contextmenu', (e) => {
                e.preventDefault();
                this.toggleBassStepAccent(i);
            });

            // Double-click = change note
            step.addEventListener('dblclick', () => {
                this.cycleBassStepNote(i);
            });

            grid.appendChild(step);
        }
    }

    createDrumGrid() {
        const grid = document.getElementById('drum-grid');
        grid.innerHTML = '';

        const tracks = ['kick', 'snare', 'closed-hh', 'open-hh'];

        for (const track of tracks) {
            for (let i = 0; i < 16; i++) {
                const step = document.createElement('div');
                step.className = `drum-step ${track}`;
                step.dataset.track = track;
                step.dataset.step = i;

                step.addEventListener('click', () => {
                    this.toggleDrumStep(track, i);
                });

                grid.appendChild(step);
            }
        }
    }

    toggleDrumStep(track, index) {
        this.startAudio();

        const trackKey = track.replace('-', '');
        const trackMap = {
            'kick': 'kick',
            'snare': 'snare',
            'closedhh': 'closedHH',
            'openhh': 'openHH'
        };
        const key = trackMap[trackKey];

        this.drumSteps[key][index] = !this.drumSteps[key][index];

        // Update WASM
        this.studio?.set_drum_step(
            index,
            this.drumSteps.kick[index],
            this.drumSteps.snare[index],
            this.drumSteps.closedHH[index],
            this.drumSteps.openHH[index]
        );

        // Update UI
        this.updateDrumStepUI(track, index);
    }

    updateDrumStepUI(track, index) {
        const stepEl = document.querySelector(`.drum-step[data-track="${track}"][data-step="${index}"]`);
        if (!stepEl) return;

        const trackKey = track.replace('-', '');
        const trackMap = {
            'kick': 'kick',
            'snare': 'snare',
            'closedhh': 'closedHH',
            'openhh': 'openHH'
        };
        const key = trackMap[trackKey];

        stepEl.classList.toggle('active', this.drumSteps[key][index]);
    }

    updateDrumStepDisplay() {
        document.querySelectorAll('.drum-step').forEach((el) => {
            const step = parseInt(el.dataset.step);
            el.classList.toggle('current', step === this.currentDrumStep);
        });
    }

    toggleBassStepActive(index) {
        this.startAudio();
        this.steps[index].active = !this.steps[index].active;
        this.updateBassStep(index);
    }

    toggleBassStepAccent(index) {
        this.startAudio();
        this.steps[index].accent = !this.steps[index].accent;
        this.updateBassStep(index);
    }

    toggleBassStepSlide(index) {
        this.startAudio();
        this.steps[index].slide = !this.steps[index].slide;
        this.updateBassStep(index);
    }

    cycleBassStepNote(index) {
        const notes = [36, 38, 40, 41, 43, 45, 48];
        const currentIndex = notes.indexOf(this.steps[index].note);
        const nextIndex = (currentIndex + 1) % notes.length;
        this.steps[index].note = notes[nextIndex];
        this.updateBassStep(index);
    }

    updateBassStep(index) {
        const step = this.steps[index];
        this.studio?.set_synth_step(index, step.note, step.accent, step.slide, step.active);
        this.updateBassStepUI(index);
    }

    updateBassStepUI(index) {
        const stepEl = document.querySelector(`.step[data-index="${index}"]`);
        if (!stepEl) return;

        const step = this.steps[index];

        stepEl.classList.toggle('active', step.active);
        stepEl.classList.toggle('has-accent', step.accent);
        stepEl.classList.toggle('has-slide', step.slide);

        const noteDisplay = stepEl.querySelector('.note-display');
        noteDisplay.textContent = this.midiToNoteName(step.note);
    }

    updateBassStepDisplay() {
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

    loadDrumPresets() {
        const select = document.getElementById('drum-preset-select');
        const presets = this.getDrumPresetData();

        presets.forEach((preset, i) => {
            const option = document.createElement('option');
            option.value = i;
            option.textContent = preset.name;
            select.appendChild(option);
        });
    }

    loadPreset(index) {
        this.studio?.load_synth_preset(index);
        this.syncBassStepsFromPreset(index);

        for (let i = 0; i < 16; i++) {
            this.updateBassStepUI(i);
        }

        document.getElementById('preset-select').value = index;
    }

    loadDrumPreset(index) {
        const presets = this.getDrumPresetData();
        if (!presets[index]) return;

        const preset = presets[index];

        // Update local state
        this.drumSteps.kick = [...preset.kick];
        this.drumSteps.snare = [...preset.snare];
        this.drumSteps.closedHH = [...preset.closedHH];
        this.drumSteps.openHH = [...preset.openHH];

        // Update WASM
        for (let i = 0; i < 16; i++) {
            this.studio?.set_drum_step(
                i,
                this.drumSteps.kick[i],
                this.drumSteps.snare[i],
                this.drumSteps.closedHH[i],
                this.drumSteps.openHH[i]
            );
        }

        // Update UI
        const tracks = ['kick', 'snare', 'closed-hh', 'open-hh'];
        for (const track of tracks) {
            for (let i = 0; i < 16; i++) {
                this.updateDrumStepUI(track, i);
            }
        }

        document.getElementById('drum-preset-select').value = index;
    }

    getDrumPresetData() {
        // Drum patterns: true = hit, false = rest
        const k = (arr) => arr.map(v => v === 1);

        return [
            {
                name: "Basic 4/4",
                kick:     k([1,0,0,0, 1,0,0,0, 1,0,0,0, 1,0,0,0]),
                snare:    k([0,0,0,0, 1,0,0,0, 0,0,0,0, 1,0,0,0]),
                closedHH: k([1,0,1,0, 1,0,1,0, 1,0,1,0, 1,0,1,0]),
                openHH:   k([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,1])
            },
            {
                name: "Breakbeat",
                kick:     k([1,0,0,0, 0,0,1,0, 0,0,1,0, 0,0,0,0]),
                snare:    k([0,0,0,0, 1,0,0,0, 0,0,0,0, 1,0,0,1]),
                closedHH: k([1,0,1,0, 1,0,1,0, 1,0,1,0, 1,0,1,0]),
                openHH:   k([0,0,0,1, 0,0,0,1, 0,0,0,1, 0,0,0,0])
            },
            {
                name: "House 909",
                kick:     k([1,0,0,0, 1,0,0,0, 1,0,0,0, 1,0,0,0]),
                snare:    k([0,0,0,0, 1,0,0,0, 0,0,0,0, 1,0,0,0]),
                closedHH: k([1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1]),
                openHH:   k([0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0])
            },
            {
                name: "Minimal Techno",
                kick:     k([1,0,0,0, 1,0,0,0, 1,0,0,0, 1,0,0,0]),
                snare:    k([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0]),
                closedHH: k([0,0,1,0, 0,0,1,0, 0,0,1,0, 0,0,1,0]),
                openHH:   k([0,0,0,0, 0,0,0,0, 0,0,0,0, 0,0,0,0])
            },
            {
                name: "Acid Drive",
                kick:     k([1,0,0,1, 0,0,1,0, 1,0,0,1, 0,0,1,0]),
                snare:    k([0,0,0,0, 1,0,0,0, 0,0,0,0, 1,0,0,1]),
                closedHH: k([1,1,1,1, 1,1,1,1, 1,1,1,1, 1,1,1,1]),
                openHH:   k([0,0,0,0, 0,0,0,1, 0,0,0,0, 0,0,0,0])
            }
        ];
    }

    syncBassStepsFromPreset(index) {
        const presets = this.getBassPresetData();
        if (presets[index]) {
            this.steps = presets[index].steps.map(s => ({ ...s }));

            const p = presets[index];
            document.getElementById('tempo').value = p.tempo;
            document.getElementById('cutoff').value = p.cutoff;
            document.getElementById('resonance').value = p.resonance * 100;
            document.getElementById('env-mod').value = p.envMod * 100;
            document.getElementById('decay').value = p.decay;

            document.getElementById('cutoff-value').textContent = Math.round(p.cutoff);
            document.getElementById('resonance-value').textContent = Math.round(p.resonance * 100);
            document.getElementById('env-mod-value').textContent = Math.round(p.envMod * 100);
            document.getElementById('decay-value').textContent = Math.round(p.decay);

            this.setWaveform(p.saw);
        }
    }

    getBassPresetData() {
        const s = (note, accent, slide, active) => ({ note, accent, slide, active });
        const r = () => ({ note: 36, accent: false, slide: false, active: false });

        return [
            {
                tempo: 126, cutoff: 400, resonance: 0.75, envMod: 0.8, decay: 150, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(48,false,true,true), s(36,false,false,true),
                    s(38,true,false,true), r(), s(36,false,false,true), s(43,false,true,true),
                    s(36,true,false,true), r(), s(48,false,false,true), s(36,false,true,true),
                    s(41,true,false,true), s(36,false,false,true), r(), s(36,false,false,true)
                ]
            },
            {
                tempo: 132, cutoff: 300, resonance: 0.85, envMod: 0.9, decay: 120, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(38,false,true,true), s(40,false,true,true),
                    s(41,true,false,true), s(41,false,false,true), s(43,false,true,true), s(45,false,true,true),
                    s(48,true,false,true), s(48,false,false,true), s(45,false,true,true), s(43,false,true,true),
                    s(41,true,false,true), s(38,false,true,true), s(36,false,true,true), r()
                ]
            },
            {
                tempo: 138, cutoff: 350, resonance: 0.8, envMod: 0.7, decay: 100, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), s(48,false,true,true),
                    s(36,true,false,true), s(36,false,false,true), s(43,false,true,true), s(36,false,false,true),
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), s(41,false,true,true),
                    s(36,true,false,true), s(36,false,false,true), s(38,false,true,true), s(36,false,false,true)
                ]
            },
            {
                tempo: 118, cutoff: 500, resonance: 0.65, envMod: 0.6, decay: 200, saw: true,
                steps: [
                    s(41,true,false,true), r(), s(43,false,false,true), s(45,false,true,true),
                    s(48,true,false,true), r(), s(45,false,true,true), s(43,false,false,true),
                    s(41,true,false,true), r(), s(38,false,false,true), s(36,false,true,true),
                    s(38,true,false,true), r(), s(41,false,true,true), r()
                ]
            },
            {
                tempo: 128, cutoff: 600, resonance: 0.7, envMod: 0.75, decay: 180, saw: false,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), r(),
                    s(36,true,false,true), s(43,false,true,true), s(48,false,true,true), r(),
                    s(36,true,false,true), s(36,false,false,true), s(36,false,false,true), r(),
                    s(36,true,false,true), s(41,false,true,true), s(36,false,true,true), r()
                ]
            },
            {
                tempo: 130, cutoff: 450, resonance: 0.72, envMod: 0.65, decay: 140, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(36,true,false,true), s(36,false,false,true),
                    s(43,true,false,true), s(43,false,false,true), s(36,true,false,true), s(36,false,false,true),
                    s(36,true,false,true), s(36,false,false,true), s(41,true,false,true), s(41,false,false,true),
                    s(38,true,false,true), s(38,false,false,true), s(36,true,false,true), s(36,false,false,true)
                ]
            },
            {
                tempo: 125, cutoff: 250, resonance: 0.9, envMod: 0.95, decay: 100, saw: true,
                steps: [
                    s(36,true,false,true), s(48,false,true,true), s(36,false,true,true), s(48,false,true,true),
                    s(36,true,false,true), s(43,false,true,true), s(36,false,true,true), s(41,false,true,true),
                    s(36,true,false,true), s(48,false,true,true), s(36,false,true,true), s(45,false,true,true),
                    s(36,true,false,true), s(43,false,true,true), s(36,false,true,true), s(38,false,true,true)
                ]
            },
            {
                tempo: 135, cutoff: 800, resonance: 0.5, envMod: 0.4, decay: 250, saw: true,
                steps: [
                    s(36,true,false,true), r(), r(), s(36,false,false,true),
                    r(), s(36,true,false,true), r(), r(),
                    s(36,false,false,true), r(), s(43,true,true,true), r(),
                    s(36,false,true,true), r(), r(), r()
                ]
            },
            {
                tempo: 140, cutoff: 550, resonance: 0.68, envMod: 0.7, decay: 130, saw: true,
                steps: [
                    s(36,true,false,true), s(36,false,false,true), s(43,true,false,true), s(43,false,false,true),
                    s(48,true,false,true), s(48,false,false,true), s(43,true,false,true), s(43,false,false,true),
                    s(41,true,false,true), s(41,false,false,true), s(43,true,false,true), s(48,false,true,true),
                    s(53,true,false,true), s(48,false,true,true), s(43,false,true,true), s(36,false,true,true)
                ]
            },
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
            this.studio?.start();
            document.getElementById('play-btn').classList.add('playing');
            document.getElementById('play-btn').textContent = '|| PAUSE';
        }
    }

    stop() {
        this.isPlaying = false;
        this.currentStep = -1;
        this.currentDrumStep = -1;
        this.studio?.stop();
        document.getElementById('play-btn').classList.remove('playing');
        document.getElementById('play-btn').textContent = '▶ PLAY';
        this.updateBassStepDisplay();
        this.updateDrumStepDisplay();
    }

    setupKeyboard() {
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

        document.addEventListener('keydown', (e) => {
            if (e.repeat) return;
            const midi = this.keyMap[e.key.toLowerCase()];
            if (midi !== undefined && !this.pressedKeys.has(e.key)) {
                this.pressedKeys.add(e.key);
                this.playNote(midi);

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

                const keyEl = document.querySelector(`.key[data-midi="${midi}"]`);
                keyEl?.classList.remove('pressed');
            }
        });
    }

    async playNote(midi) {
        await this.startAudio();
        this.studio?.synth_note_on(midi, false, false);
    }

    stopNote() {
        this.studio?.synth_note_off();
    }
}

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    const app = new AcidStudioApp();
    app.init();
});

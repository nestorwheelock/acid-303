// AudioWorklet Processor for Acid-303
// Runs in a separate thread for real-time audio

class Acid303Processor extends AudioWorkletProcessor {
    constructor() {
        super();
        this.synth = null;
        this.wasmReady = false;

        // Handle messages from main thread
        this.port.onmessage = (event) => {
            const { type, data } = event.data;

            switch (type) {
                case 'init':
                    this.initWasm(data.wasmModule);
                    break;
                case 'noteOn':
                    if (this.synth) {
                        this.synth.note_on(data.note, data.accent, data.slide);
                    }
                    break;
                case 'noteOff':
                    if (this.synth) {
                        this.synth.note_off();
                    }
                    break;
                case 'setParam':
                    if (this.synth) {
                        this.setParameter(data.param, data.value);
                    }
                    break;
                case 'setStep':
                    if (this.synth) {
                        this.synth.set_step(data.index, data.note, data.accent, data.slide, data.active);
                    }
                    break;
                case 'loadPreset':
                    if (this.synth) {
                        this.synth.load_preset(data.index);
                        this.port.postMessage({ type: 'presetLoaded', data: { index: data.index } });
                    }
                    break;
                case 'start':
                    if (this.synth) {
                        this.synth.start();
                    }
                    break;
                case 'stop':
                    if (this.synth) {
                        this.synth.stop();
                    }
                    break;
            }
        };
    }

    async initWasm(wasmBytes) {
        try {
            // Instantiate WASM module
            const wasmModule = await WebAssembly.compile(wasmBytes);
            const instance = await WebAssembly.instantiate(wasmModule, {
                env: {
                    // Add any required imports here
                }
            });

            // Get the Synth constructor from wasm-bindgen glue
            // For now, we'll use a simplified approach
            this.wasmReady = true;
            this.port.postMessage({ type: 'ready' });
        } catch (error) {
            console.error('WASM init error:', error);
            this.port.postMessage({ type: 'error', data: { message: error.message } });
        }
    }

    setParameter(param, value) {
        if (!this.synth) return;

        switch (param) {
            case 'waveform':
                this.synth.set_waveform(value);
                break;
            case 'cutoff':
                this.synth.set_cutoff(value);
                break;
            case 'resonance':
                this.synth.set_resonance(value);
                break;
            case 'envMod':
                this.synth.set_env_mod(value);
                break;
            case 'decay':
                this.synth.set_decay(value);
                break;
            case 'accent':
                this.synth.set_accent(value);
                break;
            case 'slideTime':
                this.synth.set_slide_time(value);
                break;
            case 'distortion':
                this.synth.set_distortion(value);
                break;
            case 'tempo':
                this.synth.set_tempo(value);
                break;
        }
    }

    process(inputs, outputs, parameters) {
        const output = outputs[0];
        if (!output || output.length === 0) return true;

        const channel = output[0];

        if (this.synth && this.wasmReady) {
            // Process audio through WASM synth
            this.synth.process(channel);

            // Check for sequencer step changes
            const step = this.synth.tick();
            if (step >= 0) {
                this.port.postMessage({ type: 'step', data: { step } });
            }
        } else {
            // Output silence while not ready
            channel.fill(0);
        }

        return true;
    }
}

registerProcessor('acid-303-processor', Acid303Processor);

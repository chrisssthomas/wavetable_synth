# Polyphonic Analog Synthesizer - AI Coding Agent Instructions

## Project Architecture

This is a real-time MIDI polyphonic analog synthesizer in Rust that processes MIDI input and generates audio output through the Rodio audio library with proper ADSR envelopes.

### Core Components & Data Flow

**MIDI Input → Voice Manager → Audio Stream**: 
- `main.rs` handles MIDI input callbacks that trigger note events in `PolySynth`
- `VoiceManager` handles polyphonic voice allocation with up to 16 concurrent voices
- Each `Voice` contains an `AnalogOscillator` + `ADSR` envelope
- `PolySynth` implements Rodio's `Source` trait for continuous audio generation

**Key Dependencies**:
- `rodio` (0.14.0): Audio output and `Source` trait implementation  
- `midir` (0.6): Real-time MIDI input handling
- `midly` (0.5.3): Present but unused (likely for future MIDI file support)

### Current Architecture (Working & Stable)

**Voice Management System**:
- **`Voice`**: Individual note with oscillator, ADSR envelope, velocity sensitivity
- **`VoiceManager`**: Handles polyphony, voice stealing, note on/off events  
- **`PolySynth`**: Rodio Source implementation that mixes all active voices

**Analog Waveform Generation**:
- Mathematical waveforms: Sine, Sawtooth, Square, Triangle
- Phase-based oscillation with proper frequency calculation
- Currently defaults to Sawtooth wave

### Key Implementation Patterns

**Frequency Calculation**:
```rust
frequency = 440 * 2^((midi_key - 69) / 12)  // Equal temperament, A4=440Hz
```

**Voice Lifecycle**:
```rust
// Note On: Creates voice with ADSR start
voice_manager.note_on(key, velocity);
// Note Off: Triggers ADSR release phase  
voice_manager.note_off(key);
```

**Shared State Architecture**:
```rust
// PolySynth uses Arc<Mutex<VoiceManager>> for thread-safe voice access
let synth = PolySynth::new(44100, 16);  // 16 max voices
sink.append(synth.clone());  // Clone shares same voice manager
```

### MIDI Event Handling

Properly handles:
- **Note On** (`0x90` with velocity > 0): Triggers note start with ADSR attack
- **Note Off** (`0x80` or `0x90` with velocity 0): Triggers ADSR release
- **Velocity sensitivity**: MIDI velocity (0-127) mapped to voice amplitude
- **Voice stealing**: Oldest voices removed when hitting 16-voice limit

### Development Commands

- **Run**: `cargo run` 
- **Build release**: `cargo build --release`
- **Check**: `cargo check` for quick compilation validation

## Extension Points

**Adding Waveforms**: Extend `WaveType` enum in `oscillator.rs` and implement in `get_sample()`

**Synthesis Parameters**: Modify `VoiceSettings` struct for filter, LFO, or other controls

**Effects Processing**: Add processing in `VoiceManager::get_sample()` before voice mixing

The architecture is now stable, polyphonic, and properly integrated with ADSR envelopes.
- **Format**: `cargo fmt`

## Integration Priority

When working on audio features:
1. Connect `WavetableOscillator` to replace `SineWave` in main.rs
2. Integrate ADSR envelope control with note on/off events
3. Consider polyphony - current design is monophonic (single frequency queue)

The existing components are well-designed but disconnected - focus on bridging the gap between MIDI input handling and the wavetable synthesis engine.
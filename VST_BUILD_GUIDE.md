# VST Plugin Build Instructions

This synthesizer can be built both as a standalone application and as a VST plugin for use in DAWs like Ableton Live, Logic Pro, FL Studio, etc.

## Building the Standalone Application

```bash
# Default build (includes GUI)
cargo build --release --features standalone

# Run standalone version
cargo run --features standalone
```

## Building the VST Plugin

### macOS (VST .component)

```bash
# Build the VST plugin library
cargo build --release --features vst --lib

# Create VST bundle directory
mkdir -p "WavetableSynth.component/Contents/MacOS"

# Copy the library
cp target/release/libwavetable_synth_vst.dylib "WavetableSynth.component/Contents/MacOS/WavetableSynth"

# Create Info.plist
cat > "WavetableSynth.component/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>WavetableSynth</string>
    <key>CFBundleIdentifier</key>
    <string>com.rustsynth.wavetablesynth</string>
    <key>CFBundleName</key>
    <string>Wavetable Synth</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
</dict>
</plist>
EOF

# Install to system Components folder (requires admin privileges)
sudo cp -r "WavetableSynth.component" "/Library/Audio/Plug-Ins/Components/"

# Or install to user Components folder
cp -r "WavetableSynth.component" "~/Library/Audio/Plug-Ins/Components/"
```

### Windows (VST .dll)

```bash
# Build the VST plugin library
cargo build --release --features vst --lib --target x86_64-pc-windows-msvc

# Rename the library to .dll
cp target/x86_64-pc-windows-msvc/release/wavetable_synth_vst.dll WavetableSynth.dll

# Copy to VST plugin directory (typical locations):
# C:\Program Files\Steinberg\VstPlugins\
# C:\Program Files\Common Files\VST2\
# Or your DAW-specific plugin folder
```

### Linux (VST .so)

```bash
# Build the VST plugin library
cargo build --release --features vst --lib

# Rename the library
cp target/release/libwavetable_synth_vst.so WavetableSynth.so

# Copy to VST plugin directory:
# ~/.vst/
# /usr/lib/vst/
# Or your DAW-specific plugin folder
```

## VST Plugin Features

The VST plugin includes:

### MIDI Support
- Full polyphonic MIDI input (16 voices)
- Note On/Off with velocity sensitivity
- MIDI CC automation:
  - CC 1 (Mod Wheel): Filter Frequency
  - CC 74: Filter Cutoff
  - CC 71: Filter Resonance

### Parameters (DAW Automatable)
1. **Attack** (0.001s - 2.0s): Envelope attack time
2. **Decay** (0.001s - 2.0s): Envelope decay time
3. **Sustain** (0.0 - 1.0): Envelope sustain level
4. **Release** (0.001s - 3.0s): Envelope release time
5. **Filter Freq** (200Hz - 8000Hz): Low-pass filter cutoff frequency
6. **Filter Res** (0.1 - 10.0): Low-pass filter resonance
7. **Waveform** (0-1): Oscillator waveform (Sine, Saw, Square, Triangle)
8. **Volume** (0.0 - 1.0): Master output volume

### Audio Quality Improvements
- **PolyBLEP anti-aliasing**: Reduces harsh high-frequency artifacts
- **Parameter smoothing**: Eliminates clicks from rapid parameter changes
- **DC blocking**: Removes unwanted DC offset
- **Proper voice management**: 16-voice polyphony with voice stealing

## Testing in DAWs

### Ableton Live
1. Copy the plugin to your VST folder
2. Rescan plugins in Live's preferences
3. Create a new MIDI track
4. Add "Wavetable Synth" as an instrument
5. Play MIDI notes or use automation to control parameters

### Logic Pro (macOS only)
1. Install the .component bundle to `/Library/Audio/Plug-Ins/Components/`
2. Open Logic Pro and rescan Audio Units
3. Create a Software Instrument track
4. Choose "Wavetable Synth" from the AU Instruments menu

### FL Studio
1. Copy the .dll to your VST plugins folder
2. Refresh plugin database in FL Studio
3. Load as a generator instrument

## Troubleshooting

### Audio Clicks/Pops
If you still experience audio artifacts:
1. Increase your DAW's buffer size (512 or 1024 samples)
2. Use a dedicated audio interface with ASIO drivers (Windows) or aggregate device (macOS)
3. Close unnecessary applications to reduce CPU load

### macOS Audio Interface
The built-in audio on MacBook Pro M4 should work fine, but for best results:
- Use an external audio interface (Focusrite Scarlett, PreSonus AudioBox, etc.)
- Set buffer size to 256-512 samples in your DAW
- Use 44.1kHz or 48kHz sample rate

### Plugin Not Loading
- Ensure you have the correct architecture (x64)
- Check plugin paths are correct for your OS
- Verify your DAW supports VST 2.x plugins
- Try rescanning plugins in your DAW

## Performance Notes

- **CPU Usage**: Optimized for real-time performance
- **Memory**: Low memory footprint (~1MB)
- **Latency**: Sub-millisecond processing latency
- **Voice Count**: 16 concurrent voices maximum
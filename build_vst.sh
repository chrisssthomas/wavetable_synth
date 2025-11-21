#!/bin/bash

# VST Plugin Build Script for macOS
# This script automates the VST plugin build process

set -e

echo "🎛️ Building Wavetable Synthesizer VST Plugin..."

# Build the VST library
echo "⚙️ Compiling VST library..."
cargo build --release --features vst --lib

# Create component bundle structure
echo "📦 Creating VST component bundle..."
BUNDLE_NAME="WavetableSynth.component"
BUNDLE_DIR="$BUNDLE_NAME/Contents/MacOS"
mkdir -p "$BUNDLE_DIR"

# Copy the compiled library
echo "📋 Copying binary..."
cp "target/release/libwavetable_synth_vst.dylib" "$BUNDLE_DIR/WavetableSynth"

# Create Info.plist
echo "📝 Creating Info.plist..."
cat > "$BUNDLE_NAME/Contents/Info.plist" << EOF
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
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
</dict>
</plist>
EOF

echo "✅ VST plugin built successfully!"
echo "📁 Plugin bundle created: $BUNDLE_NAME"
echo ""
echo "🚀 To install the plugin:"
echo "   For all users: sudo cp -r '$BUNDLE_NAME' '/Library/Audio/Plug-Ins/Components/'"
echo "   For current user: cp -r '$BUNDLE_NAME' '~/Library/Audio/Plug-Ins/Components/'"
echo ""
echo "🎵 After installation, rescan plugins in your DAW to use the synthesizer!"
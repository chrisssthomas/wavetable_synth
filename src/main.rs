mod envelope;
mod oscillator;
mod voice;
mod synth;
mod filter;
mod distortion;
mod reverb;
#[cfg(feature = "standalone")]
mod gui_pro;

#[cfg(test)]
mod tests;

use synth::PolySynth;
use voice::VoiceSettings;
use oscillator::WaveType;
use distortion::DistortionType;
use reverb::ReverbType;
#[cfg(feature = "standalone")]
use gui_pro::SynthGui;

use midir::MidiInput;
use rodio::{OutputStream, Sink};
use std::error::Error;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🎹 Starting Analog Synthesizer with GUI...");
    
    // Initialize audio output
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    // Create the synthesizer
    const SAMPLE_RATE: u32 = 44100;
    const MAX_VOICES: usize = 16;
    let synth = PolySynth::new(SAMPLE_RATE, MAX_VOICES);
    let shared_synth = Arc::new(synth.clone());  // Clone for sharing
    
    // Set up default synthesizer settings
    let default_settings = VoiceSettings {
        osc1_wave_type: WaveType::Sawtooth,
        osc2_wave_type: WaveType::Square,
        osc1_shape: 0.6,        // Aggressive sawtooth shaping for growl
        osc2_shape: 0.4,        // Moderate square shaping
        osc_mix: 0.5,
        osc2_detune: 0.0,
        attack_time: 0.001,
        decay_time: 0.1,
        sustain_level: 0.8,
        release_time: 0.1,
        filter_freq: 2000.0,
        filter_resonance: 0.4,
        distortion_type: DistortionType::Overdrive,
        distortion_drive: 0.35,
        distortion_tone: 0.65,
        distortion_level: 0.8,
        reverb_type: ReverbType::Room,
        reverb_size: 0.4,
        reverb_decay: 0.5,
        reverb_mix: 0.25,       // Subtle reverb by default
        master_volume: 1.0,
    };
    shared_synth.update_settings(default_settings);
    
    // Start audio playback
    sink.append(synth);
    
    // Setup MIDI in a separate thread
    let synth_midi = Arc::clone(&shared_synth);
    thread::spawn(move || {
        if let Err(e) = setup_midi(synth_midi) {
            println!("MIDI setup error: {}", e);
        }
    });
    
    // Launch GUI
    #[cfg(feature = "standalone")]
    {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_title("Analog Synthesizer"),
            ..Default::default()
        };
        
        eframe::run_native(
            "Analog Synthesizer",
            options,
            Box::new(|_cc| Ok(Box::new(SynthGui::new(shared_synth)))),
        )?;
    }
    
    #[cfg(not(feature = "standalone"))]
    {
        println!("Running in headless mode. Use Ctrl+C to exit.");
        // Keep the application running in headless mode
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    
    Ok(())
}

fn setup_midi(synth: Arc<PolySynth>) -> Result<(), Box<dyn Error>> {
    let midi_in = MidiInput::new("Analog Synth MIDI Input")?;
    let in_ports = midi_in.ports();
    
    let in_port = match in_ports.first() {
        Some(port) => {
            println!("🎼 Connected to MIDI: {}", midi_in.port_name(port)?);
            port
        },
        None => {
            println!("⚠️  No MIDI input ports available");
            println!("💡 You can still use the GUI to adjust settings");
            // Keep the thread alive even without MIDI
            loop {
                thread::sleep(std::time::Duration::from_secs(1));
            }
        },
    };
    
    // Setup MIDI input callback
    let _conn_in = midi_in.connect(in_port, "synth-input", move |_timestamp, message, _| {
        match message {
            [0x90, key, velocity] if *velocity > 0 => {
                // Note On
                synth.note_on(*key, *velocity);
                println!("🎵 Note ON:  Key={} Vel={}", key, velocity);
            },
            [0x80, key, _] | [0x90, key, 0] => {
                // Note Off (explicit note off or note on with velocity 0)
                synth.note_off(*key);
                println!("� Note OFF: Key={}", key);
            },
            _ => {
                // Ignore other MIDI messages for now
            }
        }
    }, ())?;
    
    println!("🎹 MIDI is ready! Play some notes on your keyboard");
    
    // Keep MIDI thread alive
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
pub mod envelope;
pub mod wavetable_oscillator;
pub mod oscillator;
pub mod voice;
pub mod synth;
pub mod filter;
pub mod distortion;
pub mod reverb;
pub mod gui_pro;

#[cfg(feature = "vst")]
pub mod plugin;

#[cfg(test)]
mod tests;

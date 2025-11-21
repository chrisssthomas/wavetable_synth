#[cfg(feature = "vst")]
mod envelope;
#[cfg(feature = "vst")]
mod oscillator;
#[cfg(feature = "vst")]
mod voice;
#[cfg(feature = "vst")]
mod synth;
#[cfg(feature = "vst")]
mod filter;
#[cfg(feature = "vst")]
mod plugin;

#[cfg(feature = "vst")]
pub use plugin::*;
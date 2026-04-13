//! # transcribe-rs
//!
//! A Rust library providing text-to-speech synthesis using the Kokoro engine.
//!
//! ## Features
//!
//! - **Kokoro TTS**: High-quality text-to-speech with multiple voices and languages
//! - **Flexible Model Loading**: Load models with custom parameters
//! - **Multiple Voices**: Support for 9 languages with various voice styles
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! transcribe-rs = { version = "0.2", features = ["kokoro"] }
//! ```
//!
//! ```ignore
//! use std::path::PathBuf;
//! use transcribe_rs::{engines::kokoro::KokoroEngine, SynthesisEngine};
//!
//! let mut engine = KokoroEngine::new();
//! engine.load_model(&PathBuf::from("models/kokoro-v1.0"))?;
//!
//! let result = engine.synthesize("Hello, world!", None)?;
//! result.write_wav(&PathBuf::from("output.wav"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod engines;

use std::path::Path;

/// The result of a synthesis (text-to-speech) operation.
///
/// Contains raw f32 audio samples and the sample rate of the output audio.
#[derive(Debug)]
pub struct SynthesisResult {
    /// Raw audio samples as f32 values
    pub samples: Vec<f32>,
    /// Sample rate of the audio (24000 for Kokoro)
    pub sample_rate: u32,
}

impl SynthesisResult {
    /// Write the audio to a 32-bit float WAV file.
    pub fn write_wav(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;
        for &sample in &self.samples {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;
        Ok(())
    }

    /// Write the audio to an OGG file with a target bitrate in bits per second.
    pub fn write_ogg(
        &self,
        path: &Path,
        bitrate: std::num::NonZeroU32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = std::fs::File::create(path)?;
        let mut encoder = vorbis_rs::VorbisEncoderBuilder::new(
            std::num::NonZeroU32::new(self.sample_rate).expect("Sample rate must not be zero"),
            std::num::NonZeroU8::new(1).unwrap(),
            &mut file,
        )?
        .bitrate_management_strategy(vorbis_rs::VorbisBitrateManagementStrategy::Vbr {
            target_bitrate: bitrate,
        })
        .build()?;

        encoder.encode_audio_block([self.samples.as_slice()])?;
        encoder.finish()?;
        Ok(())
    }

    /// Save the audio to a file, inferring the format from the extension.
    ///
    /// Supports `.wav` and `.ogg` formats. For `.ogg`, uses a default target bitrate of `64_000` (64 kbps)
    /// unless an `ogg_bitrate` parameter is specified.
    pub fn save_to_file(
        &self,
        path: &Path,
        ogg_bitrate: Option<std::num::NonZeroU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ext) = path.extension() {
            if ext.eq_ignore_ascii_case("ogg") {
                return self.write_ogg(
                    path,
                    ogg_bitrate.unwrap_or(std::num::NonZeroU32::new(64_000).unwrap()),
                );
            }
        }
        self.write_wav(path)
    }

    /// Duration of the audio in seconds.
    pub fn duration_secs(&self) -> f64 {
        self.samples.len() as f64 / self.sample_rate as f64
    }
}

/// Common interface for text-to-speech synthesis engines.
///
/// This trait defines the standard operations that all synthesis engines must support.
/// Each engine may have different parameter types for model loading and inference configuration.
pub trait SynthesisEngine {
    /// Parameters for configuring inference behavior (voice, speed, etc.)
    type SynthesisParams;
    /// Parameters for configuring model loading (threads, etc.)
    type ModelParams: Default;

    /// Load a model from the specified path using default parameters.
    fn load_model(&mut self, model_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.load_model_with_params(model_path, Self::ModelParams::default())
    }

    /// Load a model from the specified path with custom parameters.
    fn load_model_with_params(
        &mut self,
        model_path: &Path,
        params: Self::ModelParams,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Unload the currently loaded model and free associated resources.
    fn unload_model(&mut self);

    /// Synthesize speech from the given text.
    fn synthesize(
        &mut self,
        text: &str,
        params: Option<Self::SynthesisParams>,
    ) -> Result<SynthesisResult, Box<dyn std::error::Error>>;

    /// Synthesize speech from the given text and write to a WAV or OGG file based on the extension.
    ///
    /// Default implementation calls `synthesize()` then `SynthesisResult::save_to_file()`.
    /// For OGG files, a default target bitrate of 64 kbps is used. Call `save_to_file` manually on the result to
    /// override the target bitrate.
    fn synthesize_to_file(
        &mut self,
        text: &str,
        file_path: &Path,
        params: Option<Self::SynthesisParams>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.synthesize(text, params)?.save_to_file(file_path, None)
    }
}

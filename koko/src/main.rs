use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use clap::Parser;
use tts_rs::{
    engines::kokoro::{KokoroEngine, KokoroInferenceParams, KokoroModelParams},
    SynthesisEngine,
};

#[derive(Parser, Debug)]
#[command(
    name = "koko",
    version,
    about = "Kokoro TTS — convert text to speech and save as OGG",
    long_about = "koko synthesizes speech from text (inline or file) using the Kokoro ONNX model.\n\
                  Output is always an OGG Vorbis file."
)]
struct Args {
    /// Text to synthesize (mutually exclusive with --file)
    #[arg(short, long, group = "input", value_name = "TEXT")]
    text: Option<String>,

    /// Path to a text file to synthesize (mutually exclusive with --text)
    #[arg(short, long, group = "input", value_name = "FILE")]
    file: Option<PathBuf>,

    /// Output .ogg file path
    #[arg(short, long, default_value = "output.ogg", value_name = "OUT")]
    output: PathBuf,

    /// Voice identifier (e.g. af_heart, bf_emma, jf_alpha)
    #[arg(short, long, default_value = "af_heart", value_name = "VOICE")]
    voice: String,

    /// Speech speed multiplier (0.5 = slow, 1.0 = normal, 2.0 = fast)
    #[arg(short, long, default_value = "1.0", value_name = "SPEED")]
    speed: f32,

    /// OGG target bitrate in kbps (minimum ~32 for 24 kHz audio)
    #[arg(short, long, default_value = "32", value_name = "KBPS")]
    bitrate: u32,

    /// Path to the directory containing model files
    #[arg(short, long, default_value = "../models", value_name = "DIR")]
    models: PathBuf,

    /// Number of CPU threads to use for inference (default: all available)
    #[arg(long, value_name = "N")]
    threads: Option<usize>,

    /// Path to a voice file (.npy, .bin, or .npz/.bin zip archive).
    ///
    /// A raw .bin or .npy file contains a single voice whose name is derived from
    /// the file stem (e.g. `af_heart.bin` → voice `"af_heart"`). A zip archive
    /// contains multiple named voices. When this flag is set and --voice is not
    /// provided, the voice name is inferred from the file stem automatically.
    #[arg(long, value_name = "PATH")]
    voice_file: Option<PathBuf>,

    /// List all available voices and exit
    #[arg(long)]
    list_voices: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let wall_start = Instant::now();

    // Validate output path
    match args.output.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("ogg") => {}
        _ => bail!(
            "Output file must have a .ogg extension, got: {}",
            args.output.display()
        ),
    }

    // Validate bitrate
    let bitrate_bps = args.bitrate.checked_mul(1_000).context("Bitrate overflow")?;
    let bitrate = NonZeroU32::new(bitrate_bps)
        .context("Bitrate must be greater than zero")?;

    // Validate speed
    if !(0.1..=4.0).contains(&args.speed) {
        bail!("Speed must be between 0.1 and 4.0, got: {}", args.speed);
    }

    eprintln!("Loading model from {} ...", args.models.display());
    let load_start = Instant::now();

    let mut engine = KokoroEngine::new();
    let model_params = KokoroModelParams {
        num_threads: args.threads,
        voice_file: args.voice_file.clone(),
        ..KokoroModelParams::default()
    };
    engine
        .load_model_with_params(&args.models, model_params)
        .map_err(|e| anyhow::anyhow!("Failed to load model from {}: {e}", args.models.display()))?;

    eprintln!("Model loaded in {:.2?}", load_start.elapsed());

    // When a single voice file was given and --voice was not explicitly set,
    // use the name derived from the file stem (e.g. `voice.bin` → `"voice"`).
    let voice = if args.voice_file.is_some() && args.voice == "af_heart" {
        engine
            .first_voice_name()
            .unwrap_or(&args.voice)
            .to_string()
    } else {
        args.voice.clone()
    };

    if args.list_voices {
        let voices = engine.list_voices();
        println!("Available voices ({}):", voices.len());
        for v in &voices {
            println!("  {v}");
        }
        return Ok(());
    }

    // Require --text or --file (clap `group` enforces mutual exclusion, but not "at least one")
    let text = match (args.text, args.file) {
        (Some(t), _) => t,
        (_, Some(f)) => std::fs::read_to_string(&f)
            .with_context(|| format!("Failed to read input file: {}", f.display()))?,
        (None, None) => bail!("Provide input via --text \"...\" or --file <path>"),
    };

    let text = text.trim();
    if text.is_empty() {
        bail!("Input text is empty");
    }

    eprintln!(
        "Synthesizing {} chars with voice '{}' at speed {:.1}x ...",
        text.len(),
        voice,
        args.speed,
    );

    let params = KokoroInferenceParams {
        voice: voice.clone(),
        speed: args.speed,
        ..Default::default()
    };

    let synth_start = Instant::now();
    let result = engine
        .synthesize(text, Some(params))
        .map_err(|e| anyhow::anyhow!("Synthesis failed for voice '{}': {e}", voice))?;
    let synth_elapsed = synth_start.elapsed();

    let audio_secs = result.duration_secs();
    let realtime = audio_secs / synth_elapsed.as_secs_f64();
    eprintln!(
        "Synthesized {:.2}s of audio in {:.2?} ({:.1}x real-time)",
        audio_secs, synth_elapsed, realtime
    );

    result
        .write_ogg(&args.output, bitrate)
        .map_err(|e| anyhow::anyhow!("Failed to write OGG to {}: {e}", args.output.display()))?;

    eprintln!("Saved to {}", args.output.display());
    eprintln!("Total time: {:.2?}", wall_start.elapsed());

    engine.unload_model();
    Ok(())
}

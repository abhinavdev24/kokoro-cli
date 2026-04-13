use std::path::PathBuf;
use std::time::Instant;

use tts_rs::{
    engines::kokoro::{KokoroEngine, KokoroInferenceParams, KokoroModelParams},
    SynthesisEngine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut engine = KokoroEngine::new();
    let model_path = PathBuf::from("models");

    let load_start = Instant::now();
    engine.load_model_with_params(&model_path, KokoroModelParams::default())?;
    println!("Model loaded in {:.2?}", load_start.elapsed());

    println!("Available voices: {:?}", engine.list_voices());

    let text = std::fs::read_to_string("example_text/llm_transformers.txt")?;
    let text = text.as_str();

    let params = KokoroInferenceParams {
        voice: "af_heart".to_string(),
        speed: 1.0,
        ..Default::default()
    };

    let synth_start = Instant::now();
    let result = engine.synthesize(text, Some(params))?;
    let synth_dur = synth_start.elapsed();

    let audio_duration = result.samples.len() as f64 / result.sample_rate as f64;
    let speedup = audio_duration / synth_dur.as_secs_f64();
    println!(
        "Synthesized {:.2}s audio in {:.2?} ({:.1}x real-time)",
        audio_duration, synth_dur, speedup
    );

    result.save_to_file(&PathBuf::from("output.wav"), None)?;
    println!("Saved to output.wav");

    // Save as OGG with a specific bitrate (e.g., 32 kbps; note that 16 kbps is not supported for 24kHz audio)
    result.save_to_file(
        &PathBuf::from("output.ogg"),
        Some(std::num::NonZeroU32::new(32_000).unwrap()),
    )?;
    println!("Saved to output.ogg (Bitrate: 32 kbps)");

    engine.unload_model();
    Ok(())
}

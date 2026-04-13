# Kokoro CLI

Local text-to-speech synthesis using the [Kokoro 82M](https://huggingface.co/hexgrad/Kokoro-82M) ONNX model. Converts text to OGG Vorbis audio entirely on-device, no API required.

## Requirements

- Debian/Ubuntu Linux (x86_64)
- ~500 MB disk space for models

---

## 1. System dependencies

```bash
sudo apt-get update
sudo apt-get install -y \
    espeak-ng \
    pkg-config \
    libssl-dev \
    build-essential \
    curl
```

| Package                     | Purpose                                                    |
| --------------------------- | ---------------------------------------------------------- |
| `espeak-ng`                 | Phonemizer — called at runtime to convert text to phonemes |
| `build-essential`           | C linker required by Rust                                  |
| `pkg-config` / `libssl-dev` | Required by the ONNX Runtime crate at build time           |
| `curl`                      | Used by the model download script                          |

## 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

## 3. Download models

From the root of this repository:

```bash
chmod +x download_models.sh
./download_models.sh
```

This downloads two files into `./models/`:

| File                                  | Description                                 |
| ------------------------------------- | ------------------------------------------- |
| `kokoro-cliro-quant-convinteger.onnx` | Quantized neural TTS model (~88 MB)         |
| `af_heart.pt`                         | American English female voice style vectors |

## 4. Build

```bash
cd kokoro-cli
cargo build --release
```

The binary is built at `kokoro-cli/target/release/kokoro-cli`. The ONNX Runtime shared library (`libonnxruntime.so`) is automatically downloaded for your architecture during the build and copied next to the binary.

## 5. Run

From the `kokoro-cli/` directory, the `--models` flag defaults to `../models` (the `models/` folder at the root of this repo).

```bash
# Synthesize from a string
./target/release/kokoro-cli --text "Hello, world!" --output hello.ogg

# Synthesize from a file
./target/release/kokoro-cli --file ../examples/llm_transformers.txt --output speech.ogg

# Change voice and speed
./target/release/kokoro-cli --text "Good morning." --voice bf_emma --speed 1.2 --output morning.ogg

# List all available voices
./target/release/kokoro-cli --list-voices
```

## 6. Install system-wide (optional)

```bash
sudo cp kokoro-cli/target/release/kokoro-cli /usr/local/bin/kokoro-cli
sudo cp kokoro-cli/target/release/libonnxruntime.so* /usr/local/lib/
sudo ldconfig
```

Then use `kokoro-cli` from anywhere, pointing it at the models:

```bash
kokoro-cli --text "Hello." --models /path/to/kokoro-cliro-local/models --output out.ogg
```

Or set a permanent alias in `~/.bashrc`:

```bash
echo 'alias kokoro-cli="kokoro-cli --models /path/to/kokoro-cliro-local/models"' >> ~/.bashrc
source ~/.bashrc
```

---

## CLI reference

```
kokoro-cli [OPTIONS]

Options:
  -t, --text <TEXT>      Text to synthesize (mutually exclusive with --file)
  -f, --file <FILE>      Text file to synthesize (mutually exclusive with --text)
  -o, --output <OUT>     Output .ogg file [default: output.ogg]
  -v, --voice <VOICE>    Voice identifier [default: af_heart]
  -s, --speed <SPEED>    Speed multiplier (0.1–4.0) [default: 1.0]
  -b, --bitrate <KBPS>   OGG target bitrate in kbps [default: 32]
  -m, --models <DIR>     Path to models directory [default: ../models]
      --threads <N>      CPU threads for inference [default: all available]
      --voice-file <PATH> Load a voice directly from a .pt/.bin/.npy file
      --list-voices      List all available voices and exit
  -h, --help             Print help
  -V, --version          Print version
```

## Available voice prefixes

| Prefix        | Language         | Gender        |
| ------------- | ---------------- | ------------- |
| `af_`         | American English | Female        |
| `am_`         | American English | Male          |
| `bf_`         | British English  | Female        |
| `bm_`         | British English  | Male          |
| `ef_`         | Spanish          | Female        |
| `ff_`         | French           | Female        |
| `hf_` / `hm_` | Hindi            | Female / Male |
| `if_` / `im_` | Italian          | Female / Male |
| `jf_` / `jm_` | Japanese         | Female / Male |
| `pf_` / `pm_` | Portuguese (BR)  | Female / Male |
| `zf_` / `zm_` | Mandarin Chinese | Female / Male |

Run `kokoro-cli --list-voices` for the full list of specific voice names.

---

## Project structure

```
kokoro-cliro-local/
├── download_models.sh          # Downloads model files into ./models
├── models/                     # Model files (not in source control)
│   ├── kokoro-cliro-quant-convinteger.onnx
│   └── af_heart.pt
├── kokoro-cli/                       # CLI binary crate
│   ├── src/main.rs
│   └── Cargo.toml
├── tts-rs/                     # Kokoro Rust library (path dependency)
└── examples/                   # Sample input/output
```

## Credits

TTS inference powered by [tts-rs](https://github.com/rishiskhare/tts-rs/) by [@rishiskhare](https://github.com/rishiskhare).

## License

MIT

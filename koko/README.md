# koko

A fast CLI tool for text-to-speech synthesis using the [Kokoro](https://huggingface.co/hexgrad/Kokoro-82M) ONNX model. Output is always an OGG Vorbis file.

## Features

- Synthesize from an inline string or a text file
- 50 voices across 9 languages (American/British English, Spanish, French, Hindi, Italian, Japanese, Portuguese, Mandarin)
- Configurable voice, speed, and OGG bitrate
- ~1.5–2× real-time synthesis speed on CPU

## Install on Linux (Debian/Ubuntu)

### 1. System dependencies

```bash
sudo apt-get update
sudo apt-get install -y \
    espeak-ng \
    pkg-config \
    libssl-dev \
    build-essential \
    curl
```

| Package                     | Why                                          |
|-----------------------------|----------------------------------------------|
| `espeak-ng`                 | Phonemizer — called at runtime as subprocess |
| `build-essential`           | C linker needed by Rust                      |
| `pkg-config` / `libssl-dev` | Required by the ort crate during build       |

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

### 3. Get the source

Copy the `kokoro-local/` directory to the server (run on your local machine):

```bash
rsync -av --exclude='*/target' /path/to/kokoro-local/ user@server:/opt/kokoro-local/
```

### 4. Build the release binary

```bash
cd /opt/kokoro-local/koko
cargo build --release
```

The binary is at `target/release/koko`. The ONNX Runtime library
(`libonnxruntime.so`) is automatically downloaded for your architecture
during the build and copied next to the binary.

### 5. Install as a system command

```bash
# Install the binary
sudo cp target/release/koko /usr/local/bin/koko

# Install the ONNX Runtime library so the dynamic linker can find it
sudo cp target/release/libonnxruntime.so* /usr/local/lib/
sudo ldconfig
```

### 6. Place the models

koko looks for models in `./models` by default, or wherever `--models` points.
A convenient system-wide location:

```bash
sudo mkdir -p /opt/koko/models
sudo cp /opt/kokoro-local/models/kokoro-quant-convinteger.onnx /opt/koko/models/
sudo cp /opt/kokoro-local/models/voices-v1.0.bin               /opt/koko/models/
```

### 7. Verify

```bash
koko --list-voices --models /opt/koko/models
```

You should see 50 voices printed. koko is ready.

---

## Usage

```text
koko [OPTIONS]

Options:
  -t, --text <TEXT>      Text to synthesize
  -f, --file <FILE>      Text file to synthesize
  -o, --output <OUT>     Output .ogg file [default: output.ogg]
  -v, --voice <VOICE>    Voice identifier [default: af_heart]
  -s, --speed <SPEED>    Speed multiplier 0.5–2.0 [default: 1.0]
  -b, --bitrate <KBPS>   OGG target bitrate in kbps [default: 32]
  -m, --models <DIR>     Path to models directory [default: models]
      --threads <N>      CPU threads for inference [default: all]
      --list-voices      List all available voices and exit
  -h, --help             Print help
  -V, --version          Print version
```

### Synthesize from a string

```bash
koko --text "Hello, world!" --output hello.ogg --models /opt/koko/models
```

### Synthesize from a file

```bash
koko --file input.txt --output speech.ogg --models /opt/koko/models
```

### Change voice and speed

```bash
koko --text "Good morning." --voice bf_emma --speed 1.2 --output morning.ogg --models /opt/koko/models
```

### List all voices

```bash
koko --list-voices --models /opt/koko/models
```

### Skip typing `--models` every time

Export the path in your shell profile (`~/.bashrc` or `~/.profile`):

```bash
echo 'alias koko="koko --models /opt/koko/models"' >> ~/.bashrc
source ~/.bashrc

# Now you can just run:
koko --file input.txt --output out.ogg
```

---

## Available voices

| Prefix | Language          | Gender |
|--------|-------------------|--------|
| `af_`  | American English  | Female |
| `am_`  | American English  | Male   |
| `bf_`  | British English   | Female |
| `bm_`  | British English   | Male   |
| `ef_`  | Spanish           | Female |
| `em_`  | Spanish           | Male   |
| `ff_`  | French            | Female |
| `hf_`  | Hindi             | Female |
| `hm_`  | Hindi             | Male   |
| `if_`  | Italian           | Female |
| `im_`  | Italian           | Male   |
| `jf_`  | Japanese          | Female |
| `jm_`  | Japanese          | Male   |
| `pf_`  | Portuguese (BR)   | Female |
| `pm_`  | Portuguese (BR)   | Male   |
| `zf_`  | Mandarin Chinese  | Female |
| `zm_`  | Mandarin Chinese  | Male   |

Run `koko --list-voices` for the full list of specific voice names.

---

## Project structure

```text
kokoro-local/
├── models/                          # Model files (not in source control)
│   ├── kokoro-quant-convinteger.onnx
│   └── voices-v1.0.bin
├── koko/                            # This CLI tool
│   ├── src/main.rs
│   ├── Cargo.toml
│   └── README.md
└── tts-rs/                          # Kokoro Rust library (path dependency)
```

## License

MIT

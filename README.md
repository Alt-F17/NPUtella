# NPUtella, Local NPU Whisper STT

Local native Windows NPU-accelerated dictation for Snapdragon X Plus.
Hold `Ctrl`, then `Win`, speak, release, and the recognized text is pasted into the focused app.
Multilingual Whisper-Base runs locally through ONNX Runtime's QNN Execution Provider. No cloud inference.

## Setup

## Download Release

GitHub releases ship one independent MSI installer:

```text
NPUtella-<version>-SnapdragonXPlus-NPU.msi
```

This release is for Snapdragon X Plus Windows ARM64 NPU devices. It bundles:

- native `nputella.exe`
- Start Menu shortcut
- English, French, and bilingual EN/FR Whisper model artifacts
- Whisper tokenizer files
- ONNX Runtime QNN DLLs

More languages are planned for later releases.

## Developer Setup

### 1. Python version for setup

Use the AMD64/x64 Python build from python.org when running `setup.py`.
Do not use the ARM64 build for Qualcomm AI Hub export tooling.

Check yours:

```powershell
python -c "import platform; print(platform.machine())"
```

It should print `AMD64`.

### 2. Get a Qualcomm AI Hub token

- Sign up at https://app.aihub.qualcomm.com
- Go to Settings -> API Token
- Run: `qai-hub configure --api_token <your_token>`

The Hub compiles the model for the target Qualcomm device and downloads the ONNX artifacts.

### 3. Install and export

```powershell
cd C:\path\to\NPUtella
python setup.py
```

`setup.py` installs `requirements.txt`, exports multilingual Whisper-Base for developer builds, and normalizes the runtime model layout to:

```text
models/
  whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core/
    encoder.onnx
    decoder.onnx
```

### 4. Build the native app

```powershell
cargo build --release --target aarch64-pc-windows-msvc
```

The native executable is:

```text
target\aarch64-pc-windows-msvc\release\nputella.exe
```

The packaged release includes the matching ARM64 ONNX Runtime/QNN DLLs, model files, and tokenizer assets. No Python or Qualcomm AI Hub token is needed by end users.

### 5. Run

```powershell
target\aarch64-pc-windows-msvc\release\nputella.exe
```

For the background launcher, `start_nputella.vbs` runs the packaged native executable beside it.

## Usage

| Action | Result |
|---|---|
| Hold `Ctrl`, then `Win` | Starts recording and shows red audio bars |
| Release `Ctrl+Win` | Transcribes locally and pastes with `Ctrl+V` |
| Click idle overlay | Toggles recording |
| Hover and click `dict` | Opens the dictionary manager |
| Hover and click `bi`/`fr`/`en` | Cycles transcription language |
| Tap under 300 ms | Ignored to prevent accidental triggers |

The native app listens for `Ctrl+Win` directly. Press `Ctrl` first, then `Win`; pressing `Win` first is passed through to Windows.

## Local Adaptation

NPUtella applies local post-processing before paste:

- language prompt selection supports `bi`, `en`, and `fr`
- dictionary replacements and snippet expansion
- spoken punctuation, new lines, and `press enter`
- IDE/chat-aware filename and symbol tagging
- basic code and math phrase formatting
- custom dictionary entries persist to `%APPDATA%\NPUtella\dictionary.toml`

The shipped dictionary contains one phonetic starter entry:

- `NPUtella` with aliases for common spoken forms of the brand name

To add dictionary entries, hover the idle pill and click `dict`, or use the NPUtella system tray menu.
The dictionary manager edits custom written forms, comma-separated aliases, phonetic matching, priority, and language.

Optional config is loaded from `nputella.toml` in the project root first, then `%APPDATA%\NPUtella\config.toml`.

```toml
language = "auto" # auto, bi, en, fr; use fr or en to force one language while testing
local_adaptation_enabled = false
smart_formatting = true
code_formatting = true
math_formatting = true
file_tagging = true
symbol_tagging = true
keep_transcript_on_clipboard = true
local_llm_enabled = false
local_llm_model = "phi-3.5-mini"
local_llm_endpoint = "http://127.0.0.1:5273/v1/chat/completions"

[[dictionary]]
from = "n p u tella"
written = "NPUtella"

[[snippet]]
trigger = "code fence"
expansion = "```\n\n```"
```

## How It Works

```text
Ctrl+Win down
  -> native cpal recorder captures mono audio
Ctrl+Win up
  -> audio -> native Whisper log-mel features [1, 80, 3000]
  -> encoder.onnx through ONNX Runtime/QNN -> cross-attention KV cache
  -> decoder.onnx through ONNX Runtime/QNN -> greedy token decode
  -> local Whisper BPE tokenizer
  -> native clipboard write + SendInput Ctrl+V
```

The QNN Execution Provider attempts to route encoder and decoder to Qualcomm HTP in FP16 burst mode.
CPU fallback is used if QNN EP fails to initialize.

## Files

```text
Cargo.toml       native Rust app manifest
src/             native Windows app source
installer/       WiX MSI installer definition
scripts/         release packaging and publish scripts
main.py          legacy Python entry point
transcriber.py   legacy Python audio/inference implementation
overlay.py       legacy Tkinter overlay
setup.py         dependency install and Qualcomm AI Hub export
requirements.txt Python dependencies
f17.ahk          optional legacy F17 remap for development
keytest.py       diagnostic key listener
start_nputella.vbs hidden launcher for packaged native exe
whisper-base-local/ local tokenizer and feature extractor config
models/          compiled ONNX model artifacts
```

## Packaging

Run this from a Snapdragon X Plus Windows ARM64 development machine with Rust plus either WiX Toolset v4 on `PATH` or the WiX v4 .NET tool available. Release packaging also requires `NPUTELLA_RELEASE_ASSET_ROOT` to point at a prebuilt ARM64 asset bundle:

```powershell
.\scripts\build-installer.ps1
```

The `Cargo.toml` version must be an MSI-safe numeric `major.minor.patch` value, for example `2.0.0`.

The script builds `target\aarch64-pc-windows-msvc\release\nputella.exe`, stages the app, model, tokenizer, and ARM64 QNN runtime files, runs `nputella.exe --self-check`, writes release metadata/checksums, then writes the standalone MSI to `dist\`.

To publish with the GitHub CLI:

```powershell
.\scripts\publish-release.ps1
```

## Troubleshooting

`models missing`: run `python setup.py` and confirm `encoder.onnx` and `decoder.onnx` exist in the nested runtime model directory.

`QNN EP not active`: confirm the release payload contains `runtime\onnxruntime\capi\QnnHtp.dll` and the other bundled ONNX Runtime/QNN DLLs.

Paste does not work in some apps: some apps block synthetic input. The transcript is still copied to the clipboard.

Export fails with device not found: edit `DEVICE` in `setup.py` or run `qai-hub list-devices` to find a valid target.

## Rewrite Notes

The native Rust rewrite implements these subsystems:

- Global Ctrl+Win press/release listener with internal F17 compatibility for development.
- 16 kHz mono microphone capture with block-level RMS levels.
- Whisper-compatible log-mel preprocessing.
- ONNX Runtime sessions with QNN provider options and CPU fallback.
- Static-cache greedy decoder loop using the exported model's exact tensor names and shapes.
- Whisper BPE/tokenizer support from `whisper-base-local`.
- Borderless always-on-top floating overlay with recording, transcribing, done, loading, and error states.
- Native clipboard write plus synthetic `Ctrl+V`.

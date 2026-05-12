# NPUtella, Local NPU WisprFlow

Local native Windows NPU-accelerated dictation for Snapdragon X Plus.
Hold `Right Alt`, speak, release, and the recognized text is pasted into the focused app.
Whisper-Base-En runs locally through ONNX Runtime's QNN Execution Provider. No cloud inference.

## Setup

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
cd C:\Users\felix\Nextcloud\NPUtella
python setup.py
```

`setup.py` installs `requirements.txt`, exports Whisper-Base-En, and normalizes the runtime model layout to:

```text
models/
  whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core/
    encoder.onnx
    decoder.onnx
```

### 4. Build the native app

```powershell
cargo build --release
```

The native executable is:

```text
target\release\nputella.exe
```

The Rust ONNX Runtime binding currently expects an ONNX Runtime 1.22.x DLL. The older DLL inside the Python `venv-arm64` may be 1.17.x and is not compatible with the native app. If the overlay starts but shows a model runtime error, install or place a matching ONNX Runtime/QNN 1.22.x runtime on `PATH` or beside `nputella.exe`.

### 5. Run

```powershell
target\release\nputella.exe
```

For the background launcher, `start_nputella.vbs` runs the native executable when it exists and falls back to the old Python runtime otherwise.

## Usage

| Action | Result |
|---|---|
| Hold `Right Alt` | Starts recording and shows red audio bars |
| Release `Right Alt` | Transcribes locally and pastes with `Ctrl+V` |
| Click idle overlay | Toggles recording |
| Tap under 300 ms | Ignored to prevent accidental triggers |

`f17.ahk` remaps `Right Alt` to `F17`, and the native app listens for `F17` through a low-level Windows keyboard hook.

## How It Works

```text
F17 down
  -> native cpal recorder captures mono audio
F17 up
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
main.py          legacy Python entry point
transcriber.py   legacy Python audio/inference implementation
overlay.py       legacy Tkinter overlay
setup.py         dependency install and Qualcomm AI Hub export
requirements.txt Python dependencies
f17.ahk          Right Alt -> F17 remap
keytest.py       diagnostic key listener
start_nputella.vbs hidden launcher for native exe with Python fallback
whisper-base-local/ local tokenizer and feature extractor config
models/          compiled ONNX model artifacts
```

## Troubleshooting

`models missing`: run `python setup.py` and confirm `encoder.onnx` and `decoder.onnx` exist in the nested runtime model directory.

`QNN EP not active`: check that `QnnHtp.dll` is on `PATH`. It usually ships with onnxruntime-qnn or the Qualcomm AI Runtime SDK.

Paste does not work in some apps: some apps block synthetic input. The transcript is still copied to the clipboard.

Export fails with device not found: edit `DEVICE` in `setup.py` or run `qai-hub list-devices` to find a valid target.

## Rewrite Notes

The native Rust rewrite implements these subsystems:

- Global F17 press/release listener with optional suppression.
- 16 kHz mono microphone capture with block-level RMS levels.
- Whisper-compatible log-mel preprocessing.
- ONNX Runtime sessions with QNN provider options and CPU fallback.
- Static-cache greedy decoder loop using the exported model's exact tensor names and shapes.
- Whisper BPE/tokenizer support from `whisper-base-local`.
- Borderless always-on-top floating overlay with recording, transcribing, done, loading, and error states.
- Native clipboard write plus synthetic `Ctrl+V`.

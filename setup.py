"""
setup.py - installs dependencies and exports Whisper-Base-En ONNX models
optimized for Snapdragon X Plus NPU via Qualcomm AI Hub Workbench.

Requires:
  - Qualcomm ID at https://aihub.qualcomm.com
  - qai-hub configured: qai-hub configure --api_token <YOUR_TOKEN>
  - AMD64/x64 Python for export tooling, not ARM64 Python
"""

import platform
import shutil
import struct
import subprocess
import sys
from pathlib import Path

MODEL_ROOT = Path(__file__).parent / "models"
RUNTIME_MODEL_DIR = (
    MODEL_ROOT
    / "whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core"
)
DEVICE = "Snapdragon X Plus 8-Core CRD"


def check_python_arch():
    bits = struct.calcsize("P") * 8
    machine = platform.machine().lower()
    if bits != 64 or "arm" in machine:
        print("ERROR: setup requires AMD64/x64 Python, not ARM64 or 32-bit Python.")
        print("Install the Windows x86-64 Python build from python.org.")
        print(f"Detected: machine={platform.machine()}, pointer_bits={bits}")
        sys.exit(1)


def install_deps():
    print("Installing dependencies from requirements.txt...")
    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", "-r", "requirements.txt"]
    )


def _copy_if_different(src: Path, dst: Path):
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.resolve() == dst.resolve():
        return
    shutil.copy2(src, dst)


def _find_exported_model(patterns: list[str]) -> Path | None:
    candidates: list[Path] = []
    for pattern in patterns:
        candidates.extend(MODEL_ROOT.rglob(pattern))
    candidates = [p for p in candidates if p.is_file()]
    if not candidates:
        return None

    def score(path: Path) -> tuple[int, int]:
        name = path.name.lower()
        preferred_name = int(name in {"encoder.onnx", "decoder.onnx"})
        return (preferred_name, -len(str(path)))

    return sorted(candidates, key=score, reverse=True)[0]


def normalize_model_layout() -> bool:
    encoder = _find_exported_model(["encoder.onnx", "*Encoder*.onnx", "*encoder*.onnx"])
    decoder = _find_exported_model(["decoder.onnx", "*Decoder*.onnx", "*decoder*.onnx"])

    if not encoder or not decoder:
        print("\nCould not find exported encoder/decoder ONNX files in:", MODEL_ROOT)
        print("Files present:")
        for path in sorted(MODEL_ROOT.rglob("*")):
            if path.is_file():
                print(" ", path.relative_to(MODEL_ROOT))
        return False

    _copy_if_different(encoder, RUNTIME_MODEL_DIR / "encoder.onnx")
    _copy_if_different(decoder, RUNTIME_MODEL_DIR / "decoder.onnx")
    print("Runtime models ready:")
    print(" ", RUNTIME_MODEL_DIR / "encoder.onnx")
    print(" ", RUNTIME_MODEL_DIR / "decoder.onnx")
    return True


def export_models():
    MODEL_ROOT.mkdir(exist_ok=True)

    print(f"\nExporting Whisper-Base-En ONNX for: {DEVICE}")
    print("This compiles on Qualcomm AI Hub cloud and downloads optimized ONNX.")
    print("Takes roughly 5-10 minutes on first run.\n")

    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "qai_hub_models.models.whisper_base_en.export",
            "--target-runtime",
            "onnx",
            "--device",
            DEVICE,
            "--output-dir",
            str(MODEL_ROOT),
            "--skip-profiling",
        ],
        capture_output=False,
    )

    if result.returncode != 0:
        print("\nExport failed. Common fixes:")
        print("  1. Run: qai-hub configure --api_token <your_token>")
        print("     Get token at: https://app.aihub.qualcomm.com/settings/")
        print("  2. Make sure you're using AMD64/x64 Python, not ARM64")
        print("  3. Try: pip install -U qai_hub_models")
        return False

    return normalize_model_layout()


if __name__ == "__main__":
    check_python_arch()
    install_deps()
    if not export_models():
        sys.exit(1)

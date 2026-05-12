"""
NPU-accelerated Whisper inference via ONNX QNN Execution Provider.
Targets Snapdragon X Plus Hexagon HTP (krypt0s).
"""

import numpy as np
import sounddevice as sd
import onnxruntime as ort
from pathlib import Path
import threading
from typing import Callable, Optional

SAMPLE_RATE  = 16000
HOP_LENGTH   = 160
CHUNK_SAMPLES = SAMPLE_RATE * 30

LOCAL_MODEL = Path(__file__).parent / "whisper-base-local"

# Whisper multilingual base token IDs
SOT          = 50258
EOT          = 50256
NOTIMESTAMPS = 50363
TRANSCRIBE   = 50359
LANG_EN      = 50259
MAX_DECODE   = 199
PROMPT_TOKENS = [SOT, LANG_EN, TRANSCRIBE, NOTIMESTAMPS]

MODEL_DIR    = Path(__file__).parent / "models" / "whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core"
ENCODER_PATH = MODEL_DIR / "encoder.onnx"
DECODER_PATH = MODEL_DIR / "decoder.onnx"

N_LAYERS = 6
N_HEADS  = 8
HEAD_DIM = 64


# ── NPU session ──────────────────────────────────────────────────────────────

def _qnn_session(model_path: Path) -> ort.InferenceSession:
    so = ort.SessionOptions()
    so.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
    qnn_options = {
        "backend_path": "QnnHtp.dll",
        "enable_htp_fp16_precision": "1",
        "htp_performance_mode": "burst",
        "htp_graph_finalization_optimization_mode": "3",
    }
    try:
        sess = ort.InferenceSession(
            str(model_path),
            sess_options=so,
            providers=[("QNNExecutionProvider", qnn_options)],
        )
        if not any("QNN" in p for p in sess.get_providers()):
            raise RuntimeError("QNN EP not active")
        return sess
    except Exception as e:
        print(f"[whisprnpu] QNN EP failed ({e}), falling back to CPU")
        return ort.InferenceSession(
            str(model_path),
            sess_options=so,
            providers=["CPUExecutionProvider"],
        )


# ── mel filterbank (pure numpy, HTK-style, matches Whisper preprocessing) ───

_MEL_FILTERS: Optional[np.ndarray] = None

def _get_mel_filters() -> np.ndarray:
    global _MEL_FILTERS
    if _MEL_FILTERS is not None:
        return _MEL_FILTERS
    n_mels, n_fft, sr, fmin, fmax = 80, 400, 16000, 0.0, 8000.0

    def hz_to_mel(f):
        return 2595.0 * np.log10(1.0 + f / 700.0)

    def mel_to_hz(m):
        return 700.0 * (10.0 ** (m / 2595.0) - 1.0)

    mel_points = np.linspace(hz_to_mel(fmin), hz_to_mel(fmax), n_mels + 2)
    hz_points  = mel_to_hz(mel_points)
    bins       = np.floor((n_fft + 1) * hz_points / sr).astype(int)
    filters    = np.zeros((n_mels, n_fft // 2 + 1), dtype=np.float32)

    for m in range(1, n_mels + 1):
        l, c, r = bins[m - 1], bins[m], bins[m + 1]
        for k in range(l, c):
            if c != l:
                filters[m - 1, k] = (k - l) / (c - l)
        for k in range(c, r):
            if r != c:
                filters[m - 1, k] = (r - k) / (r - c)

    _MEL_FILTERS = filters
    return _MEL_FILTERS


_feature_extractor = None

def _get_feature_extractor():
    global _feature_extractor
    if _feature_extractor is None:
        from transformers import WhisperFeatureExtractor
        _feature_extractor = WhisperFeatureExtractor.from_pretrained(str(LOCAL_MODEL))
    return _feature_extractor

def audio_to_mel(audio: np.ndarray) -> np.ndarray:
    audio = audio.astype(np.float32).flatten()
    fe = _get_feature_extractor()
    result = fe(audio, sampling_rate=SAMPLE_RATE, return_tensors="np")
    return result["input_features"].astype(np.float16)  # [1, 80, 3000]


# ── Whisper NPU inference ────────────────────────────────────────────────────

class WhisperNPU:
    def __init__(self, status_callback: Optional[Callable] = None):
        self.status_cb = status_callback or (lambda s: None)
        self._encoder  = None
        self._decoder  = None
        self._tokenizer = None
        self._ready    = False
        threading.Thread(target=self._load, daemon=True).start()

    def _load(self):
        self.status_cb("loading")
        try:
            if not ENCODER_PATH.exists() or not DECODER_PATH.exists():
                self.status_cb("models_missing")
                return
            self._encoder = _qnn_session(ENCODER_PATH)
            self._decoder = _qnn_session(DECODER_PATH)
            self._load_tokenizer()
            self._ready = True
            self.status_cb("ready")
        except Exception as e:
            self.status_cb(f"error:{e}")

    def _load_tokenizer(self):
        try:
            import tiktoken
            self._tokenizer = tiktoken.get_encoding("gpt2")
        except Exception:
            self._tokenizer = None

    def is_ready(self) -> bool:
        return self._ready

    def transcribe(self, audio: np.ndarray) -> str:
        if not self._ready:
            return ""

        MASK_NEG      = -100.0
        MEAN_DEC_LEN  = 200

        mel = audio_to_mel(audio)

        # encode → cross-attention KV cache
        enc_out  = self._encoder.run(None, {"input_features": mel})
        cross_kv = {}
        for i in range(N_LAYERS):
            cross_kv[f"k_cache_cross_{i}"] = enc_out[i * 2].astype(np.float16)
            cross_kv[f"v_cache_cross_{i}"] = enc_out[i * 2 + 1].astype(np.float16)

        # self-attention KV caches — sized to MEAN_DEC_LEN - 1 = 199
        k_self = {
            f"k_cache_self_{i}_in": np.zeros((N_HEADS, 1, HEAD_DIM, MEAN_DEC_LEN - 1), dtype=np.float16)
            for i in range(N_LAYERS)
        }
        v_self = {
            f"v_cache_self_{i}_in": np.zeros((N_HEADS, 1, MEAN_DEC_LEN - 1, HEAD_DIM), dtype=np.float16)
            for i in range(N_LAYERS)
        }

        # attention mask: fill with MASK_NEG, unmask as we go
        attn_mask = np.full((1, 1, 1, MEAN_DEC_LEN), MASK_NEG, dtype=np.float16)

        output_ids  = PROMPT_TOKENS.copy()
        position_id = 0

        for n in range(MEAN_DEC_LEN - 1):
            # current input token
            input_id = output_ids[n] if n < len(output_ids) else output_ids[-1]

            # unmask current position (from the right)
            attn_mask[0, 0, 0, MEAN_DEC_LEN - n - 1] = 0.0

            feed = {
                "input_ids":      np.array([[input_id]], dtype=np.int32),
                "attention_mask": attn_mask,
                "position_ids":   np.array([position_id], dtype=np.int32),
                **cross_kv,
                **k_self,
                **v_self,
            }

            out    = self._decoder.run(None, feed)
            logits = out[0]  # [1, 51865, 1, 1]
            for i in range(N_LAYERS):
                k_self[f"k_cache_self_{i}_in"] = out[1 + i * 2]
                v_self[f"v_cache_self_{i}_in"] = out[2 + i * 2]

            next_token = int(np.argmax(logits[0, :, 0, 0]))

            # only append generated tokens (after we've consumed the prompt)
            if n >= len(output_ids) - 1:
                if next_token == EOT:
                    break
                output_ids.append(next_token)

            position_id += 1

        # decode — use HF tokenizer with skip_special_tokens
        try:
            from transformers import WhisperTokenizer
            tok = WhisperTokenizer.from_pretrained(str(LOCAL_MODEL))
            return tok.decode(output_ids, skip_special_tokens=True).strip()
        except Exception:
            pass

        # fallback: tiktoken
        if self._tokenizer:
            try:
                text_tokens = [t for t in output_ids if t < 50256]
                return self._tokenizer.decode(text_tokens).strip()
            except Exception:
                pass

        return f"[{len(output_ids)} tokens]"

# ── Audio recorder (single stream, optional level callback for visualiser) ───

class AudioRecorder:
    def __init__(self):
        self._frames:   list[np.ndarray] = []
        self._recording = False
        self._stream    = None
        self._level_cb: Optional[Callable[[float], None]] = None

    def start(self, level_callback: Optional[Callable[[float], None]] = None):
        """Open the mic stream. level_callback(rms) fires every block."""
        self._frames    = []
        self._recording = True
        self._level_cb  = level_callback
        self._stream    = sd.InputStream(
            samplerate = SAMPLE_RATE,
            channels   = 1,
            dtype      = "float32",
            blocksize  = 512,           # smaller = more responsive meter
            callback   = self._cb,
        )
        self._stream.start()

    def stop(self) -> np.ndarray:
        self._recording = False
        self._level_cb  = None
        if self._stream:
            self._stream.stop()
            self._stream.close()
            self._stream = None
        if not self._frames:
            return np.zeros(SAMPLE_RATE, dtype=np.float32)
        return np.concatenate(self._frames).flatten()

    def _cb(self, indata: np.ndarray, frames: int, time_info, status):
        if not self._recording:
            return
        self._frames.append(indata.copy())
        if self._level_cb is not None:
            rms = float(np.sqrt(np.mean(indata ** 2)))
            self._level_cb(rms)

use crate::audio::Capture;
use crate::logger;
use crate::mel::audio_to_mel;
use anyhow::{anyhow, Context, Result};
use half::f16;
use ndarray::Array4;
use ort::execution_providers::qnn::QNNPerformanceMode;
use ort::execution_providers::{
    ExecutionProviderDispatch, QNNExecutionProvider,
};
use ort::session::Session;
use ort::value::TensorRef;
use std::path::Path;

const SOT: i64 = 50_258;
const EOT: i64 = 50_257;
const LEGACY_EOT: i64 = 50_256;
const NOTIMESTAMPS: i64 = 50_363;
const TRANSCRIBE: i64 = 50_359;
const LANG_EN: i64 = 50_259;
const MAX_DECODE: usize = 199;
const FIRST_SPECIAL: i64 = 50_257;

const MODEL_DIR_NAME: &str = "whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core";

pub struct WhisperEngine {
    encoder: Session,
    decoder: Session,
    tokenizer: tokenizers::Tokenizer,
}

impl WhisperEngine {
    pub fn load(root: &Path) -> Result<Self> {
        let model_dir = root.join("models").join(MODEL_DIR_NAME);
        let encoder_path = model_dir.join("encoder.onnx");
        let decoder_path = model_dir.join("decoder.onnx");
        logger::line(format!("model dir: {}", model_dir.display()));
        logger::line(format!(
            "encoder exists={} path={}",
            encoder_path.exists(),
            encoder_path.display()
        ));
        logger::line(format!(
            "decoder exists={} path={}",
            decoder_path.exists(),
            decoder_path.display()
        ));
        if !encoder_path.exists() || !decoder_path.exists() {
            return Err(anyhow!("models missing"));
        }

        let runtime_dir = root
            .join("venv-arm64")
            .join("Lib")
            .join("site-packages")
            .join("onnxruntime")
            .join("capi");
        let runtime_dll = runtime_dir.join("onnxruntime.dll");
        if runtime_dll.exists() {
            logger::line(format!(
                "initializing ONNX Runtime from {}",
                runtime_dll.display()
            ));
            let _ = ort::init_from(runtime_dll.to_string_lossy()).commit()?;
        } else {
            logger::line("initializing ONNX Runtime via default loader");
            let _ = ort::init().commit()?;
        }
        logger::line("ONNX Runtime initialized");

        let qnn_backend = runtime_dir.join("QnnHtp.dll");
        logger::line(format!(
            "QNN backend exists={} path={}",
            qnn_backend.exists(),
            qnn_backend.display()
        ));

        logger::line("loading encoder session");
        let encoder = Session::builder()?
            .with_execution_providers([make_qnn_ep(&qnn_backend)])?
            .commit_from_file(&encoder_path)
            .with_context(|| format!("loading encoder from {}", encoder_path.display()))?;
        logger::line("loading decoder session");
        let decoder = Session::builder()?
            .with_execution_providers([make_qnn_ep(&qnn_backend)])?
            .commit_from_file(&decoder_path)
            .with_context(|| format!("loading decoder from {}", decoder_path.display()))?;
        logger::line("loading tokenizer");
        let tokenizer = load_tokenizer(&root.join("whisper-base-local"))?;
        Ok(Self {
            encoder,
            decoder,
            tokenizer,
        })
    }

    pub fn transcribe(&mut self, capture: &Capture) -> Result<String> {
        let audio = resample_to_16k(&capture.samples, capture.sample_rate);
        logger::line(format!(
            "transcribe audio: input_samples={} input_rate={} mel_samples={}",
            capture.samples.len(),
            capture.sample_rate,
            audio.len()
        ));
        let mel = audio_to_mel(&audio);
        let mut mel_min = f32::INFINITY;
        let mut mel_max = f32::NEG_INFINITY;
        let mut mel_sum = 0.0f32;
        let mut mel_count = 0usize;
        for v in mel.iter() {
            let f = f32::from(*v);
            mel_min = mel_min.min(f);
            mel_max = mel_max.max(f);
            mel_sum += f;
            mel_count += 1;
        }
        logger::line(format!(
            "mel stats: min={:.6} max={:.6} mean={:.6}",
            mel_min,
            mel_max,
            mel_sum / mel_count.max(1) as f32
        ));
        let mel = mel.insert_axis(ndarray::Axis(0));

        let enc_outputs = self.encoder.run(ort::inputs![
            TensorRef::from_array_view(mel.view())?
        ])?;

        let mut cross_k = Vec::with_capacity(6);
        let mut cross_v = Vec::with_capacity(6);
        for layer in 0..6 {
            cross_k.push(
                enc_outputs[layer * 2]
                    .try_extract_array::<f16>()?
                    .to_owned(),
            );
            cross_v.push(
                enc_outputs[layer * 2 + 1]
                    .try_extract_array::<f16>()?
                    .to_owned(),
            );
        }

        let mut self_k: Vec<Array4<f16>> = (0..6)
            .map(|_| Array4::from_elem((8, 1, 64, 199), f16::from_f32(0.0)))
            .collect();
        let mut self_v: Vec<Array4<f16>> = (0..6)
            .map(|_| Array4::from_elem((8, 1, 199, 64), f16::from_f32(0.0)))
            .collect();
        let mut attention_mask = Array4::from_elem((1, 1, 1, 200), f16::from_f32(-100.0));

        let prompt_len = 4;
        let mut output_ids = vec![SOT, LANG_EN, TRANSCRIBE, NOTIMESTAMPS];
        let mut position_id = 0i32;

        for n in 0..MAX_DECODE {
            let input_id = *output_ids.get(n).unwrap_or(output_ids.last().unwrap());
            attention_mask[[0, 0, 0, MAX_DECODE - n]] = f16::from_f32(0.0);

            let input_ids = ndarray::Array2::from_elem((1, 1), input_id as i32);
            let position_ids = ndarray::Array1::from_elem(1, position_id);

            let outputs = self.decoder.run(ort::inputs![
                "input_ids" => TensorRef::from_array_view(input_ids.view())?,
                "attention_mask" => TensorRef::from_array_view(attention_mask.view())?,
                "position_ids" => TensorRef::from_array_view(position_ids.view())?,
                "k_cache_self_0_in" => TensorRef::from_array_view(self_k[0].view())?,
                "v_cache_self_0_in" => TensorRef::from_array_view(self_v[0].view())?,
                "k_cache_self_1_in" => TensorRef::from_array_view(self_k[1].view())?,
                "v_cache_self_1_in" => TensorRef::from_array_view(self_v[1].view())?,
                "k_cache_self_2_in" => TensorRef::from_array_view(self_k[2].view())?,
                "v_cache_self_2_in" => TensorRef::from_array_view(self_v[2].view())?,
                "k_cache_self_3_in" => TensorRef::from_array_view(self_k[3].view())?,
                "v_cache_self_3_in" => TensorRef::from_array_view(self_v[3].view())?,
                "k_cache_self_4_in" => TensorRef::from_array_view(self_k[4].view())?,
                "v_cache_self_4_in" => TensorRef::from_array_view(self_v[4].view())?,
                "k_cache_self_5_in" => TensorRef::from_array_view(self_k[5].view())?,
                "v_cache_self_5_in" => TensorRef::from_array_view(self_v[5].view())?,
                "k_cache_cross_0" => TensorRef::from_array_view(cross_k[0].view())?,
                "v_cache_cross_0" => TensorRef::from_array_view(cross_v[0].view())?,
                "k_cache_cross_1" => TensorRef::from_array_view(cross_k[1].view())?,
                "v_cache_cross_1" => TensorRef::from_array_view(cross_v[1].view())?,
                "k_cache_cross_2" => TensorRef::from_array_view(cross_k[2].view())?,
                "v_cache_cross_2" => TensorRef::from_array_view(cross_v[2].view())?,
                "k_cache_cross_3" => TensorRef::from_array_view(cross_k[3].view())?,
                "v_cache_cross_3" => TensorRef::from_array_view(cross_v[3].view())?,
                "k_cache_cross_4" => TensorRef::from_array_view(cross_k[4].view())?,
                "v_cache_cross_4" => TensorRef::from_array_view(cross_v[4].view())?,
                "k_cache_cross_5" => TensorRef::from_array_view(cross_k[5].view())?,
                "v_cache_cross_5" => TensorRef::from_array_view(cross_v[5].view())?,
            ])?;

            let logits = outputs[0].try_extract_array::<f16>()?;
            let mut best_token = 0i64;
            let mut best_score = f32::NEG_INFINITY;
            for (idx, value) in logits.iter().enumerate() {
                let score = f32::from(*value);
                if score > best_score {
                    best_score = score;
                    best_token = idx as i64;
                }
            }

            for layer in 0..6 {
                self_k[layer] = outputs[1 + layer * 2]
                    .try_extract_array::<f16>()?
                    .to_owned()
                    .into_dimensionality()?;
                self_v[layer] = outputs[2 + layer * 2]
                    .try_extract_array::<f16>()?
                    .to_owned()
                    .into_dimensionality()?;
            }

            if n >= output_ids.len() - 1 {
                if best_token == EOT || best_token == LEGACY_EOT {
                    break;
                }
                output_ids.push(best_token);
            }
            position_id += 1;
        }

        logger::line(format!("generated token ids: {:?}", output_ids));
        let ids: Vec<u32> = output_ids[prompt_len..]
            .iter()
            .copied()
            .filter(|token| *token < FIRST_SPECIAL)
            .map(|v| v as u32)
            .collect();
        logger::line(format!("decoded token ids: {:?}", ids));
        let text = self
            .tokenizer
            .decode(&ids, false)
            .unwrap_or_else(|_| format!("[{} tokens]", ids.len()));
        Ok(text.trim().to_string())
    }
}

fn resample_to_16k(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    if sample_rate == 16_000 || samples.is_empty() {
        return samples.to_vec();
    }
    let out_len = ((samples.len() as f64) * 16_000.0 / sample_rate as f64).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    let ratio = sample_rate as f64 / 16_000.0;
    for i in 0..out_len {
        let src = i as f64 * ratio;
        let idx = src.floor() as usize;
        let frac = (src - idx as f64) as f32;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

fn load_tokenizer(path: &Path) -> Result<tokenizers::Tokenizer> {
    use tokenizers::decoders::byte_level::ByteLevel as ByteLevelDecoder;
    use tokenizers::models::bpe::BPE;
    use tokenizers::Tokenizer;

    let vocab = path.join("vocab.json").to_string_lossy().to_string();
    let merges = path.join("merges.txt").to_string_lossy().to_string();
    let model = BPE::from_file(&vocab, &merges)
        .build()
        .map_err(|err| anyhow!("building whisper BPE tokenizer: {err}"))?;
    let mut tokenizer = Tokenizer::new(model);
    tokenizer.with_decoder(Some(ByteLevelDecoder::default()));
    Ok(tokenizer)
}

fn make_qnn_ep(qnn_backend: &Path) -> ExecutionProviderDispatch {
    QNNExecutionProvider::default()
        .with_backend_path(qnn_backend.to_string_lossy())
        .with_htp_fp16_precision(true)
        .with_htp_graph_finalization_optimization_mode(3)
        .with_performance_mode(QNNPerformanceMode::Burst)
        .build()
        .fail_silently()
}

use crate::audio::Capture;
use crate::config::Language;
use crate::logger;
use crate::mel::audio_to_mel;
use anyhow::{anyhow, Context, Result};
use half::f16;
use ndarray::{Array1, Array2, Array4, ArrayViewD};
use ort::ep::{qnn::PerformanceMode, ExecutionProviderDispatch, QNN};
use ort::session::Session;
use ort::value::TensorRef;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::System::LibraryLoader::SetDllDirectoryW;

const SOT: i64 = 50_258;
const EOT: i64 = 50_257;
const LEGACY_EOT: i64 = 50_256;
const NOTIMESTAMPS: i64 = 50_363;
const TRANSCRIBE: i64 = 50_359;
const LANG_EN: i64 = 50_259;
const LANG_FR: i64 = 50_265;
const MAX_DECODE: usize = 199;
const FIRST_SPECIAL: i64 = 50_257;

const MODEL_DIR_NAME: &str =
    "whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core";

pub struct WhisperEngine {
    encoder: Session,
    decoder: Session,
    tokenizer: tokenizers::Tokenizer,
    bias: DecoderBias,
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

        let runtime_dir = runtime_onnx_dir(root);
        let runtime_dll = runtime_dir.join("onnxruntime.dll");
        if runtime_dll.exists() {
            add_runtime_dll_search_path(&runtime_dir)?;
            logger::line(format!(
                "initializing ONNX Runtime from {}",
                runtime_dll.display()
            ));
            let committed = ort::init_from(&runtime_dll)
                .with_context(|| {
                    format!("configuring ONNX Runtime from {}", runtime_dll.display())
                })?
                .commit();
            if !committed {
                logger::line("ONNX Runtime environment already configured");
            }
        } else {
            logger::line("initializing ONNX Runtime via default loader");
            let committed = ort::init().commit();
            if !committed {
                logger::line("ONNX Runtime environment already configured");
            }
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
            .with_execution_providers([make_qnn_ep(&qnn_backend)])
            .map_err(|err| anyhow!("configuring encoder session: {err}"))?
            .commit_from_file(&encoder_path)
            .with_context(|| format!("loading encoder from {}", encoder_path.display()))?;
        logger::line("loading decoder session");
        let decoder = Session::builder()?
            .with_execution_providers([make_qnn_ep(&qnn_backend)])
            .map_err(|err| anyhow!("configuring decoder session: {err}"))?
            .commit_from_file(&decoder_path)
            .with_context(|| format!("loading decoder from {}", decoder_path.display()))?;
        logger::line("loading tokenizer");
        let tokenizer = load_tokenizer(&root.join("whisper-base-local"))?;
        let bias = DecoderBias::disabled();
        Ok(Self {
            encoder,
            decoder,
            tokenizer,
            bias,
        })
    }

    pub fn transcribe(&mut self, capture: &Capture, language: Language) -> Result<String> {
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

        let enc_outputs = self
            .encoder
            .run(ort::inputs![TensorRef::from_array_view(&mel)?])?;

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

        let mut output_ids = match language {
            Language::Auto => vec![SOT],
            Language::English | Language::French => prompt_tokens(language),
        };
        let prompt_len = 4;
        logger::line(format!("decoder prompt tokens: {:?}", output_ids));
        let mut position_id = 0i32;
        let mut detected_language = None;

        for n in 0..MAX_DECODE {
            let input_id = *output_ids.get(n).unwrap_or(output_ids.last().unwrap());
            attention_mask[[0, 0, 0, MAX_DECODE - n]] = f16::from_f32(0.0);

            let input_ids = Array2::from_elem((1, 1), input_id as i32);
            let position_ids = Array1::from_elem(1, position_id);

            let outputs = self.decoder.run(ort::inputs![
                "input_ids" => TensorRef::from_array_view(&input_ids)?,
                "attention_mask" => TensorRef::from_array_view(&attention_mask)?,
                "position_ids" => TensorRef::from_array_view(&position_ids)?,
                "k_cache_self_0_in" => TensorRef::from_array_view(&self_k[0])?,
                "v_cache_self_0_in" => TensorRef::from_array_view(&self_v[0])?,
                "k_cache_self_1_in" => TensorRef::from_array_view(&self_k[1])?,
                "v_cache_self_1_in" => TensorRef::from_array_view(&self_v[1])?,
                "k_cache_self_2_in" => TensorRef::from_array_view(&self_k[2])?,
                "v_cache_self_2_in" => TensorRef::from_array_view(&self_v[2])?,
                "k_cache_self_3_in" => TensorRef::from_array_view(&self_k[3])?,
                "v_cache_self_3_in" => TensorRef::from_array_view(&self_v[3])?,
                "k_cache_self_4_in" => TensorRef::from_array_view(&self_k[4])?,
                "v_cache_self_4_in" => TensorRef::from_array_view(&self_v[4])?,
                "k_cache_self_5_in" => TensorRef::from_array_view(&self_k[5])?,
                "v_cache_self_5_in" => TensorRef::from_array_view(&self_v[5])?,
                "k_cache_cross_0" => TensorRef::from_array_view(&cross_k[0])?,
                "v_cache_cross_0" => TensorRef::from_array_view(&cross_v[0])?,
                "k_cache_cross_1" => TensorRef::from_array_view(&cross_k[1])?,
                "v_cache_cross_1" => TensorRef::from_array_view(&cross_v[1])?,
                "k_cache_cross_2" => TensorRef::from_array_view(&cross_k[2])?,
                "v_cache_cross_2" => TensorRef::from_array_view(&cross_v[2])?,
                "k_cache_cross_3" => TensorRef::from_array_view(&cross_k[3])?,
                "v_cache_cross_3" => TensorRef::from_array_view(&cross_v[3])?,
                "k_cache_cross_4" => TensorRef::from_array_view(&cross_k[4])?,
                "v_cache_cross_4" => TensorRef::from_array_view(&cross_v[4])?,
                "k_cache_cross_5" => TensorRef::from_array_view(&cross_k[5])?,
                "v_cache_cross_5" => TensorRef::from_array_view(&cross_v[5])?,
            ])?;

            let logits = outputs[0].try_extract_array::<f16>()?;
            let mut best_token = 0i64;
            let mut best_score = f32::NEG_INFINITY;
            for (idx, value) in logits.iter().enumerate() {
                let score = f32::from(*value) + self.bias.score(idx as i64, &output_ids);
                if score > best_score {
                    best_score = score;
                    best_token = idx as i64;
                }
            }
            if language == Language::Auto && detected_language.is_none() {
                let en_score = token_score(&logits, LANG_EN).unwrap_or(f32::NEG_INFINITY);
                let fr_score = token_score(&logits, LANG_FR).unwrap_or(f32::NEG_INFINITY);
                let lang = if fr_score > en_score {
                    LANG_FR
                } else {
                    LANG_EN
                };
                detected_language = Some(lang);
                output_ids.push(lang);
                output_ids.push(TRANSCRIBE);
                output_ids.push(NOTIMESTAMPS);
                logger::line(format!(
                    "detected language token={} en_score={:.4} fr_score={:.4}",
                    lang, en_score, fr_score
                ));
            }

            for layer in 0..6 {
                self_k[layer] = outputs[1 + layer * 2]
                    .try_extract_array::<f16>()?
                    .to_owned()
                    .into_dimensionality::<ndarray::Ix4>()?;
                self_v[layer] = outputs[2 + layer * 2]
                    .try_extract_array::<f16>()?
                    .to_owned()
                    .into_dimensionality::<ndarray::Ix4>()?;
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

#[derive(Clone, Debug)]
struct DecoderBias {
    enabled: bool,
    phrases: Vec<BiasedPhrase>,
    first_token_bias: f32,
    next_token_bias: f32,
}

#[derive(Clone, Debug)]
struct BiasedPhrase {
    tokens: Vec<i64>,
    priority: BiasPriority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BiasPriority {
    Normal,
    High,
}

impl DecoderBias {
    fn disabled() -> Self {
        let _ = BiasPriority::Normal;
        Self {
            enabled: false,
            phrases: Vec::new(),
            first_token_bias: 0.15,
            next_token_bias: 0.45,
        }
    }

    fn score(&self, token: i64, output_ids: &[i64]) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let mut bias = 0.0f32;
        for phrase in &self.phrases {
            if phrase.tokens.is_empty() {
                continue;
            }
            if phrase.tokens[0] == token && phrase.priority == BiasPriority::High {
                bias = bias.max(self.first_token_bias);
            }
            if let Some(next) = next_phrase_token(output_ids, &phrase.tokens) {
                if next == token {
                    bias = bias.max(self.next_token_bias);
                }
            }
        }
        bias
    }
}

fn next_phrase_token(output_ids: &[i64], phrase: &[i64]) -> Option<i64> {
    let max_prefix = phrase.len().saturating_sub(1).min(output_ids.len());
    for prefix_len in (1..=max_prefix).rev() {
        if output_ids.ends_with(&phrase[..prefix_len]) {
            return phrase.get(prefix_len).copied();
        }
    }
    None
}

fn prompt_tokens(language: Language) -> Vec<i64> {
    match language {
        Language::English => vec![SOT, LANG_EN, TRANSCRIBE, NOTIMESTAMPS],
        Language::French => vec![SOT, LANG_FR, TRANSCRIBE, NOTIMESTAMPS],
        Language::Auto => vec![SOT],
    }
}

fn token_score(logits: &ArrayViewD<f16>, token: i64) -> Option<f32> {
    logits
        .iter()
        .nth(token as usize)
        .map(|value| f32::from(*value))
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
    QNN::default()
        .with_backend_path(qnn_backend.display().to_string())
        .with_htp_fp16_precision(true)
        .with_htp_graph_finalization_optimization_mode(3)
        .with_performance_mode(PerformanceMode::Burst)
        .build()
        .fail_silently()
}

fn add_runtime_dll_search_path(runtime_dir: &Path) -> Result<()> {
    let wide: Vec<u16> = runtime_dir
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe {
        SetDllDirectoryW(PCWSTR(wide.as_ptr()))
            .ok()
            .with_context(|| format!("adding runtime DLL search path {}", runtime_dir.display()))?;
    }
    logger::line(format!(
        "added runtime DLL search path {}",
        runtime_dir.display()
    ));
    Ok(())
}

fn runtime_onnx_dir(root: &Path) -> std::path::PathBuf {
    let bundled = root.join("runtime").join("onnxruntime").join("capi");
    if bundled.join("onnxruntime.dll").is_file() {
        return bundled;
    }

    root.join("venv-arm64")
        .join("Lib")
        .join("site-packages")
        .join("onnxruntime")
        .join("capi")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_tokens_force_transcription_language() {
        assert_eq!(
            prompt_tokens(Language::English),
            vec![SOT, LANG_EN, TRANSCRIBE, NOTIMESTAMPS]
        );
        assert_eq!(
            prompt_tokens(Language::French),
            vec![SOT, LANG_FR, TRANSCRIBE, NOTIMESTAMPS]
        );
        assert_eq!(prompt_tokens(Language::Auto), vec![SOT]);
    }

    #[test]
    fn decoder_bias_only_continues_matching_phrase() {
        let bias = DecoderBias {
            enabled: true,
            phrases: vec![BiasedPhrase {
                tokens: vec![10, 11, 12],
                priority: BiasPriority::Normal,
            }],
            first_token_bias: 0.15,
            next_token_bias: 0.45,
        };
        assert_eq!(bias.score(11, &[SOT, 10]), 0.45);
        assert_eq!(bias.score(12, &[SOT, 10, 11]), 0.45);
        assert_eq!(bias.score(12, &[SOT, 99]), 0.0);
    }

    #[test]
    fn high_priority_bias_gets_small_first_token_boost() {
        let bias = DecoderBias {
            enabled: true,
            phrases: vec![BiasedPhrase {
                tokens: vec![42, 43],
                priority: BiasPriority::High,
            }],
            first_token_bias: 0.15,
            next_token_bias: 0.45,
        };
        assert_eq!(
            bias.score(42, &[SOT, LANG_EN, TRANSCRIBE, NOTIMESTAMPS]),
            0.15
        );
    }
}

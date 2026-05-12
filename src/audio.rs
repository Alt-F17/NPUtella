use crate::logger;
use crate::ui::AppEvent;
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Sample, SampleFormat, SampleRate, Stream, StreamConfig};
use crossbeam_channel::Sender;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

pub struct AudioRecorder {
    state: Arc<Mutex<RecordState>>,
    stream: Option<Stream>,
}

struct RecordState {
    active: bool,
    sample_rate: u32,
    channels: usize,
    frames: Vec<f32>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordState {
                active: false,
                sample_rate: 16_000,
                channels: 1,
                frames: Vec::new(),
            })),
            stream: None,
        }
    }

    pub fn start(&mut self, tx: Sender<AppEvent>) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }
        let host = cpal::default_host();
        let device = host.default_input_device().context("no input device found")?;
        let default_config = device
            .default_input_config()
            .context("could not query default input config")?;
        logger::line(format!(
            "audio device: '{}' default_sample_rate={} default_channels={} default_format={:?}",
            device.name().unwrap_or_else(|_| "unknown".to_string()),
            default_config.sample_rate().0,
            default_config.channels(),
            default_config.sample_format()
        ));

        let candidates = input_candidates(&device, &default_config);
        let mut last_error = None;
        let mut selected = None;

        for candidate in candidates {
            logger::line(format!(
                "audio stream candidate: {} sample_rate={} channels={} format={:?} buffer={:?}",
                candidate.reason,
                candidate.config.sample_rate.0,
                candidate.config.channels,
                candidate.format,
                candidate.config.buffer_size
            ));
            match build_stream(&device, candidate.format, &candidate.config, self.state.clone(), tx.clone()) {
                Ok(stream) => {
                    selected = Some((stream, candidate));
                    break;
                }
                Err(err) => {
                    logger::line(format!("audio stream candidate failed: {err:#}"));
                    last_error = Some(err);
                }
            }
        }

        let (stream, selected_config) = selected.ok_or_else(|| {
            anyhow!(
                "could not open input stream{}",
                last_error
                    .as_ref()
                    .map(|err| format!(": {err:#}"))
                    .unwrap_or_default()
            )
        })?;

        {
            let mut guard = self.state.lock().map_err(|_| anyhow!("audio state poisoned"))?;
            guard.active = true;
            guard.sample_rate = selected_config.config.sample_rate.0;
            guard.channels = selected_config.config.channels as usize;
            guard.frames.clear();
        }
        stream.play().context("starting input stream")?;
        logger::line(format!(
            "audio start: selected={} sample_rate={} channels={} format={:?}",
            selected_config.reason,
            selected_config.config.sample_rate.0,
            selected_config.config.channels,
            selected_config.format
        ));
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Capture {
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        let mut guard = match self.state.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return Capture {
                    samples: Vec::new(),
                    sample_rate: 16_000,
                }
            }
        };
        guard.active = false;
        let sample_rate = guard.sample_rate;
        let frames = std::mem::take(&mut guard.frames);
        let rms = if frames.is_empty() {
            0.0
        } else {
            (frames.iter().map(|v| v * v).sum::<f32>() / frames.len() as f32).sqrt()
        };
        let peak = frames.iter().fold(0.0f32, |acc, v| acc.max(v.abs()));
        logger::line(format!(
            "audio stop: samples={} sample_rate={} duration_s={:.3} rms={:.6} peak={:.6}",
            frames.len(),
            sample_rate,
            frames.len() as f32 / sample_rate.max(1) as f32,
            rms,
            peak
        ));
        if let Err(err) = write_debug_wav("nputella_last_capture.wav", &frames, sample_rate) {
            logger::line(format!("debug wav write failed: {err:#}"));
        }
        Capture {
            samples: frames,
            sample_rate,
        }
    }
}

pub struct Capture {
    pub samples: Vec<f32>,
    #[allow(dead_code)]
    pub sample_rate: u32,
}

struct InputCandidate {
    format: SampleFormat,
    config: StreamConfig,
    reason: &'static str,
}

fn input_candidates(
    device: &cpal::Device,
    default_config: &cpal::SupportedStreamConfig,
) -> Vec<InputCandidate> {
    let mut candidates = Vec::new();
    if let Ok(supported) = device.supported_input_configs() {
        for range in supported {
            logger::line(format!(
                "audio supported: channels={} min_rate={} max_rate={} format={:?} buffer_unknown",
                range.channels(),
                range.min_sample_rate().0,
                range.max_sample_rate().0,
                range.sample_format()
            ));
            if range.channels() == 1
                && range.sample_format() == SampleFormat::F32
                && range.min_sample_rate().0 <= 16_000
                && range.max_sample_rate().0 >= 16_000
            {
                let mut config = range.with_sample_rate(SampleRate(16_000)).config();
                config.buffer_size = BufferSize::Fixed(512);
                candidates.push(InputCandidate {
                    format: SampleFormat::F32,
                    config,
                    reason: "python-match-16k-mono-f32",
                });
            }
        }
    }

    let mut default_fixed: StreamConfig = default_config.clone().into();
    default_fixed.buffer_size = BufferSize::Fixed(512);
    candidates.push(InputCandidate {
        format: default_config.sample_format(),
        config: default_fixed,
        reason: "default-format-fixed-512",
    });

    let default_plain: StreamConfig = default_config.clone().into();
    candidates.push(InputCandidate {
        format: default_config.sample_format(),
        config: default_plain,
        reason: "default-format-default-buffer",
    });

    candidates
}

fn build_stream(
    device: &cpal::Device,
    format: SampleFormat,
    config: &StreamConfig,
    state: Arc<Mutex<RecordState>>,
    tx: Sender<AppEvent>,
) -> Result<Stream> {
    let err_tx = tx.clone();
    let stream = match format {
        SampleFormat::F32 => device.build_input_stream(
            config,
            move |data: &[f32], _| handle_input(data, &state, &tx),
            move |err| {
                logger::line(format!("audio stream error: {err}"));
                let _ = err_tx.send(AppEvent::TranscriptionError(format!("audio stream error: {err}")));
            },
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            config,
            move |data: &[i16], _| handle_input(data, &state, &tx),
            move |err| {
                logger::line(format!("audio stream error: {err}"));
                let _ = err_tx.send(AppEvent::TranscriptionError(format!("audio stream error: {err}")));
            },
            None,
        )?,
        SampleFormat::U16 => device.build_input_stream(
            config,
            move |data: &[u16], _| handle_input(data, &state, &tx),
            move |err| {
                logger::line(format!("audio stream error: {err}"));
                let _ = err_tx.send(AppEvent::TranscriptionError(format!("audio stream error: {err}")));
            },
            None,
        )?,
        other => return Err(anyhow!("unsupported input sample format: {other:?}")),
    };
    Ok(stream)
}

fn handle_input<T>(data: &[T], state: &Arc<Mutex<RecordState>>, tx: &Sender<AppEvent>)
where
    T: Sample,
    f32: cpal::FromSample<T>,
{
    let mut guard = match state.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    if !guard.active {
        return;
    }

    let channels = guard.channels.max(1);
    let mut level_acc = 0.0f32;
    let mut level_count = 0usize;
    for frame in data.chunks(channels) {
        let mut mono = 0.0f32;
        for sample in frame {
            let s = f32::from_sample(*sample);
            mono += s;
            level_acc += s * s;
            level_count += 1;
        }
        mono /= frame.len() as f32;
        guard.frames.push(mono);
    }
    if level_count > 0 {
        let rms = (level_acc / level_count as f32).sqrt();
        let _ = tx.send(AppEvent::AudioLevel(rms));
    }
}

fn write_debug_wav(path: &str, samples: &[f32], sample_rate: u32) -> Result<()> {
    let mut file = File::create(path)?;
    let channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    let block_align = channels * bits_per_sample / 8;

    file.write_all(b"RIFF")?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(b"WAVE")?;
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&0u32.to_le_bytes())?;

    let mut data_bytes = 0u32;
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm = (clamped * i16::MAX as f32) as i16;
        file.write_all(&pcm.to_le_bytes())?;
        data_bytes += 2;
    }

    let riff_size = 36 + data_bytes;
    file.seek(SeekFrom::Start(4))?;
    file.write_all(&riff_size.to_le_bytes())?;
    file.seek(SeekFrom::Start(40))?;
    file.write_all(&data_bytes.to_le_bytes())?;
    logger::line(format!(
        "debug wav written: {} samples={} sample_rate={}",
        path,
        samples.len(),
        sample_rate
    ));
    Ok(())
}

use half::f16;
use ndarray::Array2;
use once_cell::sync::OnceCell;
use rustfft::{num_complex::Complex32, FftPlanner};

const SAMPLE_RATE: usize = 16_000;
const N_FFT: usize = 400;
const HOP_LENGTH: usize = 160;
const N_MELS: usize = 80;
const N_FRAMES: usize = 3000;
const N_SAMPLES: usize = 480_000;
const PAD: usize = N_FFT / 2;

static MEL_FILTERS: OnceCell<Array2<f32>> = OnceCell::new();

pub fn audio_to_mel(samples: &[f32]) -> Array2<f16> {
    let audio = pad_or_trim(samples, N_SAMPLES);
    let padded = reflect_pad(&audio, PAD);
    let filters = MEL_FILTERS.get_or_init(build_mel_filters);
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(N_FFT);
    let window = hann_window(N_FFT);
    let mut fft_buf = vec![Complex32::default(); N_FFT];
    let mut mel = Array2::<f32>::zeros((N_MELS, N_FRAMES));

    for frame_idx in 0..N_FRAMES {
        let offset = frame_idx * HOP_LENGTH;
        for i in 0..N_FFT {
            fft_buf[i].re = padded[offset + i] * window[i];
            fft_buf[i].im = 0.0;
        }
        fft.process(&mut fft_buf);
        for m in 0..N_MELS {
            let mut sum = 0.0f32;
            for k in 0..=(N_FFT / 2) {
                let c = fft_buf[k];
                let power = c.re * c.re + c.im * c.im;
                sum += filters[[m, k]] * power;
            }
            mel[[m, frame_idx]] = sum;
        }
    }

    let mut max_val = f32::NEG_INFINITY;
    for v in mel.iter_mut() {
        *v = (*v).max(1e-10).log10();
        max_val = max_val.max(*v);
    }
    let floor = max_val - 8.0;
    for v in mel.iter_mut() {
        *v = (*v).max(floor);
        *v = (*v + 4.0) / 4.0;
    }

    mel.mapv(f16::from_f32)
}

fn pad_or_trim(samples: &[f32], target: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; target];
    let len = samples.len().min(target);
    out[..len].copy_from_slice(&samples[..len]);
    out
}

fn reflect_pad(samples: &[f32], pad: usize) -> Vec<f32> {
    let len = samples.len();
    let total = len + pad * 2;
    let mut out = Vec::with_capacity(total);
    for i in 0..total {
        let idx = reflect_index(i as isize - pad as isize, len as isize);
        out.push(samples[idx]);
    }
    out
}

fn reflect_index(mut idx: isize, len: isize) -> usize {
    if len <= 1 {
        return 0;
    }
    loop {
        if idx < 0 {
            idx = -idx;
        } else if idx >= len {
            idx = 2 * len - idx - 2;
        } else {
            return idx as usize;
        }
    }
}

fn hann_window(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| 0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / n as f32).cos())
        .collect()
}

fn build_mel_filters() -> Array2<f32> {
    let sr = SAMPLE_RATE as f32;
    let mel_min = hertz_to_mel(0.0, "slaney");
    let mel_max = hertz_to_mel(8000.0, "slaney");
    let mel_freqs: Vec<f32> = (0..(N_MELS + 2))
        .map(|i| mel_min + (i as f32 / (N_MELS + 1) as f32) * (mel_max - mel_min))
        .collect();
    let filter_freqs: Vec<f32> = mel_freqs
        .iter()
        .copied()
        .map(|m| mel_to_hertz(m, "slaney"))
        .collect();
    let fft_freqs: Vec<f32> = (0..=(N_FFT / 2))
        .map(|i| i as f32 * sr / N_FFT as f32)
        .collect();

    let mut filters = Array2::<f32>::zeros((N_MELS, N_FFT / 2 + 1));
    for i in 0..fft_freqs.len() {
        let f = fft_freqs[i];
        for m in 0..N_MELS {
            let left = filter_freqs[m];
            let center = filter_freqs[m + 1];
            let right = filter_freqs[m + 2];
            let val = if f < left || f > right {
                0.0
            } else if f <= center {
                if center > left {
                    (f - left) / (center - left)
                } else {
                    0.0
                }
            } else if right > center {
                (right - f) / (right - center)
            } else {
                0.0
            };
            filters[[m, i]] = val.max(0.0);
        }
    }

    // Slaney area normalization.
    for m in 0..N_MELS {
        let denom = filter_freqs[m + 2] - filter_freqs[m];
        if denom > 0.0 {
            let scale = 2.0 / denom;
            for k in 0..filters.ncols() {
                filters[[m, k]] *= scale;
            }
        }
    }

    filters
}

fn hertz_to_mel(freq: f32, mel_scale: &str) -> f32 {
    match mel_scale {
        "htk" => 2595.0 * (1.0 + freq / 700.0).log10(),
        "kaldi" => 1127.0 * (1.0 + freq / 700.0).ln(),
        "slaney" => {
            let min_log_hertz = 1000.0;
            let min_log_mel = 15.0;
            let logstep = 27.0 / 6.4f32.ln();
            if freq < min_log_hertz {
                3.0 * freq / 200.0
            } else {
                min_log_mel + (freq / min_log_hertz).ln() * logstep
            }
        }
        _ => unreachable!(),
    }
}

fn mel_to_hertz(mels: f32, mel_scale: &str) -> f32 {
    match mel_scale {
        "htk" => 700.0 * (10f32.powf(mels / 2595.0) - 1.0),
        "kaldi" => 700.0 * ((mels / 1127.0).exp() - 1.0),
        "slaney" => {
            let min_log_hertz = 1000.0;
            let min_log_mel = 15.0;
            let logstep = 6.4f32.ln() / 27.0;
            if mels < min_log_mel {
                200.0 * mels / 3.0
            } else {
                min_log_hertz * ((mels - min_log_mel) * logstep).exp()
            }
        }
        _ => unreachable!(),
    }
}

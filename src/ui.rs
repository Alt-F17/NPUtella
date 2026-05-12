use eframe::egui;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const IDLE_W: f32 = 46.0;
pub const IDLE_H: f32 = 18.0;
pub const ACTIVE_W: f32 = 128.0;
pub const ACTIVE_H: f32 = 30.0;
pub const SCREEN_BOTTOM_OFFSET: f32 = 60.0;
pub const WINDOW_PAD: f32 = 5.0;

const BAR_COUNT: usize = 15;
const BAR_W: f32 = 2.0;
const BAR_GAP: f32 = 2.0;
const BAR_MAX_H: f32 = 16.0;
const BAR_MIN_H: f32 = 2.0;

#[derive(Clone, Debug)]
pub enum EngineStatus {
    Loading,
    Ready,
    ModelsMissing,
    Error(String),
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    EngineStatus(EngineStatus),
    HotkeyDown,
    HotkeyUp,
    AudioLevel(f32),
    TranscriptionDone(String),
    TranscriptionError(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppPhase {
    Loading,
    Idle,
    Recording,
    Transcribing,
    Done,
    Error,
}

pub struct OverlayState {
    pub phase: AppPhase,
    pub engine_status: EngineStatus,
    pub text: String,
    pub audio_level: f32,
    pub hovered: bool,
    collapse_at: Option<Instant>,
}

impl OverlayState {
    pub fn loading() -> Self {
        Self {
            phase: AppPhase::Loading,
            engine_status: EngineStatus::Loading,
            text: "loading".to_string(),
            audio_level: 0.0,
            hovered: false,
            collapse_at: None,
        }
    }

    pub fn tick(&mut self) {
        if let Some(when) = self.collapse_at {
            if Instant::now() >= when {
                self.shrink_to_idle();
            }
        }
    }

    pub fn set_engine_status(&mut self, status: EngineStatus) {
        self.engine_status = status.clone();
        match status {
            EngineStatus::Loading => {
                self.phase = AppPhase::Loading;
                self.text = "loading".to_string();
                self.collapse_at = None;
            }
            EngineStatus::Ready => {
                self.phase = AppPhase::Done;
                self.text = "NPU ready".to_string();
                self.schedule_collapse();
            }
            EngineStatus::ModelsMissing => {
                self.phase = AppPhase::Error;
                self.text = "models missing".to_string();
                self.schedule_collapse();
            }
            EngineStatus::Error(msg) => {
                self.phase = AppPhase::Error;
                self.text = msg;
                self.schedule_collapse();
            }
        }
    }

    pub fn start_recording(&mut self) {
        self.phase = AppPhase::Recording;
        self.text.clear();
        self.collapse_at = None;
    }

    pub fn start_transcribing(&mut self) {
        self.phase = AppPhase::Transcribing;
        self.text.clear();
        self.collapse_at = None;
    }

    pub fn finish_done(&mut self, text: String) {
        self.phase = AppPhase::Done;
        self.text = text;
        self.schedule_collapse();
    }

    pub fn finish_error(&mut self, msg: String) {
        self.phase = AppPhase::Error;
        self.text = msg;
        self.schedule_collapse();
    }

    pub fn shrink_to_idle(&mut self) {
        self.phase = AppPhase::Idle;
        self.text.clear();
        self.collapse_at = None;
    }

    fn schedule_collapse(&mut self) {
        self.collapse_at = Some(Instant::now() + Duration::from_millis(500));
    }

    pub fn paint(&self, ui: &mut egui::Ui, rect: egui::Rect, hover_t: f32, morph_t: f32) {
        let painter = ui.painter_at(rect);
        let available = rect.shrink(WINDOW_PAD);
        let shape_t = smoothstep(morph_t);
        let width_t = ease_out_cubic(morph_t);
        let hover_push = hover_t * (1.0 - shape_t);
        let size = egui::vec2(
            lerp(IDLE_W, ACTIVE_W, width_t) + 4.0 * hover_push,
            lerp(IDLE_H, ACTIVE_H, shape_t) + 2.0 * hover_push,
        );
        let rect = egui::Rect::from_center_size(available.center(), size);
        let size = rect.size();
        let content_t = smoothstep(remap(morph_t, 0.18, 0.92));
        let idle_t = 1.0 - smoothstep(remap(morph_t, 0.0, 0.45));
        let bg = mix_color(
            egui::Color32::from_rgba_premultiplied(12, 12, 12, 225),
            egui::Color32::from_rgba_premultiplied(7, 7, 8, 238),
            shape_t,
        );
        let radius = size.y * 0.5;
        painter.rect(
            rect,
            radius,
            bg,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(
                    255,
                    255,
                    255,
                    lerp(70.0, 106.0, shape_t) as u8,
                ),
            ),
            egui::StrokeKind::Inside,
        );
        painter.line_segment(
            [
                egui::pos2(rect.left() + radius * 0.75, rect.top() + 1.0),
                egui::pos2(rect.right() - radius * 0.75, rect.top() + 1.0),
            ],
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(
                    255,
                    255,
                    255,
                    lerp(18.0, 34.0, shape_t) as u8,
                ),
            ),
        );

        let center = rect.center();
        if idle_t > 0.01 {
            painter.circle_filled(
                center,
                3.5,
                with_alpha(egui::Color32::from_gray(78), idle_t),
            );
        }
        if self.phase == AppPhase::Idle || content_t <= 0.01 {
            return;
        }

        match self.phase {
            AppPhase::Recording => {
                paint_audio_bars(&painter, rect, self.audio_level, content_t);
                paint_status_dot(
                    &painter,
                    egui::pos2(lerp(center.x, rect.left() + 18.0, content_t), center.y),
                    egui::Color32::from_rgb(255, 59, 48),
                    content_t,
                );
            }
            AppPhase::Transcribing => paint_spinner(&painter, rect),
            AppPhase::Done => paint_done(&painter, rect, &self.text),
            AppPhase::Loading => paint_loading(&painter, rect),
            AppPhase::Error => {
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    format!("! {}", self.text.chars().take(22).collect::<String>()),
                    egui::FontId::proportional(8.0),
                    egui::Color32::from_rgb(255, 159, 10),
                );
            }
            AppPhase::Idle => {}
        }
    }
}

fn paint_audio_bars(painter: &egui::Painter, rect: egui::Rect, level: f32, progress: f32) {
    let center = rect.center();
    let visible = ((BAR_COUNT as f32) * (progress * 1.5).min(1.0)).max(1.0) as usize;
    let total_w = visible as f32 * BAR_W + (visible.saturating_sub(1)) as f32 * BAR_GAP;
    let x0 = center.x - total_w * 0.5;
    let t = ui_time();

    for i in 0..visible {
        let sensitivity = 0.8 + 0.4 * (i as f32 * 1.3).sin();
        let val = (level * sensitivity * 70.0).clamp(0.0, 1.0);
        let h = BAR_MIN_H
            + (BAR_MAX_H - BAR_MIN_H) * val
            + (t * 8.0 + i as f64 * 0.72).sin() as f32 * 0.7;
        let dist = (i as f32 - visible as f32 * 0.5).abs() / (visible as f32 * 0.5).max(1.0);
        let intensity = 1.0 - dist * 0.35;
        let x = x0 + i as f32 * (BAR_W + BAR_GAP);
        let y1 = center.y - h * 0.5;
        let y2 = center.y + h * 0.5;
        let color = egui::Color32::from_rgb(
            (255.0 * intensity) as u8,
            (59.0 * intensity * 0.6) as u8,
            (48.0 * intensity * 0.4) as u8,
        )
        .gamma_multiply(progress);
        painter.rect_filled(
            egui::Rect::from_min_max(egui::pos2(x, y1), egui::pos2(x + BAR_W, y2)),
            1.0,
            color,
        );
    }
}

fn paint_spinner(painter: &egui::Painter, rect: egui::Rect) {
    let center = rect.center();
    let t = ui_time() as f32;
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * std::f32::consts::TAU - t * 3.0;
        let sx = center.x + 10.0 * angle.cos() * 0.6;
        let sy = center.y + 3.0 * angle.sin();
        let a = ((t * 5.0 + i as f32).sin() + 1.0) * 0.5;
        let gray = (60.0 + 160.0 * a) as u8;
        painter.circle_filled(egui::pos2(sx, sy), 1.8, egui::Color32::from_gray(gray));
    }
}

fn paint_done(painter: &egui::Painter, rect: egui::Rect, text: &str) {
    let center = rect.center();
    paint_status_dot(
        painter,
        egui::pos2(rect.left() + 18.0, center.y),
        egui::Color32::from_rgb(48, 209, 88),
        1.0,
    );
    let short = if text.chars().count() > 16 {
        text.chars().take(16).collect::<String>() + "..."
    } else {
        text.to_string()
    };
    painter.text(
        egui::pos2(center.x + 8.0, center.y),
        egui::Align2::CENTER_CENTER,
        if short.is_empty() { "ready" } else { &short },
        egui::FontId::proportional(9.0),
        egui::Color32::WHITE,
    );
}

fn paint_status_dot(painter: &egui::Painter, pos: egui::Pos2, color: egui::Color32, alpha: f32) {
    painter.circle_filled(pos, 5.0, color.gamma_multiply(0.20 * alpha));
    painter.circle_filled(pos, 3.5, color.gamma_multiply(alpha));
    painter.circle_filled(
        egui::pos2(pos.x - 1.0, pos.y - 1.0),
        1.1,
        egui::Color32::from_rgba_premultiplied(255, 255, 255, (110.0 * alpha) as u8),
    );
}

fn paint_loading(painter: &egui::Painter, rect: egui::Rect) {
    let center = rect.center();
    let t = ui_time() as f32;
    for i in 0..3 {
        let phase = (t * 3.0 + i as f32 * 1.2).sin();
        let y_off = phase * 4.0;
        let gray = (100.0 + 100.0 * (phase + 1.0) * 0.5) as u8;
        painter.circle_filled(
            egui::pos2(center.x - 16.0 + i as f32 * 16.0, center.y + y_off),
            2.5,
            egui::Color32::from_gray(gray),
        );
    }
    painter.text(
        egui::pos2(center.x + 16.0, center.y),
        egui::Align2::LEFT_CENTER,
        "loading...",
        egui::FontId::proportional(8.0),
        egui::Color32::from_gray(136),
    );
}

fn ui_time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

fn remap(value: f32, from: f32, to: f32) -> f32 {
    ((value - from) / (to - from)).clamp(0.0, 1.0)
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = 1.0 - t.clamp(0.0, 1.0);
    1.0 - t * t * t
}

fn with_alpha(color: egui::Color32, alpha: f32) -> egui::Color32 {
    color.gamma_multiply(alpha.clamp(0.0, 1.0))
}

fn mix_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    egui::Color32::from_rgba_premultiplied(
        lerp(a.r() as f32, b.r() as f32, t) as u8,
        lerp(a.g() as f32, b.g() as f32, t) as u8,
        lerp(a.b() as f32, b.b() as f32, t) as u8,
        lerp(a.a() as f32, b.a() as f32, t) as u8,
    )
}

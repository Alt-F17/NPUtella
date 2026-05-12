#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod code_context;
mod config;
mod context;
mod dictionary_manager;
mod dictionary_store;
mod hotkey;
mod logger;
mod mel;
mod phonetic_dictionary;
mod pipeline;
mod single_instance;
mod system;
mod tray;
mod ui;
mod whisper;

use anyhow::{Context, Result};
use audio::AudioRecorder;
use code_context::CodeContext;
use config::{AppConfig, Language};
use crossbeam_channel::{unbounded, Receiver, Sender};
use dictionary_manager::DictionaryManager;
use dictionary_store::DictionaryStore;
use eframe::{egui, App, NativeOptions, Renderer};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use ui::{AppEvent, AppPhase, OverlayState, SCREEN_BOTTOM_OFFSET};
use whisper::WhisperEngine;

const HOLD_MIN_MS: u64 = 300;

fn app_root() -> Result<PathBuf> {
    let mut starts = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    let exe = std::env::current_exe().context("current_exe failed")?;
    if let Some(parent) = exe.parent() {
        starts.push(parent.to_path_buf());
    }

    for start in starts {
        for dir in start.ancestors() {
            if is_project_root(dir) {
                return Ok(dir.to_path_buf());
            }
        }
    }

    Ok(exe
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(".")))
}

fn runtime_onnx_dir(root: &Path) -> PathBuf {
    root.join("venv-arm64")
        .join("Lib")
        .join("site-packages")
        .join("onnxruntime")
        .join("capi")
}

fn is_project_root(dir: &Path) -> bool {
    dir.join("whisper-base-local").is_dir()
        && dir
            .join("models")
            .join("whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core")
            .join("encoder.onnx")
            .is_file()
        && dir
            .join("models")
            .join("whisper_base-precompiled_qnn_onnx-float-qualcomm_snapdragon_x_plus_8_core")
            .join("decoder.onnx")
            .is_file()
}

struct NputellaApp {
    tx: Sender<AppEvent>,
    rx: Receiver<AppEvent>,
    engine: Arc<Mutex<Option<Arc<Mutex<WhisperEngine>>>>>,
    config: Arc<AppConfig>,
    code_context: Arc<CodeContext>,
    dictionary: Arc<DictionaryStore>,
    dictionary_manager: DictionaryManager,
    language: Language,
    show_dictionary_manager: bool,
    recorder: AudioRecorder,
    state: OverlayState,
    recording_since: Option<Instant>,
    current_pos: egui::Pos2,
    screen_size: (f32, f32),
    last_frame: Instant,
    hover_t: f32,
    morph_t: f32,
}

impl NputellaApp {
    fn new(
        tx: Sender<AppEvent>,
        rx: Receiver<AppEvent>,
        engine: Arc<Mutex<Option<Arc<Mutex<WhisperEngine>>>>>,
        config: Arc<AppConfig>,
        code_context: Arc<CodeContext>,
        dictionary: Arc<DictionaryStore>,
    ) -> Self {
        let language = config.language;
        Self {
            tx,
            rx,
            engine,
            config,
            code_context,
            dictionary,
            dictionary_manager: DictionaryManager::new(),
            language,
            show_dictionary_manager: false,
            recorder: AudioRecorder::new(),
            state: OverlayState::loading(),
            recording_since: None,
            current_pos: egui::pos2(0.0, 0.0),
            screen_size: (1920.0, 1080.0),
            last_frame: Instant::now(),
            hover_t: 0.0,
            morph_t: 1.0,
        }
    }

    fn process_events(&mut self, ctx: &egui::Context) {
        self.state.tick();
        while let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::EngineStatus(status) => {
                    self.state.set_engine_status(status);
                }
                AppEvent::HotkeyDown => self.start_recording(),
                AppEvent::HotkeyUp => self.stop_recording(),
                AppEvent::AudioLevel(level) => self.state.audio_level = level,
                AppEvent::TranscriptionDone(text) => {
                    self.state.finish_done(text);
                    self.recording_since = None;
                }
                AppEvent::TranscriptionError(msg) => {
                    self.state.finish_error(msg);
                    self.recording_since = None;
                }
                AppEvent::OpenDictionaryManager => {
                    self.show_dictionary_manager = true;
                    self.dictionary_manager.ensure_loaded(&self.dictionary);
                }
                AppEvent::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn start_recording(&mut self) {
        if self.state.phase == AppPhase::Recording || self.state.phase == AppPhase::Transcribing {
            return;
        }
        let ready = self
            .engine
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned())
            .is_some();
        if !ready {
            self.state.finish_error("model not ready".to_string());
            return;
        }
        let level_tx = self.tx.clone();
        match self.recorder.start(level_tx) {
            Ok(()) => {
                self.recording_since = Some(Instant::now());
                self.state.start_recording();
            }
            Err(err) => {
                self.state.finish_error(err.to_string());
            }
        }
    }

    fn stop_recording(&mut self) {
        if self.state.phase != AppPhase::Recording {
            return;
        }
        let elapsed_ms = self
            .recording_since
            .map(|t| t.elapsed().as_millis() as u64)
            .unwrap_or_default();
        let capture = self.recorder.stop();
        self.recording_since = None;
        if elapsed_ms < HOLD_MIN_MS {
            self.state.shrink_to_idle();
            return;
        }

        self.state.start_transcribing();
        let tx = self.tx.clone();
        let engine = self.engine.clone();
        let config = self.config.clone();
        let code_context = self.code_context.clone();
        let dictionary = self.dictionary.clone();
        let language = self.language;
        thread::spawn(move || {
            let engine = match engine.lock().ok().and_then(|guard| guard.as_ref().cloned()) {
                Some(engine) => engine,
                None => {
                    let _ = tx.send(AppEvent::TranscriptionError("model not ready".to_string()));
                    return;
                }
            };

            let result = engine
                .lock()
                .map_err(|_| anyhow::anyhow!("engine lock poisoned"))
                .and_then(|mut engine| engine.transcribe(&capture, language));

            match result {
                Ok(text) if !text.trim().is_empty() => {
                    let corrected =
                        phonetic_dictionary::correct_text(&text, &dictionary.snapshot());
                    let insert = if config.local_adaptation_enabled {
                        let target = context::TargetContext::detect();
                        pipeline::process_transcript(
                            &corrected,
                            &config,
                            &target,
                            &code_context,
                            &dictionary,
                        )
                    } else {
                        pipeline::InsertPlan {
                            text: corrected.trim().to_string(),
                            press_enter: false,
                            skip_paste: false,
                        }
                    };
                    logger::line(format!(
                        "transcription complete: raw_chars={} final_chars={} press_enter={} skip_paste={}",
                        text.chars().count(),
                        insert.text.chars().count(),
                        insert.press_enter,
                        insert.skip_paste
                    ));
                    if !insert.skip_paste {
                        let _ = system::insert_text(&insert);
                    }
                    let _ = tx.send(AppEvent::TranscriptionDone(insert.text));
                }
                Ok(_) => {
                    logger::line("transcription produced no text");
                    let _ = tx.send(AppEvent::TranscriptionError("nothing heard".to_string()));
                }
                Err(err) => {
                    logger::line(format!("transcription failed: {err:?}"));
                    let _ = tx.send(AppEvent::TranscriptionError(err.to_string()));
                }
            }
        });
    }

    fn update_window_geometry(&mut self, ctx: &egui::Context, target_size: egui::Vec2) {
        let actual_size = egui::vec2(
            (target_size.x + ui::WINDOW_PAD * 2.0).round(),
            (target_size.y + ui::WINDOW_PAD * 2.0).round(),
        );
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(actual_size));
        let screen_rect = ctx.input(|i| i.viewport().monitor_size);
        if let Some(screen) = screen_rect {
            self.screen_size = (screen.x, screen.y);
        }
        let x = (self.screen_size.0 - actual_size.x) * 0.5;
        let y = self.screen_size.1 - actual_size.y - SCREEN_BOTTOM_OFFSET;
        let pos = egui::pos2(x.max(0.0), y.max(0.0));
        if (pos.x - self.current_pos.x).abs() > 0.5 || (pos.y - self.current_pos.y).abs() > 0.5 {
            self.current_pos = pos;
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
        }
    }
}

impl App for NputellaApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_transparent_egui_style(ctx);
        let now = Instant::now();
        let dt = now
            .duration_since(self.last_frame)
            .as_secs_f32()
            .clamp(1.0 / 240.0, 1.0 / 30.0);
        self.last_frame = now;

        self.process_events(ctx);

        let bounds =
            egui::Rect::from_min_size(egui::Pos2::ZERO, ctx.input(|i| i.content_rect().size()));
        let pill_rect = OverlayState::pill_rect(bounds, self.hover_t, self.morph_t);
        let language_rect = OverlayState::language_rect(bounds, self.hover_t, self.morph_t);
        let dictionary_rect = OverlayState::dictionary_rect(bounds, self.hover_t, self.morph_t);
        let pointer_hovered = ctx.input(|i| {
            i.pointer
                .hover_pos()
                .map(|pos| {
                    pill_rect.contains(pos)
                        || language_rect.contains(pos)
                        || dictionary_rect.contains(pos)
                })
                .unwrap_or(false)
        });
        self.state.hovered = pointer_hovered;
        let hover_target = if pointer_hovered && self.state.phase == AppPhase::Idle {
            1.0
        } else {
            0.0
        };
        self.hover_t = approach(self.hover_t, hover_target, 13.0, dt);

        let morph_target = if self.state.phase == AppPhase::Idle {
            0.0
        } else {
            1.0
        };
        let morph_speed = if morph_target > self.morph_t {
            20.0
        } else {
            24.0
        };
        self.morph_t = approach(self.morph_t, morph_target, morph_speed, dt);

        let target_size = egui::vec2(
            ui::SIDE_PILL_W + ui::LANGUAGE_GAP + ui::ACTIVE_W + ui::LANGUAGE_GAP + ui::SIDE_PILL_W,
            ui::ACTIVE_H,
        );
        self.update_window_geometry(ctx, target_size);
        if (self.hover_t - hover_target).abs() > 0.001
            || (self.morph_t - morph_target).abs() > 0.001
        {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.visuals_mut().panel_fill = egui::Color32::TRANSPARENT;
        ui.visuals_mut().window_fill = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        let rect = ui.max_rect();
        let pill_rect = OverlayState::pill_rect(rect, self.hover_t, self.morph_t);
        let language_rect = OverlayState::language_rect(rect, self.hover_t, self.morph_t);
        let dictionary_rect = OverlayState::dictionary_rect(rect, self.hover_t, self.morph_t);
        let dictionary_response = ui.allocate_rect(dictionary_rect, egui::Sense::click());
        let language_response = ui.allocate_rect(language_rect, egui::Sense::click());
        let pill_response = ui.allocate_rect(pill_rect, egui::Sense::click());
        if dictionary_response.clicked() && self.state.phase == AppPhase::Idle {
            self.show_dictionary_manager = true;
            self.dictionary_manager.ensure_loaded(&self.dictionary);
            logger::line("dictionary manager opened from overlay");
        } else if language_response.clicked() && self.state.phase == AppPhase::Idle {
            self.language = self.language.cycle();
            logger::line(format!("language toggled to {:?}", self.language));
        } else if pill_response.clicked() {
            if self.state.phase == AppPhase::Recording {
                self.stop_recording();
            } else if self.state.phase == AppPhase::Idle {
                self.start_recording();
            }
        }
        self.state.hovered =
            pill_response.hovered() || language_response.hovered() || dictionary_response.hovered();
        self.state
            .paint(ui, rect, self.hover_t, self.morph_t, self.language);

        if self.show_dictionary_manager {
            self.dictionary_manager.show(
                ui.ctx(),
                &mut self.show_dictionary_manager,
                self.dictionary.clone(),
            );
        }
    }
}

fn apply_transparent_egui_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.visuals.panel_fill = egui::Color32::TRANSPARENT;
    style.visuals.window_fill = egui::Color32::TRANSPARENT;
    style.visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
    style.visuals.faint_bg_color = egui::Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
    style.visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::TRANSPARENT;
    ctx.set_global_style(style);
}

fn approach(current: f32, target: f32, speed: f32, dt: f32) -> f32 {
    let t = 1.0 - (-speed * dt).exp();
    let next = current + (target - current) * t;
    if (target - next).abs() < 0.001 {
        target
    } else {
        next
    }
}

fn main() -> Result<()> {
    let root = app_root()?;
    logger::init(&root);
    logger::line(format!("starting nputella from root {}", root.display()));
    let _single_instance = match single_instance::SingleInstance::acquire() {
        Ok(instance) => {
            logger::line("single instance lock acquired");
            instance
        }
        Err(err) => {
            logger::line(format!("startup aborted: {err}"));
            return Ok(());
        }
    };
    let _ = std::env::set_current_dir(&root);
    let onnx_dir = runtime_onnx_dir(&root);
    if onnx_dir.exists() {
        logger::line(format!(
            "found Python ONNX Runtime directory {}; not prepending to PATH",
            onnx_dir.display()
        ));
    } else {
        logger::line(format!(
            "Python ONNX Runtime directory not found at {}",
            onnx_dir.display()
        ));
    }

    let (tx, rx) = unbounded::<AppEvent>();
    let engine_slot: Arc<Mutex<Option<Arc<Mutex<WhisperEngine>>>>> = Arc::new(Mutex::new(None));
    let config = Arc::new(AppConfig::load(&root));
    logger::line(format!(
        "config: language={:?} adaptation={} smart={} code={} math={} llm_enabled={} llm_model={}",
        config.language,
        config.local_adaptation_enabled,
        config.smart_formatting,
        config.code_formatting,
        config.math_formatting,
        config.local_llm_enabled,
        config.local_llm_model
    ));
    let code_context = Arc::new(CodeContext::load(&root));
    let dictionary = Arc::new(DictionaryStore::load(&root, config.dictionary.clone()));

    {
        let tx = tx.clone();
        let engine_slot = engine_slot.clone();
        let root = root.clone();
        thread::spawn(move || {
            let _ = tx.send(AppEvent::EngineStatus(ui::EngineStatus::Loading));
            logger::line("engine load thread started");
            let load_result = std::panic::catch_unwind(|| WhisperEngine::load(&root));
            match load_result {
                Ok(Ok(engine)) => {
                    logger::line("engine loaded successfully");
                    let engine = Arc::new(Mutex::new(engine));
                    if let Ok(mut guard) = engine_slot.lock() {
                        *guard = Some(engine);
                    }
                    let _ = tx.send(AppEvent::EngineStatus(ui::EngineStatus::Ready));
                }
                Ok(Err(err)) => {
                    logger::line(format!("engine load failed: {err:?}"));
                    let status = if err.to_string().contains("models missing") {
                        ui::EngineStatus::ModelsMissing
                    } else {
                        ui::EngineStatus::Error(err.to_string())
                    };
                    let _ = tx.send(AppEvent::EngineStatus(status));
                }
                Err(payload) => {
                    let msg = payload
                        .downcast_ref::<String>()
                        .cloned()
                        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                        .unwrap_or_else(|| "model runtime panic".to_string());
                    logger::line(format!("engine load panicked: {msg}"));
                    let _ = tx.send(AppEvent::EngineStatus(ui::EngineStatus::Error(msg)));
                }
            }
        });
    }

    hotkey::spawn_hotkey_hook(tx.clone());
    tray::spawn_tray(tx.clone());

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_resizable(false)
            .with_inner_size([
                ui::SIDE_PILL_W
                    + ui::LANGUAGE_GAP
                    + ui::ACTIVE_W
                    + ui::LANGUAGE_GAP
                    + ui::SIDE_PILL_W
                    + ui::WINDOW_PAD * 2.0,
                ui::ACTIVE_H + ui::WINDOW_PAD * 2.0,
            ])
            .with_title("nputella"),
        renderer: Renderer::Glow,
        ..Default::default()
    };
    logger::line(format!(
        "native window: transparent=true decorations=false always_on_top=true taskbar=false renderer={}",
        native_options.renderer
    ));

    let tx_for_app = tx.clone();
    eframe::run_native(
        "nputella",
        native_options,
        Box::new(move |cc| {
            apply_transparent_egui_style(&cc.egui_ctx);
            Ok(Box::new(NputellaApp::new(
                tx_for_app.clone(),
                rx,
                engine_slot,
                config,
                code_context,
                dictionary,
            )))
        }),
    )
    .map_err(|err| anyhow::anyhow!("running egui app: {err}"))?;

    Ok(())
}

use crate::logger;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppKind {
    Ide,
    Terminal,
    Browser,
    Chat,
    Generic,
}

#[derive(Clone, Debug)]
pub struct TargetContext {
    pub title: String,
    pub kind: AppKind,
}

impl TargetContext {
    pub fn detect() -> Self {
        let title = foreground_window_title().unwrap_or_default();
        let kind = classify_title(&title);
        logger::line(format!("target context: kind={kind:?} title={title:?}"));
        Self { title, kind }
    }

    pub fn wants_code_formatting(&self) -> bool {
        matches!(self.kind, AppKind::Ide | AppKind::Terminal | AppKind::Chat)
    }

    pub fn wants_file_tags(&self) -> bool {
        matches!(self.kind, AppKind::Ide | AppKind::Chat)
    }

    pub fn wants_latex_math(&self) -> bool {
        matches!(self.kind, AppKind::Ide | AppKind::Chat)
            || self.title.to_ascii_lowercase().contains("markdown")
    }
}

fn classify_title(title: &str) -> AppKind {
    let lower = title.to_ascii_lowercase();
    if contains_any(
        &lower,
        &[
            "cursor",
            "visual studio code",
            "vs code",
            "windsurf",
            "jetbrains",
            "rustrover",
            "pycharm",
            "webstorm",
            "intellij",
        ],
    ) {
        AppKind::Ide
    } else if contains_any(
        &lower,
        &[
            "powershell",
            "command prompt",
            "windows terminal",
            "terminal",
        ],
    ) {
        AppKind::Terminal
    } else if contains_any(&lower, &["claude", "chatgpt", "copilot", "perplexity"]) {
        AppKind::Chat
    } else if contains_any(&lower, &["chrome", "edge", "firefox", "brave"]) {
        AppKind::Browser
    } else {
        AppKind::Generic
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn foreground_window_title() -> Option<String> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return None;
        }
        let mut buf = vec![0u16; len as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buf);
        if copied <= 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..copied as usize]))
    }
}

use crate::logger;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct CodeContext {
    pub(crate) files: Vec<String>,
    pub(crate) symbols: Vec<String>,
}

impl CodeContext {
    pub fn load(root: &Path) -> Self {
        let mut files = Vec::new();
        let mut symbols = HashSet::new();
        scan_dir(root, root, 0, &mut files, &mut symbols);
        let mut symbols: Vec<String> = symbols.into_iter().collect();
        files.sort_by_key(|f| (f.matches('\\').count(), f.len()));
        symbols.sort_by_key(|s| s.len());
        logger::line(format!(
            "code context loaded: {} files {} symbols",
            files.len(),
            symbols.len()
        ));
        Self { files, symbols }
    }

    pub fn tag_files(&self, text: &str) -> String {
        let mut out = text.to_string();
        for file in self.files.iter().take(300) {
            let spoken = spoken_file(file);
            for prefix in ["at ", "tag ", "open ", "file "] {
                let phrase = format!("{prefix}{spoken}");
                if contains_case_insensitive(&out, &phrase) {
                    out = replace_case_insensitive(&out, &phrase, &format!("@{file}"));
                }
            }
        }
        out
    }

    pub fn tag_symbols(&self, text: &str) -> String {
        let mut out = text.to_string();
        for symbol in self.symbols.iter().rev().take(500) {
            if symbol.len() < 4 || symbol.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let spoken = spoken_identifier(symbol);
            if contains_case_insensitive(&out, &spoken) {
                out = replace_word_like(&out, &spoken, &format!("`{symbol}`"));
            }
        }
        out
    }
}

fn scan_dir(
    root: &Path,
    dir: &Path,
    depth: usize,
    files: &mut Vec<String>,
    symbols: &mut HashSet<String>,
) {
    if depth > 5 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip(&name) {
            continue;
        }
        if path.is_dir() {
            scan_dir(root, &path, depth + 1, files, symbols);
        } else if is_code_file(&path) {
            if let Ok(rel) = path.strip_prefix(root) {
                files.push(rel.to_string_lossy().replace('/', "\\"));
            }
            if let Ok(text) = fs::read_to_string(&path) {
                extract_symbols(&text, symbols);
            }
        }
    }
}

fn should_skip(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "venv" | "venv-arm64" | "__pycache__" | "models" | "whisper-base-local"
    ) || name.starts_with('.')
}

fn is_code_file(path: &PathBuf) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "py" | "ts" | "tsx" | "js" | "jsx" | "json" | "toml" | "md")
    )
}

fn extract_symbols(text: &str, symbols: &mut HashSet<String>) {
    for line in text.lines() {
        let trimmed = line.trim_start();
        for prefix in [
            "fn ",
            "struct ",
            "enum ",
            "trait ",
            "mod ",
            "class ",
            "def ",
            "function ",
            "const ",
            "let ",
        ] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(symbol) = rest
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .next()
                    .filter(|s| !s.is_empty())
                {
                    symbols.insert(symbol.to_string());
                }
            }
        }
    }
}

fn spoken_file(path: &str) -> String {
    path.replace('\\', " slash ")
        .replace('/', " slash ")
        .replace('.', " dot ")
        .replace('_', " underscore ")
        .replace('-', " dash ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn spoken_identifier(value: &str) -> String {
    let mut words = Vec::new();
    let mut current = String::new();
    for ch in value.chars() {
        if ch == '_' || ch == '-' {
            if !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
        } else if ch.is_ascii_uppercase() && !current.is_empty() {
            words.push(current.to_ascii_lowercase());
            current.clear();
            current.push(ch.to_ascii_lowercase());
        } else {
            current.push(ch.to_ascii_lowercase());
        }
    }
    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
    }
    words.join(" ")
}

fn contains_case_insensitive(text: &str, needle: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn replace_case_insensitive(text: &str, needle: &str, replacement: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let needle_lower = needle.to_ascii_lowercase();
    let mut out = String::new();
    let mut start = 0usize;
    let mut cursor = 0usize;
    while let Some(pos) = lower[cursor..].find(&needle_lower) {
        let idx = cursor + pos;
        out.push_str(&text[start..idx]);
        out.push_str(replacement);
        cursor = idx + needle_lower.len();
        start = cursor;
    }
    out.push_str(&text[start..]);
    out
}

fn replace_word_like(text: &str, phrase: &str, replacement: &str) -> String {
    let mut out = String::new();
    let lower = text.to_ascii_lowercase();
    let phrase = phrase.to_ascii_lowercase();
    let mut start = 0usize;
    let mut cursor = 0usize;
    while let Some(pos) = lower[cursor..].find(&phrase) {
        let idx = cursor + pos;
        let end = idx + phrase.len();
        let left_ok = idx == 0
            || !text[..idx]
                .chars()
                .last()
                .unwrap_or(' ')
                .is_ascii_alphanumeric();
        let right_ok = end >= text.len()
            || !text[end..]
                .chars()
                .next()
                .unwrap_or(' ')
                .is_ascii_alphanumeric();
        if left_ok && right_ok {
            out.push_str(&text[start..idx]);
            out.push_str(replacement);
            cursor = end;
            start = end;
        } else {
            cursor = end;
        }
    }
    out.push_str(&text[start..]);
    out
}

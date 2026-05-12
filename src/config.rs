use crate::logger;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Auto,
    English,
    French,
}

impl Language {
    pub fn cycle(self) -> Self {
        match self {
            Self::Auto => Self::French,
            Self::French => Self::English,
            Self::English => Self::Auto,
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Auto => "bi",
            Self::French => "fr",
            Self::English => "en",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DictionaryEntry {
    pub from: String,
    pub to: String,
    pub aliases: Vec<String>,
    pub phonetic: bool,
    pub priority: DictionaryPriority,
    pub language: DictionaryLanguage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DictionaryPriority {
    Normal,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DictionaryLanguage {
    Any,
    English,
    French,
}

impl DictionaryEntry {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            aliases: Vec::new(),
            phonetic: false,
            priority: DictionaryPriority::Normal,
            language: DictionaryLanguage::Any,
        }
    }

    pub fn target(&self) -> &str {
        if self.to.trim().is_empty() {
            self.from.trim()
        } else {
            self.to.trim()
        }
    }

    pub fn spoken_aliases(&self) -> Vec<String> {
        let mut values = Vec::new();
        if !self.from.trim().is_empty() && !self.to.trim().is_empty() {
            values.push(self.from.trim().to_string());
        }
        values.extend(
            self.aliases
                .iter()
                .map(|alias| alias.trim())
                .filter(|alias| !alias.is_empty())
                .map(ToString::to_string),
        );
        let target = self.target();
        if !target.is_empty() {
            values.push(target.to_string());
        }
        values
    }
}

#[derive(Clone, Debug)]
pub struct Snippet {
    pub trigger: String,
    pub expansion: String,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub language: Language,
    pub local_adaptation_enabled: bool,
    pub dictionary: Vec<DictionaryEntry>,
    pub snippets: Vec<Snippet>,
    pub smart_formatting: bool,
    pub code_formatting: bool,
    pub math_formatting: bool,
    pub file_tagging: bool,
    pub symbol_tagging: bool,
    pub keep_transcript_on_clipboard: bool,
    pub local_llm_enabled: bool,
    pub local_llm_model: String,
    pub local_llm_endpoint: String,
}

enum Section {
    Root,
    Dictionary(DictionaryEntry),
    Snippet(Snippet),
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::Auto,
            local_adaptation_enabled: false,
            dictionary: default_dictionary(),
            snippets: default_snippets(),
            smart_formatting: true,
            code_formatting: true,
            math_formatting: true,
            file_tagging: true,
            symbol_tagging: true,
            keep_transcript_on_clipboard: true,
            local_llm_enabled: false,
            local_llm_model: "phi-3.5-mini".to_string(),
            local_llm_endpoint: "http://127.0.0.1:5273/v1/chat/completions".to_string(),
        }
    }
}

impl AppConfig {
    pub fn load(root: &Path) -> Self {
        let mut config = Self::default();
        let paths = config_paths(root);
        for path in &paths {
            if path.is_file() {
                match fs::read_to_string(path) {
                    Ok(text) => {
                        apply_config_text(&mut config, &text);
                        logger::line(format!("loaded config from {}", path.display()));
                        return config;
                    }
                    Err(err) => {
                        logger::line(format!("could not read config {}: {err}", path.display()))
                    }
                }
            }
        }
        if let Some(path) = paths.first() {
            logger::line(format!(
                "using default config; no file at {}",
                path.display()
            ));
        }
        config
    }
}

fn config_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(root.join("nputella.toml"));
    if let Ok(appdata) = std::env::var("APPDATA") {
        paths.push(PathBuf::from(appdata).join("NPUtella").join("config.toml"));
    }
    paths
}

fn apply_config_text(config: &mut AppConfig, text: &str) {
    let mut section = Section::Root;
    for raw_line in text.lines() {
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[dictionary]]" {
            flush_section(config, &mut section);
            section = Section::Dictionary(DictionaryEntry::new("", ""));
            continue;
        }
        if line == "[[snippet]]" || line == "[[snippets]]" {
            flush_section(config, &mut section);
            section = Section::Snippet(Snippet {
                trigger: String::new(),
                expansion: String::new(),
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = unquote(value.trim());
        match &mut section {
            Section::Root => match key {
                "language" => config.language = parse_language(&value),
                "local_adaptation_enabled" => {
                    config.local_adaptation_enabled =
                        parse_bool(&value, config.local_adaptation_enabled)
                }
                "smart_formatting" => {
                    config.smart_formatting = parse_bool(&value, config.smart_formatting)
                }
                "code_formatting" => {
                    config.code_formatting = parse_bool(&value, config.code_formatting)
                }
                "math_formatting" => {
                    config.math_formatting = parse_bool(&value, config.math_formatting)
                }
                "file_tagging" => config.file_tagging = parse_bool(&value, config.file_tagging),
                "symbol_tagging" => {
                    config.symbol_tagging = parse_bool(&value, config.symbol_tagging)
                }
                "keep_transcript_on_clipboard" => {
                    config.keep_transcript_on_clipboard =
                        parse_bool(&value, config.keep_transcript_on_clipboard)
                }
                "local_llm_enabled" => {
                    config.local_llm_enabled = parse_bool(&value, config.local_llm_enabled)
                }
                "local_llm_model" => config.local_llm_model = value,
                "local_llm_endpoint" => config.local_llm_endpoint = value,
                _ => {}
            },
            Section::Dictionary(entry) => match key {
                "from" | "spoken" => entry.from = value,
                "to" | "written" => entry.to = value,
                "alias" | "aliases" => entry.aliases.extend(parse_list(&value)),
                "phonetic" => entry.phonetic = parse_bool(&value, entry.phonetic),
                "priority" => entry.priority = parse_priority(&value),
                "language" => entry.language = parse_dictionary_language(&value),
                _ => {}
            },
            Section::Snippet(snippet) => match key {
                "trigger" => snippet.trigger = value,
                "expansion" | "text" => snippet.expansion = value.replace("\\n", "\n"),
                _ => {}
            },
        }
    }
    flush_section(config, &mut section);
}

fn flush_section(config: &mut AppConfig, section: &mut Section) {
    match std::mem::replace(section, Section::Root) {
        Section::Dictionary(entry)
            if !entry.from.trim().is_empty() && !entry.to.trim().is_empty() =>
        {
            config.dictionary.push(entry);
        }
        Section::Snippet(snippet)
            if !snippet.trigger.trim().is_empty() && !snippet.expansion.trim().is_empty() =>
        {
            config.snippets.push(snippet);
        }
        _ => {}
    }
}

fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or(line)
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].replace("\\\"", "\"")
    } else {
        value.to_string()
    }
}

fn parse_language(value: &str) -> Language {
    match value.trim().to_ascii_lowercase().as_str() {
        "en" | "eng" | "english" => Language::English,
        "fr" | "fra" | "fre" | "french" | "francais" | "fran\u{00e7}ais" => Language::French,
        _ => Language::Auto,
    }
}

fn parse_bool(value: &str, default: bool) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => true,
        "false" | "no" | "off" | "0" => false,
        _ => default,
    }
}

fn parse_priority(value: &str) -> DictionaryPriority {
    match value.trim().to_ascii_lowercase().as_str() {
        "high" => DictionaryPriority::High,
        _ => DictionaryPriority::Normal,
    }
}

fn parse_dictionary_language(value: &str) -> DictionaryLanguage {
    match value.trim().to_ascii_lowercase().as_str() {
        "en" | "english" => DictionaryLanguage::English,
        "fr" | "french" | "francais" | "fran\u{00e7}ais" => DictionaryLanguage::French,
        _ => DictionaryLanguage::Any,
    }
}

fn parse_list(value: &str) -> Vec<String> {
    let value = value.trim();
    let inner = value
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or(value);
    inner
        .split(',')
        .map(|item| item.trim().trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

pub fn default_dictionary() -> Vec<DictionaryEntry> {
    vec![
        DictionaryEntry::new("n p u tella", "NPUtella"),
        DictionaryEntry::new("npu tella", "NPUtella"),
    ]
}

pub fn default_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            trigger: "today date".to_string(),
            expansion: "2026-05-12".to_string(),
        },
        Snippet {
            trigger: "code fence".to_string(),
            expansion: "```\n\n```".to_string(),
        },
    ]
}

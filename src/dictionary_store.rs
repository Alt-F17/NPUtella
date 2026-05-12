use crate::config::{DictionaryEntry, DictionaryLanguage, DictionaryPriority};
use crate::logger;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DictionaryStore {
    base: Arc<Vec<DictionaryEntry>>,
    learned: Arc<Mutex<Vec<DictionaryEntry>>>,
    path: PathBuf,
}

impl DictionaryStore {
    pub fn load(root: &Path, base: Vec<DictionaryEntry>) -> Self {
        let path = dictionary_path(root);
        let learned = if let Ok(text) = fs::read_to_string(&path) {
            logger::line(format!("loaded learned dictionary from {}", path.display()));
            parse_entries(&text)
        } else {
            logger::line(format!("no learned dictionary at {}", path.display()));
            Vec::new()
        };
        let mut learned = learned;
        normalize_entries(&mut learned);
        dedupe(&mut learned);
        Self {
            base: Arc::new(base),
            learned: Arc::new(Mutex::new(learned)),
            path,
        }
    }

    pub fn snapshot(&self) -> Vec<DictionaryEntry> {
        let mut entries = self.base.as_ref().clone();
        if let Ok(guard) = self.learned.lock() {
            entries.extend(guard.clone());
        }
        dedupe(&mut entries);
        entries
    }

    pub fn user_entries(&self) -> Vec<DictionaryEntry> {
        self.learned
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn replace_user_entries(&self, mut entries: Vec<DictionaryEntry>) -> bool {
        entries.retain(|entry| !entry.target().trim().is_empty());
        normalize_entries(&mut entries);
        dedupe(&mut entries);
        let mut guard = match self.learned.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        *guard = entries;
        self.persist_locked(&guard)
    }

    pub fn learn(&self, from: String, to: String) -> bool {
        let mut entry = DictionaryEntry::new(from, to);
        entry.phonetic = true;
        entry.priority = DictionaryPriority::High;
        self.add_entry(entry)
    }

    pub fn add_entry(&self, entry: DictionaryEntry) -> bool {
        let mut guard = match self.learned.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        if guard.iter().any(|item| same_entry(item, &entry)) {
            return false;
        }
        guard.push(entry);
        normalize_entries(&mut guard);
        dedupe(&mut guard);
        self.persist_locked(&guard)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn persist_locked(&self, entries: &[DictionaryEntry]) -> bool {
        if let Err(err) = save_entries(&self.path, entries) {
            logger::line(format!("failed to persist learned dictionary: {err}"));
            false
        } else {
            logger::line(format!(
                "persisted learned dictionary to {}",
                self.path.display()
            ));
            true
        }
    }
}

fn dictionary_path(root: &Path) -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata)
            .join("NPUtella")
            .join("dictionary.toml")
    } else {
        root.join("nputella_dictionary.toml")
    }
}

fn parse_entries(text: &str) -> Vec<DictionaryEntry> {
    let mut entries = Vec::new();
    let mut current = DictionaryEntry::new("", "");

    for raw_line in text.lines() {
        let line = raw_line.split('#').next().unwrap_or(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[dictionary]]" {
            push_current(&mut entries, &mut current);
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = unquote(value.trim());
        match key {
            "from" | "spoken" => current.from = value,
            "to" | "written" => current.to = value,
            "alias" | "aliases" => current.aliases.extend(parse_list(&value)),
            "phonetic" => current.phonetic = parse_bool(&value, current.phonetic),
            "priority" => current.priority = parse_priority(&value),
            "language" => current.language = parse_language(&value),
            _ => {}
        }
    }
    push_current(&mut entries, &mut current);
    entries
}

fn push_current(entries: &mut Vec<DictionaryEntry>, current: &mut DictionaryEntry) {
    let has_legacy = !current.from.trim().is_empty() && !current.to.trim().is_empty();
    let has_new = !current.to.trim().is_empty() && !current.aliases.is_empty();
    if has_legacy || has_new {
        entries.push(current.clone());
    }
    *current = DictionaryEntry::new("", "");
}

fn save_entries(path: &Path, entries: &[DictionaryEntry]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text = String::new();
    for entry in entries {
        if entry.target().is_empty() {
            continue;
        }
        text.push_str("[[dictionary]]\n");
        if !entry.from.trim().is_empty() && !entry.to.trim().is_empty() {
            text.push_str(&format!("from = \"{}\"\n", escape(&entry.from)));
        }
        text.push_str(&format!("written = \"{}\"\n", escape(entry.target())));
        if !entry.aliases.is_empty() {
            text.push_str("aliases = [");
            for (idx, alias) in entry.aliases.iter().enumerate() {
                if idx > 0 {
                    text.push_str(", ");
                }
                text.push_str(&format!("\"{}\"", escape(alias)));
            }
            text.push_str("]\n");
        }
        if entry.phonetic {
            text.push_str("phonetic = true\n");
        }
        if entry.priority == DictionaryPriority::High {
            text.push_str("priority = \"high\"\n");
        }
        match entry.language {
            DictionaryLanguage::English => text.push_str("language = \"en\"\n"),
            DictionaryLanguage::French => text.push_str("language = \"fr\"\n"),
            DictionaryLanguage::Any => {}
        }
        text.push('\n');
    }
    fs::write(path, text)
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        value.to_string()
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

fn parse_language(value: &str) -> DictionaryLanguage {
    match value.trim().to_ascii_lowercase().as_str() {
        "en" | "english" => DictionaryLanguage::English,
        "fr" | "french" | "francais" | "fran\u{00e7}ais" => DictionaryLanguage::French,
        _ => DictionaryLanguage::Any,
    }
}

fn dedupe(entries: &mut Vec<DictionaryEntry>) {
    let mut seen = Vec::<(String, String)>::new();
    entries.retain(|entry| {
        let key = (
            entry.from.to_ascii_lowercase(),
            entry.target().to_ascii_lowercase(),
        );
        if seen.iter().any(|item| item == &key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}

fn same_entry(a: &DictionaryEntry, b: &DictionaryEntry) -> bool {
    a.from.eq_ignore_ascii_case(&b.from) && a.target().eq_ignore_ascii_case(b.target())
}

fn normalize_entries(entries: &mut [DictionaryEntry]) {
    for entry in entries {
        entry.from = entry.from.trim().to_string();
        entry.to = entry.to.trim().to_string();
        entry.aliases = entry
            .aliases
            .iter()
            .map(|alias| alias.trim().to_string())
            .filter(|alias| !alias.is_empty())
            .collect();
    }
}

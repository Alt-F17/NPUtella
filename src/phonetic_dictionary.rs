use crate::config::{DictionaryEntry, DictionaryPriority};
use crate::logger;
use strsim::jaro_winkler;

#[derive(Clone, Debug)]
struct Candidate {
    start: usize,
    end: usize,
    original: String,
    replacement: String,
    score: f64,
    kind: &'static str,
}

#[derive(Clone, Debug)]
struct Token {
    text: String,
    start: usize,
    end: usize,
}

pub fn correct_text(text: &str, entries: &[DictionaryEntry]) -> String {
    let tokens = tokenize(text);
    if tokens.is_empty() || entries.is_empty() {
        return text.to_string();
    }

    let mut candidates = Vec::new();
    for start_idx in 0..tokens.len() {
        for end_idx in start_idx..(start_idx + 5).min(tokens.len()) {
            let start = tokens[start_idx].start;
            let end = tokens[end_idx].end;
            let span = &text[start..end];
            let span_norm = normalize(span);
            if span_norm.is_empty() {
                continue;
            }
            let span_compact = compact(&span_norm);
            let span_phone = phonetic_key(&span_norm);
            let span_word_count = span_norm.split_whitespace().count();
            for entry in entries {
                if entry.target().is_empty() {
                    continue;
                }
                if let Some((score, kind)) = score_entry(
                    &span_norm,
                    &span_compact,
                    &span_phone,
                    span_word_count,
                    entry,
                ) {
                    let threshold = match entry.priority {
                        DictionaryPriority::High => 0.90,
                        DictionaryPriority::Normal => 0.94,
                    };
                    if score >= threshold {
                        candidates.push(Candidate {
                            start,
                            end,
                            original: span.to_string(),
                            replacement: entry.target().to_string(),
                            score,
                            kind,
                        });
                    }
                }
            }
        }
    }

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| (b.end - b.start).cmp(&(a.end - a.start)))
    });

    let mut selected: Vec<Candidate> = Vec::new();
    'candidate: for candidate in candidates {
        for existing in &selected {
            if ranges_overlap(candidate.start, candidate.end, existing.start, existing.end) {
                continue 'candidate;
            }
        }
        selected.push(candidate);
    }
    selected.sort_by_key(|candidate| candidate.start);

    let mut out = String::new();
    let mut cursor = 0usize;
    for candidate in selected {
        if cursor > candidate.start {
            continue;
        }
        logger::line(format!(
            "dictionary correction: {:?} -> {:?} kind={} score={:.3}",
            candidate.original, candidate.replacement, candidate.kind, candidate.score
        ));
        out.push_str(&text[cursor..candidate.start]);
        out.push_str(&candidate.replacement);
        cursor = candidate.end;
    }
    out.push_str(&text[cursor..]);
    out
}

fn score_entry(
    span_norm: &str,
    span_compact: &str,
    span_phone: &str,
    span_word_count: usize,
    entry: &DictionaryEntry,
) -> Option<(f64, &'static str)> {
    let mut best = None;
    for alias in entry.spoken_aliases() {
        let alias_norm = normalize(&alias);
        if alias_norm.is_empty() {
            continue;
        }
        if span_norm == alias_norm {
            update_best(&mut best, 1.0, "exact");
        }
        if span_compact == compact(&alias_norm) {
            update_best(&mut best, 0.985, "compact");
        }
        if entry.phonetic && span_word_count > 1 {
            let alias_phone = phonetic_key(&alias_norm);
            if !alias_phone.is_empty() && !span_phone.is_empty() {
                update_best(
                    &mut best,
                    jaro_winkler(span_phone, &alias_phone),
                    "phonetic",
                );
            }
        }
    }
    best
}

fn update_best(best: &mut Option<(f64, &'static str)>, score: f64, kind: &'static str) {
    if best.map(|(old, _)| score > old).unwrap_or(true) {
        *best = Some((score, kind));
    }
}

fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (idx, ch) in text.char_indices() {
        if ch.is_alphanumeric() || ch == '\'' || ch == '-' {
            if start.is_none() {
                start = Some(idx);
            }
        } else if let Some(s) = start.take() {
            tokens.push(Token {
                text: text[s..idx].to_string(),
                start: s,
                end: idx,
            });
        }
    }
    if let Some(s) = start {
        tokens.push(Token {
            text: text[s..].to_string(),
            start: s,
            end: text.len(),
        });
    }
    tokens
}

fn normalize(text: &str) -> String {
    tokenize(text)
        .into_iter()
        .map(|token| {
            token
                .text
                .to_ascii_lowercase()
                .replace("'s", "s")
                .replace('-', " ")
        })
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn phonetic_key(text: &str) -> String {
    let compact = compact(text);
    if compact.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let chars: Vec<char> = compact.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();
        match ch {
            'a' | 'e' | 'i' | 'o' | 'u' => {
                if out.is_empty() {
                    out.push('a');
                }
            }
            'c' => {
                if matches!(next, Some('e' | 'i' | 'y')) {
                    out.push('s');
                } else {
                    out.push('k');
                }
            }
            'q' | 'k' => out.push('k'),
            'x' => out.push_str("ks"),
            'z' => out.push('s'),
            'f' | 'v' => out.push('f'),
            'g' | 'j' => out.push('j'),
            'y' => out.push('i'),
            'w' | 'h' => {}
            _ => out.push(ch),
        }
        i += 1;
    }
    collapse_duplicates(&out)
}

fn collapse_duplicates(text: &str) -> String {
    let mut out = String::new();
    let mut last = None;
    for ch in text.chars() {
        if Some(ch) != last {
            out.push(ch);
            last = Some(ch);
        }
    }
    out
}

fn ranges_overlap(a_start: usize, a_end: usize, b_start: usize, b_end: usize) -> bool {
    a_start < b_end && b_start < a_end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DictionaryEntry, DictionaryPriority};

    #[test]
    fn corrects_exact_and_compact_aliases() {
        let mut entry = DictionaryEntry::new("nix os", "NixOS");
        entry.aliases.push("nix o s".to_string());
        let out = correct_text("I installed nix o s yesterday", &[entry]);
        assert_eq!(out, "I installed NixOS yesterday");
    }

    #[test]
    fn corrects_phonetic_high_priority_match() {
        let mut entry = DictionaryEntry::new("nix os", "NixOS");
        entry.aliases.push("nicsos".to_string());
        entry.aliases.push("nicks os".to_string());
        entry.phonetic = true;
        entry.priority = DictionaryPriority::High;
        let out = correct_text("my nicsos config works", &[entry]);
        assert_eq!(out, "my NixOS config works");
    }

    #[test]
    fn avoids_unrelated_phrase() {
        let mut entry = DictionaryEntry::new("nix os", "NixOS");
        entry.phonetic = true;
        let out = correct_text("Nick's laptop is here", &[entry]);
        assert_eq!(out, "Nick's laptop is here");
    }
}

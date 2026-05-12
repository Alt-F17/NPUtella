use crate::code_context::CodeContext;
use crate::config::AppConfig;
use crate::context::TargetContext;
use crate::dictionary_store::DictionaryStore;

#[derive(Clone, Debug)]
pub struct InsertPlan {
    pub text: String,
    pub press_enter: bool,
    pub skip_paste: bool,
}

pub fn process_transcript(
    raw: &str,
    config: &AppConfig,
    target: &TargetContext,
    code: &CodeContext,
    dictionary: &DictionaryStore,
) -> InsertPlan {
    let mut text = raw.trim().to_string();
    let mut press_enter = false;

    if let Some(command) = parse_learn_command(&text) {
        let learned = dictionary.learn(command.from, command.to);
        return InsertPlan {
            text: if learned {
                "dictionary updated".to_string()
            } else {
                "dictionary already knew that".to_string()
            },
            press_enter: false,
            skip_paste: true,
        };
    }

    if config.smart_formatting {
        let (next, enter) = extract_press_enter(&text);
        text = next;
        press_enter = enter;
    }

    text = apply_snippets(&text, &config.snippets);

    if config.smart_formatting {
        text = smart_format(&text);
    }

    if config.math_formatting && should_apply_math(&text, target) {
        text = apply_math_formatting(&text, target.wants_latex_math());
    }

    if config.code_formatting && target.wants_code_formatting() {
        text = apply_code_words(&text);
        if config.file_tagging && target.wants_file_tags() {
            text = code.tag_files(&text);
        }
        if config.symbol_tagging {
            text = code.tag_symbols(&text);
        }
    }

    InsertPlan {
        text: text.trim().to_string(),
        press_enter,
        skip_paste: false,
    }
}

struct LearnCommand {
    from: String,
    to: String,
}

fn parse_learn_command(text: &str) -> Option<LearnCommand> {
    let lower = text.to_ascii_lowercase();
    for prefix in [
        "learn ",
        "teach ",
        "add word ",
        "remember ",
        "dictionary learn ",
    ] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let tail = &text[text.len() - rest.len()..];
            for splitter in [" as ", " to ", " equals ", " spelled "] {
                if let Some((from, to)) = tail.split_once(splitter) {
                    let from = from.trim().trim_matches('"').to_string();
                    let to = to.trim().trim_matches('"').to_string();
                    if !from.is_empty() && !to.is_empty() {
                        return Some(LearnCommand { from, to });
                    }
                }
            }
        }
    }
    None
}

fn extract_press_enter(text: &str) -> (String, bool) {
    let lower = text.to_ascii_lowercase();
    for phrase in [" press enter", " and press enter", " hit enter", " send it"] {
        if lower.ends_with(phrase) {
            let keep = text.len().saturating_sub(phrase.len());
            return (text[..keep].trim_end().to_string(), true);
        }
    }
    (text.to_string(), false)
}

fn apply_snippets(text: &str, snippets: &[crate::config::Snippet]) -> String {
    let trimmed = text.trim();
    for snippet in snippets {
        if trimmed.eq_ignore_ascii_case(snippet.trigger.trim()) {
            return snippet.expansion.clone();
        }
    }
    text.to_string()
}

fn smart_format(text: &str) -> String {
    let mut out = text.trim().to_string();
    for (from, to) in [
        (" new paragraph ", "\n\n"),
        (" new line ", "\n"),
        (" comma", ","),
        (" period", "."),
        (" full stop", "."),
        (" question mark", "?"),
        (" exclamation mark", "!"),
        (" colon", ":"),
        (" semicolon", ";"),
        (" open parentheses", "("),
        (" close parentheses", ")"),
        (" open parenthesis", "("),
        (" close parenthesis", ")"),
        (" open bracket", "["),
        (" close bracket", "]"),
        (" open brace", "{"),
        (" close brace", "}"),
    ] {
        out = replace_case_insensitive(&out, from, to);
    }
    out = out
        .replace(" ,", ",")
        .replace(" .", ".")
        .replace(" ?", "?")
        .replace(" !", "!")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace("[ ", "[")
        .replace(" ]", "]")
        .replace("{ ", "{")
        .replace(" }", "}");
    capitalize_sentence_start(&out)
}

fn apply_code_words(text: &str) -> String {
    let mut out = text.to_string();
    for (from, to) in [
        (" equals equals", " == "),
        (" not equals", " != "),
        (" greater than or equal to", " >= "),
        (" less than or equal to", " <= "),
        (" greater than", " > "),
        (" less than", " < "),
        (" arrow", " -> "),
        (" fat arrow", " => "),
        (" double colon", "::"),
        (" dot ", "."),
        (" underscore", "_"),
        (" slash", "/"),
        (" backslash", "\\"),
        (" pipe", "|"),
        (" ampersand", "&"),
    ] {
        out = replace_case_insensitive(&out, from, to);
    }
    out
}

fn should_apply_math(text: &str, target: &TargetContext) -> bool {
    let lower = text.to_ascii_lowercase();
    target.wants_latex_math()
        || [
            " squared",
            " cubed",
            " alpha",
            " beta",
            " gamma",
            " lambda",
            " integral",
            " derivative",
            " less than or equal",
            " greater than or equal",
            " plus or minus",
            " over ",
        ]
        .iter()
        .any(|phrase| lower.contains(phrase))
}

fn apply_math_formatting(text: &str, latex: bool) -> String {
    let mut out = text.to_string();
    let greek = [
        ("alpha", "\u{03b1}", "\\alpha"),
        ("beta", "\u{03b2}", "\\beta"),
        ("gamma", "\u{03b3}", "\\gamma"),
        ("delta", "\u{03b4}", "\\delta"),
        ("theta", "\u{03b8}", "\\theta"),
        ("lambda", "\u{03bb}", "\\lambda"),
        ("mu", "\u{03bc}", "\\mu"),
        ("pi", "\u{03c0}", "\\pi"),
        ("sigma", "\u{03c3}", "\\sigma"),
        ("omega", "\u{03c9}", "\\omega"),
    ];
    for (spoken, unicode, tex) in greek {
        out = replace_word(&out, spoken, if latex { tex } else { unicode });
    }
    for (from, unicode, tex) in [
        (" plus or minus ", " \u{00b1} ", " \\pm "),
        (" less than or equal to ", " \u{2264} ", " \\le "),
        (" greater than or equal to ", " \u{2265} ", " \\ge "),
        (" not equal to ", " \u{2260} ", " \\ne "),
        (" times ", " \u{00d7} ", " \\times "),
        (" divided by ", " \u{00f7} ", " / "),
        (" squared", "\u{00b2}", "^2"),
        (" cubed", "\u{00b3}", "^3"),
        (" integral ", " \u{222b} ", " \\int "),
        (" derivative ", " d/dx ", " \\frac{d}{dx} "),
    ] {
        out = replace_case_insensitive(&out, from, if latex { tex } else { unicode });
    }
    out
}

fn capitalize_sentence_start(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut cap_next = true;
    for ch in text.chars() {
        if cap_next && ch.is_ascii_alphabetic() {
            out.push(ch.to_ascii_uppercase());
            cap_next = false;
        } else {
            out.push(ch);
        }
        if matches!(ch, '.' | '?' | '!' | '\n') {
            cap_next = true;
        } else if !ch.is_whitespace() {
            cap_next = false;
        }
    }
    out
}

fn replace_word(text: &str, word: &str, replacement: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let word = word.to_ascii_lowercase();
    let mut out = String::new();
    let mut start = 0usize;
    let mut cursor = 0usize;
    while let Some(pos) = lower[cursor..].find(&word) {
        let idx = cursor + pos;
        let end = idx + word.len();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DictionaryEntry, Snippet};
    use crate::context::{AppKind, TargetContext};
    use crate::dictionary_store::DictionaryStore;
    use std::path::Path;

    #[test]
    fn handles_dictionary_snippet_and_enter() {
        let config = AppConfig {
            dictionary: vec![DictionaryEntry::new("n p u tella", "NPUtella")],
            snippets: vec![Snippet {
                trigger: "sign off".to_string(),
                expansion: "Thanks,\nFelix".to_string(),
            }],
            ..AppConfig::default()
        };
        let code = CodeContext {
            files: Vec::new(),
            symbols: Vec::new(),
        };
        let dictionary = DictionaryStore::load(Path::new("."), config.dictionary.clone());
        let target = TargetContext {
            title: "Notepad".to_string(),
            kind: AppKind::Generic,
        };
        let plan = process_transcript("sign off press enter", &config, &target, &code, &dictionary);
        assert_eq!(plan.text, "Thanks,\nFelix");
        assert!(plan.press_enter);
    }

    #[test]
    fn formats_math() {
        let target = TargetContext {
            title: "Notepad".to_string(),
            kind: AppKind::Generic,
        };
        assert_eq!(
            apply_math_formatting("alpha squared", false),
            "\u{03b1}\u{00b2}"
        );
        assert_eq!(apply_math_formatting("alpha squared", true), "\\alpha^2");
        assert!(should_apply_math("x less than or equal to y", &target));
    }
}

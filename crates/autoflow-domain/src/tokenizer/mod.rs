use std::collections::BTreeMap;
use std::sync::OnceLock;

use chrono::{DateTime, FixedOffset, Local};
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

mod display;

pub use display::{format_template, parse_template};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePipeline {
    #[serde(default)]
    pub segments: Vec<TemplateSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TemplateSegment {
    Literal { value: String },
    Token { name: String },
    Transform { name: String },
    RegexPreset { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenContext {
    pub original: String,
    pub stem: String,
    pub ext: String,
    pub parent: String,
    #[serde(default)]
    pub grandparent: String,
    pub now: DateTime<FixedOffset>,
    pub counter: u64,
    #[serde(default)]
    pub counter_formatted: String,
    pub guid: Uuid,
    pub index: u64,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
}

impl TokenContext {
    pub fn from_path(path: &std::path::Path, index: u64, counter: u64) -> Self {
        let original = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let parent_path = path.parent();
        let parent = parent_path
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let grandparent = parent_path
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Self {
            original,
            stem,
            ext,
            parent,
            grandparent,
            now: Local::now().fixed_offset(),
            counter,
            counter_formatted: String::new(),
            guid: Uuid::new_v4(),
            index,
            variables: BTreeMap::new(),
        }
    }

    pub fn with_variables(mut self, variables: BTreeMap<String, String>) -> Self {
        self.variables = variables;
        self
    }

    /// Build a context whose `parent`/`grandparent` tokens are relative to a
    /// blueprint root. Files directly inside the root get empty parent tokens
    /// instead of the root folder's own name.
    pub fn from_path_with_root(
        path: &std::path::Path,
        root: &std::path::Path,
        index: u64,
        counter: u64,
    ) -> Self {
        let mut ctx = Self::from_path(path, index, counter);
        if let Ok(relative) = path.strip_prefix(root) {
            let parent_path = relative.parent();
            ctx.parent = parent_path
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            ctx.grandparent = parent_path
                .and_then(|p| p.parent())
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
        }
        ctx
    }

    /// Build a context for a directory path. Folder names are not split into
    /// stem/extension the way file names are.
    pub fn for_directory(path: &std::path::Path, index: u64, counter: u64) -> Self {
        let mut ctx = Self::from_path(path, index, counter);
        ctx.original = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        ctx.stem = ctx.original.clone();
        ctx.ext = String::new();
        ctx
    }
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("unknown token: {0}")]
    UnknownToken(String),
    #[error("unknown transform: {0}")]
    UnknownTransform(String),
    #[error("unknown regex preset: {0}")]
    UnknownRegexPreset(String),
    #[error("invalid transform: {0}")]
    InvalidTransform(String),
    #[error("regex error: {0}")]
    Regex(String),
}

pub fn evaluate(pipeline: &TemplatePipeline, ctx: &TokenContext) -> Result<String, TokenError> {
    let mut value = String::new();
    for segment in &pipeline.segments {
        match segment {
            TemplateSegment::Literal { value: lit } => value.push_str(lit),
            TemplateSegment::Token { name } => value.push_str(&resolve_token(name, ctx)?),
            TemplateSegment::Transform { name } => value = apply_transform(name, &value)?,
            TemplateSegment::RegexPreset { name } => value = apply_regex_preset(name, &value)?,
        }
    }
    Ok(value)
}

fn resolve_token(name: &str, ctx: &TokenContext) -> Result<String, TokenError> {
    let lower = name.to_ascii_lowercase();
    Ok(match lower.as_str() {
        "original" => ctx.original.clone(),
        "stem" => ctx.stem.clone(),
        "ext" => ctx.ext.clone(),
        "parent" => ctx.parent.clone(),
        "grandparent" => ctx.grandparent.clone(),
        "date" => ctx.now.format("%Y-%m-%d").to_string(),
        "year" => ctx.now.format("%Y").to_string(),
        "month" => ctx.now.format("%m").to_string(),
        "day" => ctx.now.format("%d").to_string(),
        "hour" => ctx.now.format("%H").to_string(),
        "minute" => ctx.now.format("%M").to_string(),
        "second" => ctx.now.format("%S").to_string(),
        "counter" => {
            if !ctx.counter_formatted.is_empty() {
                ctx.counter_formatted.clone()
            } else {
                ctx.counter.to_string()
            }
        }
        "guid" => ctx.guid.to_string(),
        "index" => ctx.index.to_string(),
        other => {
            if let Some(value) = ctx.variables.get(other) {
                value.clone()
            } else if let Some(value) = ctx.variables.get(name) {
                value.clone()
            } else {
                return Err(TokenError::UnknownToken(other.to_string()));
            }
        }
    })
}

fn apply_transform(name: &str, input: &str) -> Result<String, TokenError> {
    let invocation = TransformInvocation::parse(name)?;
    Ok(match invocation.name.as_str() {
        "upper" => {
            invocation.require_no_args()?;
            input.to_uppercase()
        }
        "lower" => {
            invocation.require_no_args()?;
            input.to_lowercase()
        }
        "capitalized" => {
            invocation.require_no_args()?;
            capitalized(input)
        }
        "title" => {
            invocation.require_no_args()?;
            title_case(input)
        }
        "snake" => {
            invocation.require_no_args()?;
            to_snake(input)
        }
        "camel" => {
            invocation.require_no_args()?;
            to_camel(input)
        }
        "pascal" => {
            invocation.require_no_args()?;
            to_pascal(input)
        }
        "kebab" => {
            invocation.require_no_args()?;
            to_kebab(input)
        }
        "trim" => {
            invocation.require_no_args()?;
            input.trim().to_string()
        }
        "collapse_spaces" => {
            invocation.require_no_args()?;
            collapse_spaces(input)
        }
        "strip_symbols" => {
            invocation.require_no_args()?;
            strip_symbols(input)
        }
        "spaces_to_underscore" => {
            invocation.require_no_args()?;
            spaces_to_underscore(input)
        }
        "sanitize_windows_filename" => {
            invocation.require_no_args()?;
            sanitize_windows_filename(input)
        }
        "take" => take_chars(input, &invocation)?,
        "skip" => skip_chars(input, &invocation)?,
        "slice" => slice_chars(input, &invocation)?,
        _ => return Err(TokenError::UnknownTransform(invocation.raw)),
    })
}

fn compile_regex(cell: &'static OnceLock<Regex>, pattern: &'static str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).expect("valid built-in regex"))
}

fn apply_regex_preset(name: &str, input: &str) -> Result<String, TokenError> {
    static REMOVE_DIGITS: OnceLock<Regex> = OnceLock::new();
    static STRIP_SPECIAL: OnceLock<Regex> = OnceLock::new();
    static EXTRACT_DATE: OnceLock<Regex> = OnceLock::new();
    static REMOVE_PARENS: OnceLock<Regex> = OnceLock::new();
    static DIGITS_ONLY: OnceLock<Regex> = OnceLock::new();
    static LETTERS_ONLY: OnceLock<Regex> = OnceLock::new();

    let normalized = normalize_identifier(name);
    match normalized.as_str() {
        "remove_digits" => Ok(compile_regex(&REMOVE_DIGITS, r"\d+")
            .replace_all(input, "")
            .to_string()),
        "strip_special" => Ok(compile_regex(&STRIP_SPECIAL, r"[^\p{L}\p{N}._\- ]+")
            .replace_all(input, "")
            .to_string()),
        "extract_date" => Ok(compile_regex(
            &EXTRACT_DATE,
            r"\d{4}[-_.]\d{2}[-_.]\d{2}|\d{2}[-_.]\d{2}[-_.]\d{4}",
        )
        .find(input)
        .map(|date| date.as_str().to_string())
        .unwrap_or_default()),
        "remove_parens" => Ok(collapse_spaces(
            compile_regex(&REMOVE_PARENS, r"\s*\([^)]*\)\s*")
                .replace_all(input, " ")
                .as_ref(),
        )),
        "digits_only" => Ok(compile_regex(&DIGITS_ONLY, r"\D+")
            .replace_all(input, "")
            .to_string()),
        "letters_only" => Ok(compile_regex(&LETTERS_ONLY, r"[^\p{L}]+")
            .replace_all(input, "")
            .to_string()),
        _ => Err(TokenError::UnknownRegexPreset(name.to_string())),
    }
}

#[derive(Debug, PartialEq)]
struct TransformInvocation {
    raw: String,
    name: String,
    args: Vec<String>,
}

impl TransformInvocation {
    fn parse(raw: &str) -> Result<Self, TokenError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(TokenError::InvalidTransform("empty transform".into()));
        }

        let (name, args) = if let Some(open) = trimmed.find('(') {
            if !trimmed.ends_with(')') {
                return Err(TokenError::InvalidTransform(trimmed.to_string()));
            }
            let name = trimmed[..open].trim();
            let args = trimmed[open + 1..trimmed.len() - 1]
                .split(',')
                .map(|arg| arg.trim().to_string())
                .filter(|arg| !arg.is_empty())
                .collect();
            (name, args)
        } else {
            (trimmed, Vec::new())
        };

        Ok(Self {
            raw: trimmed.to_string(),
            name: normalize_identifier(name),
            args,
        })
    }

    fn require_no_args(&self) -> Result<(), TokenError> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(TokenError::InvalidTransform(format!(
                "{} does not accept arguments",
                self.raw
            )))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Start,
    End,
}

fn take_chars(input: &str, invocation: &TransformInvocation) -> Result<String, TokenError> {
    require_arg_range(invocation, 1, 2)?;
    let count = parse_usize_arg(invocation, 0)?;
    match parse_direction_arg(invocation.args.get(1))? {
        Direction::Start => Ok(input.chars().take(count).collect()),
        Direction::End => {
            let len = input.chars().count();
            Ok(input
                .chars()
                .skip(len.saturating_sub(count))
                .collect::<String>())
        }
    }
}

fn skip_chars(input: &str, invocation: &TransformInvocation) -> Result<String, TokenError> {
    require_arg_range(invocation, 1, 2)?;
    let count = parse_usize_arg(invocation, 0)?;
    match parse_direction_arg(invocation.args.get(1))? {
        Direction::Start => Ok(input.chars().skip(count).collect()),
        Direction::End => {
            let keep = input.chars().count().saturating_sub(count);
            Ok(input.chars().take(keep).collect())
        }
    }
}

fn slice_chars(input: &str, invocation: &TransformInvocation) -> Result<String, TokenError> {
    require_arg_range(invocation, 2, 2)?;
    let start = parse_usize_arg(invocation, 0)?;
    let end = parse_usize_arg(invocation, 1)?;
    if start >= end {
        return Ok(String::new());
    }
    Ok(input.chars().skip(start).take(end - start).collect())
}

fn require_arg_range(
    invocation: &TransformInvocation,
    min: usize,
    max: usize,
) -> Result<(), TokenError> {
    let len = invocation.args.len();
    if len < min || len > max {
        Err(TokenError::InvalidTransform(format!(
            "{} expects {} argument(s)",
            invocation.raw, min
        )))
    } else {
        Ok(())
    }
}

fn parse_usize_arg(invocation: &TransformInvocation, index: usize) -> Result<usize, TokenError> {
    invocation
        .args
        .get(index)
        .ok_or_else(|| TokenError::InvalidTransform(invocation.raw.clone()))?
        .parse::<usize>()
        .map_err(|_| TokenError::InvalidTransform(invocation.raw.clone()))
}

fn parse_direction_arg(arg: Option<&String>) -> Result<Direction, TokenError> {
    let Some(arg) = arg else {
        return Ok(Direction::Start);
    };
    match normalize_identifier(arg).as_str() {
        "start" | "from_start" | "left" | "front" | "begin" | "beginning" => Ok(Direction::Start),
        "end" | "from_end" | "right" | "back" | "last" => Ok(Direction::End),
        _ => Err(TokenError::InvalidTransform(format!(
            "unknown direction: {arg}"
        ))),
    }
}

fn normalize_identifier(name: &str) -> String {
    let mut out = String::new();
    let mut previous_was_separator = true;

    for ch in name.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && !previous_was_separator && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if ch == '_' || ch == '-' || ch.is_whitespace() {
            if !out.is_empty() && !out.ends_with('_') {
                out.push('_');
            }
            previous_was_separator = true;
        }
    }

    out.trim_matches('_').to_string()
}

fn capitalized(input: &str) -> String {
    let lower = input.to_lowercase();
    let mut chars = lower.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut out = String::new();
    out.extend(first.to_uppercase());
    out.extend(chars);
    out
}

fn title_case(input: &str) -> String {
    let mut out = String::new();
    let mut next_is_word_start = true;

    for ch in input.to_lowercase().chars() {
        if ch.is_alphanumeric() {
            if next_is_word_start {
                out.extend(ch.to_uppercase());
            } else {
                out.push(ch);
            }
            next_is_word_start = false;
        } else if ch == '\'' {
            // Keep apostrophes inside the current word: don't -> Don't.
            out.push(ch);
        } else {
            out.push(ch);
            next_is_word_start = true;
        }
    }

    out
}

fn to_snake(input: &str) -> String {
    words_for_case(input).join("_")
}

fn to_camel(input: &str) -> String {
    let words = words_for_case(input);
    let Some((first, rest)) = words.split_first() else {
        return String::new();
    };

    let mut out = first.clone();
    for word in rest {
        out.push_str(&capitalized(word));
    }
    out
}

fn to_pascal(input: &str) -> String {
    let mut out = String::new();
    for word in words_for_case(input) {
        out.push_str(&capitalized(&word));
    }
    out
}

fn to_kebab(input: &str) -> String {
    to_snake(input).replace('_', "-")
}

fn words_for_case(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = input.chars().collect();

    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_alphanumeric() {
            let previous = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
            let next = chars.get(index + 1).copied();
            let starts_new_word = ch.is_uppercase()
                && !current.is_empty()
                && (previous
                    .map(|prev| prev.is_lowercase() || prev.is_numeric())
                    .unwrap_or(false)
                    || previous.map(|prev| prev.is_uppercase()).unwrap_or(false)
                        && next.map(|next| next.is_lowercase()).unwrap_or(false));

            if starts_new_word {
                words.push(std::mem::take(&mut current));
            }
            current.extend(ch.to_lowercase());
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn collapse_spaces(input: &str) -> String {
    static COLLAPSE_SPACES: OnceLock<Regex> = OnceLock::new();
    compile_regex(&COLLAPSE_SPACES, r"\s+")
        .replace_all(input.trim(), " ")
        .to_string()
}

fn collapse_underscores(input: &str) -> String {
    static COLLAPSE_UNDERSCORES: OnceLock<Regex> = OnceLock::new();
    compile_regex(&COLLAPSE_UNDERSCORES, r"_+")
        .replace_all(input.trim_matches('_'), "_")
        .to_string()
}

fn strip_symbols(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace() || matches!(ch, '.' | '_' | '-'))
        .collect()
}

fn spaces_to_underscore(input: &str) -> String {
    collapse_underscores(&collapse_spaces(input).replace(' ', "_"))
}

fn sanitize_windows_filename(input: &str) -> String {
    const INVALID: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut out: String = input
        .chars()
        .map(|c| {
            if INVALID.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    out = out.trim_end_matches(&['.', ' '][..]).to_string();
    if out.is_empty() {
        return "file".into();
    }

    let stem = out
        .split('.')
        .next()
        .unwrap_or(&out)
        .trim()
        .to_ascii_uppercase();
    if crate::path_policy::WINDOWS_RESERVED.contains(&stem.as_str()) {
        out.insert(0, '_');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn evaluates_tokens_and_transforms() {
        // Use native separators so the test exercises the same path semantics on every OS.
        let path = format!(
            "in{}Docs{}report.PDF",
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        );
        let ctx = TokenContext::from_path(Path::new(&path), 3, 7);
        let pipeline = TemplatePipeline {
            segments: vec![
                TemplateSegment::Token {
                    name: "parent".into(),
                },
                TemplateSegment::Literal { value: "/".into() },
                TemplateSegment::Token {
                    name: "stem".into(),
                },
                TemplateSegment::Transform {
                    name: "lower".into(),
                },
            ],
        };
        assert_eq!(evaluate(&pipeline, &ctx).unwrap(), "docs/report");
    }

    #[test]
    fn sanitize_windows_filename_removes_invalid_chars() {
        let pipeline = TemplatePipeline {
            segments: vec![
                TemplateSegment::Literal {
                    value: "bad<file>:name".into(),
                },
                TemplateSegment::Transform {
                    name: "sanitize_windows_filename".into(),
                },
            ],
        };
        let ctx = TokenContext::from_path(Path::new("x"), 0, 0);
        assert_eq!(evaluate(&pipeline, &ctx).unwrap(), "bad_file__name");
    }

    #[test]
    fn evaluates_literal_path_segments_and_brackets() {
        let ctx = TokenContext::from_path(Path::new("root/Docs/report.pdf"), 1, 1);
        let pipeline = parse_template("{parent}/docs/{stem}");
        assert_eq!(evaluate(&pipeline, &ctx).unwrap(), "Docs/docs/report");

        let bracketed = parse_template("[2024] {stem}");
        assert_eq!(evaluate(&bracketed, &ctx).unwrap(), "[2024] report");
    }

    #[test]
    fn root_relative_context_keeps_root_files_parent_empty() {
        let ctx = TokenContext::from_path_with_root(
            Path::new("root/report.pdf"),
            Path::new("root"),
            1,
            1,
        );
        assert_eq!(ctx.parent, "");
        assert_eq!(ctx.grandparent, "");

        let nested = TokenContext::from_path_with_root(
            Path::new("root/in/Docs/report.pdf"),
            Path::new("root"),
            1,
            1,
        );
        assert_eq!(nested.parent, "Docs");
        assert_eq!(nested.grandparent, "in");
    }

    #[test]
    fn directory_context_does_not_split_folder_extensions() {
        let ctx = TokenContext::for_directory(Path::new("root/v1.2"), 1, 1);
        assert_eq!(ctx.stem, "v1.2");
        assert_eq!(ctx.ext, "");
        assert_eq!(ctx.original, "v1.2");
    }

    #[test]
    fn evaluates_new_tokens_from_path_and_clock() {
        let mut ctx = TokenContext::from_path(Path::new("root/in/Docs/report.PDF"), 3, 7);
        ctx.now = DateTime::parse_from_rfc3339("2026-06-19T14:05:42Z").unwrap();

        let pipeline = TemplatePipeline {
            segments: vec![
                TemplateSegment::Token {
                    name: "grandparent".into(),
                },
                TemplateSegment::Literal { value: "/".into() },
                TemplateSegment::Token {
                    name: "date".into(),
                },
                TemplateSegment::Literal { value: "/".into() },
                TemplateSegment::Token {
                    name: "parent".into(),
                },
            ],
        };

        assert_eq!(evaluate(&pipeline, &ctx).unwrap(), "in/2026-06-19/Docs");
    }

    #[test]
    fn evaluates_case_and_cleanup_transforms() {
        let ctx = TokenContext::from_path(Path::new("x"), 0, 0);

        assert_transform(&ctx, "hello WORLD", "capitalized", "Hello world");
        assert_transform(&ctx, "hello WORLD-file", "title", "Hello World-File");
        assert_transform(&ctx, "hello WORLD-file", "camel", "helloWorldFile");
        assert_transform(&ctx, "hello WORLD-file", "pascal", "HelloWorldFile");
        assert_transform(&ctx, "XMLParser", "snake", "xml_parser");
        assert_transform(&ctx, "XMLParser", "kebab", "xml-parser");
        assert_transform(&ctx, "don't stop", "title", "Don't Stop");
        assert_transform(&ctx, "a  b\tc", "spacesToUnderscore", "a_b_c");
        assert_transform(&ctx, "a@b#c.pdf", "strip_symbols", "abc.pdf");
        assert_transform(&ctx, "CON", "sanitize_windows_filename", "_CON");
    }

    #[test]
    fn evaluates_substring_transforms() {
        let ctx = TokenContext::from_path(Path::new("x"), 0, 0);

        assert_transform(&ctx, "abcdef", "take(3)", "abc");
        assert_transform(&ctx, "abcdef", "take(2, end)", "ef");
        assert_transform(&ctx, "abcdef", "skip(2)", "cdef");
        assert_transform(&ctx, "abcdef", "skip(2, end)", "abcd");
        assert_transform(&ctx, "abcdef", "slice(1, 4)", "bcd");
    }

    #[test]
    fn rejects_invalid_substring_arguments() {
        let ctx = TokenContext::from_path(Path::new("x"), 0, 0);
        let pipeline = TemplatePipeline {
            segments: vec![
                TemplateSegment::Literal {
                    value: "abcdef".into(),
                },
                TemplateSegment::Transform {
                    name: "take(nope)".into(),
                },
            ],
        };

        assert!(matches!(
            evaluate(&pipeline, &ctx),
            Err(TokenError::InvalidTransform(_))
        ));
    }

    #[test]
    fn evaluates_regex_presets_and_camel_case_aliases() {
        let ctx = TokenContext::from_path(Path::new("x"), 0, 0);

        assert_regex(
            &ctx,
            "NF 2026-06-19 (old) #42",
            "extract_date",
            "2026-06-19",
        );
        assert_regex(&ctx, "NF 2026-06-19 (old) #42", "digitsOnly", "2026061942");
        assert_regex(&ctx, "NF 2026-06-19 (old) #42", "letters_only", "NFold");
        assert_regex(
            &ctx,
            "NF 2026-06-19 (old) #42",
            "removeParens",
            "NF 2026-06-19 #42",
        );
        assert_regex(&ctx, "arquivo 123.pdf", "removeDigits", "arquivo .pdf");
        assert_regex(&ctx, "arquivo@# 1.pdf", "stripSpecial", "arquivo 1.pdf");
    }

    fn assert_transform(ctx: &TokenContext, input: &str, transform: &str, expected: &str) {
        let pipeline = TemplatePipeline {
            segments: vec![
                TemplateSegment::Literal {
                    value: input.into(),
                },
                TemplateSegment::Transform {
                    name: transform.into(),
                },
            ],
        };

        assert_eq!(evaluate(&pipeline, ctx).unwrap(), expected);
    }

    fn assert_regex(ctx: &TokenContext, input: &str, preset: &str, expected: &str) {
        let pipeline = TemplatePipeline {
            segments: vec![
                TemplateSegment::Literal {
                    value: input.into(),
                },
                TemplateSegment::RegexPreset {
                    name: preset.into(),
                },
            ],
        };

        assert_eq!(evaluate(&pipeline, ctx).unwrap(), expected);
    }
}

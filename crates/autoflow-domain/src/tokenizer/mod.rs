use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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
    pub now: DateTime<Utc>,
    pub counter: u64,
    #[serde(default)]
    pub counter_formatted: String,
    pub guid: Uuid,
    pub index: u64,
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
            now: Utc::now(),
            counter,
            counter_formatted: String::new(),
            guid: Uuid::new_v4(),
            index,
        }
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
        other => return Err(TokenError::UnknownToken(other.to_string())),
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

fn apply_regex_preset(name: &str, input: &str) -> Result<String, TokenError> {
    let normalized = normalize_identifier(name);
    match normalized.as_str() {
        "remove_digits" => {
            let re = Regex::new(r"\d+").map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(re.replace_all(input, "").to_string())
        }
        "strip_special" => {
            let re =
                Regex::new(r"[^\p{L}\p{N}._\- ]+").map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(re.replace_all(input, "").to_string())
        }
        "extract_date" => {
            let re = Regex::new(r"\d{4}[-_.]\d{2}[-_.]\d{2}|\d{2}[-_.]\d{2}[-_.]\d{4}")
                .map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(re
                .find(input)
                .map(|date| date.as_str().to_string())
                .unwrap_or_default())
        }
        "remove_parens" => {
            let re =
                Regex::new(r"\s*\([^)]*\)\s*").map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(collapse_spaces(re.replace_all(input, " ").as_ref()))
        }
        "digits_only" => {
            let re = Regex::new(r"\D+").map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(re.replace_all(input, "").to_string())
        }
        "letters_only" => {
            let re = Regex::new(r"[^\p{L}]+").map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(re.replace_all(input, "").to_string())
        }
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
        } else {
            out.push(ch);
            next_is_word_start = true;
        }
    }

    out
}

fn to_snake(input: &str) -> String {
    let mut out = String::new();
    for (i, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '-' {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    collapse_underscores(&out)
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
    let re = Regex::new(r"\s+").expect("valid regex");
    re.replace_all(input.trim(), " ").to_string()
}

fn collapse_underscores(input: &str) -> String {
    let re = Regex::new(r"_+").expect("valid regex");
    re.replace_all(input.trim_matches('_'), "_").to_string()
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
        .map(|c| if INVALID.contains(&c) { '_' } else { c })
        .collect();
    out = out.trim_end_matches(&['.', ' '][..]).to_string();
    if out.is_empty() {
        "file".into()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn evaluates_tokens_and_transforms() {
        let ctx = TokenContext::from_path(Path::new(r"C:\in\Docs\report.PDF"), 3, 7);
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
    fn evaluates_new_tokens_from_path_and_clock() {
        let mut ctx = TokenContext::from_path(Path::new("root/in/Docs/report.PDF"), 3, 7);
        ctx.now = DateTime::parse_from_rfc3339("2026-06-19T14:05:42Z")
            .unwrap()
            .with_timezone(&Utc);

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
        assert_transform(&ctx, "a  b\tc", "spacesToUnderscore", "a_b_c");
        assert_transform(&ctx, "a@b#c.pdf", "strip_symbols", "abc.pdf");
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

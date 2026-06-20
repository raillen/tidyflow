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
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Self {
            original,
            stem,
            ext,
            parent,
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
    let lower = name.to_ascii_lowercase();
    Ok(match lower.as_str() {
        "upper" => input.to_ascii_uppercase(),
        "lower" => input.to_ascii_lowercase(),
        "snake" => to_snake(input),
        "kebab" => to_kebab(input),
        "trim" => input.trim().to_string(),
        "collapse_spaces" => collapse_spaces(input),
        "sanitize_windows_filename" => sanitize_windows_filename(input),
        other => return Err(TokenError::UnknownTransform(other.to_string())),
    })
}

fn apply_regex_preset(name: &str, input: &str) -> Result<String, TokenError> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        "remove_digits" => {
            let re = Regex::new(r"\d+").map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(re.replace_all(input, "").to_string())
        }
        "strip_special" => {
            let re =
                Regex::new(r"[^a-zA-Z0-9._\- ]+").map_err(|e| TokenError::Regex(e.to_string()))?;
            Ok(re.replace_all(input, "").to_string())
        }
        other => Err(TokenError::UnknownRegexPreset(other.to_string())),
    }
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

fn to_kebab(input: &str) -> String {
    to_snake(input).replace('_', "-")
}

fn collapse_spaces(input: &str) -> String {
    let re = Regex::new(r"\s+").expect("valid regex");
    re.replace_all(input.trim(), " ").to_string()
}

fn collapse_underscores(input: &str) -> String {
    let re = Regex::new(r"_+").expect("valid regex");
    re.replace_all(input.trim_matches('_'), "_").to_string()
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
                TemplateSegment::Literal {
                    value: "/".into(),
                },
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
}

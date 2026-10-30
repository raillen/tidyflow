use super::{normalize_identifier, TemplatePipeline, TemplateSegment};

const TRANSFORMS: &[&str] = &[
    "upper",
    "lower",
    "capitalized",
    "title",
    "snake",
    "camel",
    "pascal",
    "kebab",
    "trim",
    "collapse_spaces",
    "strip_symbols",
    "spaces_to_underscore",
    "sanitize_windows_filename",
    "take",
    "skip",
    "slice",
];

const REGEX_PRESETS: &[&str] = &[
    "remove_digits",
    "strip_special",
    "extract_date",
    "remove_parens",
    "digits_only",
    "letters_only",
];

fn is_known_transform(name: &str) -> bool {
    let base = name.split('(').next().unwrap_or(name);
    TRANSFORMS.contains(&normalize_identifier(base).as_str())
}

fn is_known_regex_preset(name: &str) -> bool {
    REGEX_PRESETS.contains(&normalize_identifier(name).as_str())
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_escapable(ch: char) -> bool {
    matches!(ch, '[' | ']' | '/' | '\\')
}

fn read_until(chars: &[char], start: usize, delimiter: char) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut index = start;
    while index < chars.len() {
        let ch = chars[index];
        if ch == delimiter {
            return Some((value, index + 1));
        }
        if (delimiter == ']' || delimiter == '/') && (ch == '\n' || ch == '\r') {
            return None;
        }
        value.push(ch);
        index += 1;
    }
    None
}

fn flush_literal(segments: &mut Vec<TemplateSegment>, literal: &mut String) {
    if !literal.is_empty() {
        segments.push(TemplateSegment::Literal {
            value: std::mem::take(literal),
        });
    }
}

/// Parse the human-facing template syntax into the canonical pipeline.
///
/// Supported segments:
/// - `{token}`
/// - `[transform]` / `[take(3, end)]`
/// - `/regex_preset/`
/// - literals, with `\{`, `\}`, `\[`, `\]`, `\/` and `\\` escapes
///
/// A `/name/` sequence only becomes a regex preset when `name` is a known
/// preset. This keeps ordinary path segments such as `/docs/` literal.
/// A `[...]` sequence only becomes a transform when it starts with a known
/// transform name; other bracketed text stays literal.
pub fn parse_template(input: &str) -> TemplatePipeline {
    let chars: Vec<char> = input.chars().collect();
    let mut segments = Vec::new();
    let mut literal = String::new();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        match ch {
            '\\' if index + 1 < chars.len() && is_escapable(chars[index + 1]) => {
                literal.push(chars[index + 1]);
                index += 2;
            }
            '{' if index + 1 < chars.len() && chars[index + 1] == '{' => {
                literal.push('{');
                index += 2;
            }
            '}' if index + 1 < chars.len() && chars[index + 1] == '}' => {
                literal.push('}');
                index += 2;
            }
            '{' => {
                if let Some((content, next)) = read_until(&chars, index + 1, '}') {
                    if is_identifier(&content) {
                        flush_literal(&mut segments, &mut literal);
                        segments.push(TemplateSegment::Token { name: content });
                        index = next;
                        continue;
                    }
                }
                literal.push(ch);
                index += 1;
            }
            '[' => {
                if let Some((content, next)) = read_until(&chars, index + 1, ']') {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() && is_known_transform(trimmed) {
                        flush_literal(&mut segments, &mut literal);
                        segments.push(TemplateSegment::Transform {
                            name: trimmed.to_string(),
                        });
                        index = next;
                        continue;
                    }
                }
                literal.push(ch);
                index += 1;
            }
            '/' => {
                if let Some((content, next)) = read_until(&chars, index + 1, '/') {
                    if is_identifier(&content) && is_known_regex_preset(&content) {
                        flush_literal(&mut segments, &mut literal);
                        segments.push(TemplateSegment::RegexPreset { name: content });
                        index = next;
                        continue;
                    }
                }
                literal.push(ch);
                index += 1;
            }
            _ => {
                literal.push(ch);
                index += 1;
            }
        }
    }

    flush_literal(&mut segments, &mut literal);
    TemplatePipeline { segments }
}

fn needs_slash_escape(value: &str, slash_index: usize) -> bool {
    let rest = &value[slash_index + 1..];
    let Some(end) = rest.find('/') else {
        return false;
    };
    let candidate = &rest[..end];
    is_identifier(candidate) && is_known_regex_preset(candidate)
}

/// Render a pipeline back to the display syntax, escaping literals so a
/// parse/format round-trip preserves them.
pub fn format_template(pipeline: &TemplatePipeline) -> String {
    let mut out = String::new();
    for (segment_index, segment) in pipeline.segments.iter().enumerate() {
        match segment {
            TemplateSegment::Literal { value } => {
                let followed_by_segment = segment_index + 1 < pipeline.segments.len();
                for (index, ch) in value.char_indices() {
                    match ch {
                        '\\' => {
                            let next = value[index + 1..].chars().next();
                            if next.is_some_and(is_escapable)
                                || (next.is_none() && followed_by_segment)
                            {
                                out.push('\\');
                            }
                            out.push('\\');
                        }
                        '{' => out.push_str("{{"),
                        '}' => out.push_str("}}"),
                        '[' | ']' => {
                            out.push('\\');
                            out.push(ch);
                        }
                        '/' if needs_slash_escape(value, index) => {
                            out.push('\\');
                            out.push('/');
                        }
                        _ => out.push(ch),
                    }
                }
            }
            TemplateSegment::Token { name } => {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
            TemplateSegment::Transform { name } => {
                out.push('[');
                out.push_str(name);
                out.push(']');
            }
            TemplateSegment::RegexPreset { name } => {
                out.push('/');
                out.push_str(name);
                out.push('/');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> TemplatePipeline {
        parse_template(input)
    }

    #[test]
    fn keeps_ordinary_path_segments_literal() {
        let pipeline = parse("{parent}/docs/{stem}");
        assert_eq!(
            pipeline.segments,
            vec![
                TemplateSegment::Token {
                    name: "parent".into()
                },
                TemplateSegment::Literal {
                    value: "/docs/".into()
                },
                TemplateSegment::Token {
                    name: "stem".into()
                },
            ]
        );
    }

    #[test]
    fn keeps_unknown_alphanumeric_segments_literal() {
        let pipeline = parse("{year}/v2/{stem}");
        assert_eq!(
            pipeline.segments,
            vec![
                TemplateSegment::Token {
                    name: "year".into()
                },
                TemplateSegment::Literal {
                    value: "/v2/".into()
                },
                TemplateSegment::Token {
                    name: "stem".into()
                },
            ]
        );
    }

    #[test]
    fn recognizes_known_presets() {
        let pipeline = parse("{stem}/remove_digits/{ext}");
        assert_eq!(
            pipeline.segments,
            vec![
                TemplateSegment::Token {
                    name: "stem".into()
                },
                TemplateSegment::RegexPreset {
                    name: "remove_digits".into()
                },
                TemplateSegment::Token { name: "ext".into() },
            ]
        );
    }

    #[test]
    fn keeps_unknown_bracketed_text_literal() {
        assert_eq!(
            parse("[2024] report").segments,
            vec![TemplateSegment::Literal {
                value: "[2024] report".into()
            }]
        );
    }

    #[test]
    fn recognizes_known_transforms() {
        assert_eq!(
            parse("{stem}[take(3, end)]").segments,
            vec![
                TemplateSegment::Token {
                    name: "stem".into()
                },
                TemplateSegment::Transform {
                    name: "take(3, end)".into()
                },
            ]
        );
    }

    #[test]
    fn supports_escapes_and_preserves_windows_separators() {
        let pipeline = parse(r"C:\in\Docs\{stem}\[final\]");
        assert_eq!(
            pipeline.segments,
            vec![
                TemplateSegment::Literal {
                    value: r"C:\in\Docs\".into()
                },
                TemplateSegment::Token {
                    name: "stem".into()
                },
                TemplateSegment::Literal {
                    value: "[final]".into()
                },
            ]
        );
    }

    #[test]
    fn round_trips_literals_with_brackets_and_preset_like_slashes() {
        let pipeline = TemplatePipeline {
            segments: vec![
                TemplateSegment::Literal {
                    value: "relatorio [final]/remove_digits/".into(),
                },
                TemplateSegment::Token {
                    name: "stem".into(),
                },
            ],
        };
        let display = format_template(&pipeline);
        assert_eq!(parse(&display), pipeline);
    }

    #[test]
    fn escapes_literal_braces_with_doubling() {
        let pipeline = TemplatePipeline {
            segments: vec![TemplateSegment::Literal {
                value: "{projectName}".into(),
            }],
        };
        assert_eq!(format_template(&pipeline), "{{projectName}}");
        assert_eq!(parse("{{projectName}}"), pipeline);
    }

    #[test]
    fn round_trips_typical_path_template() {
        let pipeline = parse("{year}/{month}/docs/{stem}{ext}");
        assert_eq!(parse(&format_template(&pipeline)), pipeline);
    }
}

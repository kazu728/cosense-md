//! Inline classification: scan a line once, turn each `[...]` span into a typed
//! `Inline` node, and wrap bare URLs in the plain-text runs between them.
//!
//! There is no ordered list of regex substitutions and no negative-lookahead
//! enumeration: a single left-to-right pass dispatches on the bracket contents,
//! so rules cannot re-match one another's output and unknown spans fall through
//! verbatim as `Inline::Unknown`.

use crate::ast::{Emphasis, Inline};
use regex::Regex;
use std::sync::LazyLock;

pub const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "gif", "svg", "webp"];

static URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"https?://[^\s<>"'\]]+"#).unwrap());

pub fn parse_inline(text: &str) -> Vec<Inline> {
    let mut result = Vec::new();
    let mut plain = String::new();
    let mut rest = text;

    while !rest.is_empty() {
        if rest.starts_with('[') {
            if let Some((inline, consumed)) = try_bracket(rest) {
                push_plain(&mut plain, &mut result);
                result.push(inline);
                rest = &rest[consumed..];
                continue;
            }
        }
        match rest.chars().next() {
            Some(ch) => {
                plain.push(ch);
                rest = &rest[ch.len_utf8()..];
            }
            None => break,
        }
    }
    push_plain(&mut plain, &mut result);
    result
}

/// Flush accumulated literal text, wrapping bare `http(s)` URLs in `<...>`.
/// A URL already preceded by `<` or `(` is left alone (it is inside an existing
/// autolink or markdown link).
fn push_plain(plain: &mut String, out: &mut Vec<Inline>) {
    if plain.is_empty() {
        return;
    }
    let text = std::mem::take(plain);
    let mut last = 0;
    for m in URL_RE.find_iter(&text) {
        let preceded = text[..m.start()].chars().next_back();
        if matches!(preceded, Some('<') | Some('(')) {
            continue;
        }
        if m.start() > last {
            out.push(Inline::Text(text[last..m.start()].to_string()));
        }
        out.push(Inline::AutoLink(m.as_str().to_string()));
        last = m.end();
    }
    if last < text.len() {
        out.push(Inline::Text(text[last..].to_string()));
    }
}

/// Parse one bracket span starting at `rest[0] == '['`. Returns the node and the
/// number of bytes consumed, or `None` when the `[` is not the start of a span
/// we own: no closing `]`, an inner that crosses a line boundary (Cosense is
/// line-oriented, so a bracket never spans lines), or an existing markdown link
/// `[label](url)`.
fn try_bracket(rest: &str) -> Option<(Inline, usize)> {
    if let Some(after) = rest.strip_prefix("[[") {
        if let Some(close) = after.find("]]") {
            let inner = &after[..close];
            if !inner.contains('\n') {
                return Some((
                    Inline::Emphasis(Emphasis {
                        bold: true,
                        italic: false,
                        strike: false,
                        text: inner.to_string(),
                    }),
                    close + 4, // "[[" + inner + "]]"
                ));
            }
        }
    }

    let after_open = &rest[1..];
    let close = after_open.find(']')?;
    let inner = &after_open[..close];
    if inner.contains('\n') {
        return None;
    }
    let consumed = close + 2; // "[" + inner + "]"

    if rest[consumed..].starts_with('(') {
        return None;
    }
    Some((classify_bracket(inner), consumed))
}

fn classify_bracket(inner: &str) -> Inline {
    if let Some(emphasis) = decoration(inner) {
        return Inline::Emphasis(emphasis);
    }
    if let Some(image) = image_directive(inner) {
        return image;
    }
    if let Some(link) = url_bracket(inner) {
        return link;
    }
    if is_tag(inner) {
        return Inline::Tag(inner.to_string());
    }
    Inline::Unknown(format!("[{inner}]"))
}

/// `[* x]` `[** x]` (bold), `[*/ x]` `[*- x]` `[/- x]` `[/ x]` `[- x]` — a
/// decoration marker must be followed by whitespace, which is what keeps
/// `[/project/page]` from being read as italic.
fn decoration(inner: &str) -> Option<Emphasis> {
    // Bold is a run of one or more `*` (Cosense scales the size by count; Markdown
    // has a single bold), so it does not fit the fixed-string table below. Combos
    // like `*/` are left to the table: their leading `*` is followed by `/`, not
    // whitespace, so this rule never claims them.
    let stars = inner.chars().take_while(|&c| c == '*').count();
    if stars > 0 {
        let after = &inner[stars..];
        if after.starts_with(|c: char| c.is_whitespace()) {
            return Some(Emphasis {
                bold: true,
                italic: false,
                strike: false,
                text: after.trim_start().to_string(),
            });
        }
    }

    const MARKERS: [(&str, bool, bool, bool); 5] = [
        ("*/", true, true, false),
        ("*-", true, false, true),
        ("/-", false, true, true),
        ("/", false, true, false),
        ("-", false, false, true),
    ];
    for (marker, bold, italic, strike) in MARKERS {
        if let Some(after) = inner.strip_prefix(marker) {
            if after.starts_with(|c: char| c.is_whitespace()) {
                return Some(Emphasis {
                    bold,
                    italic,
                    strike,
                    text: after.trim_start().to_string(),
                });
            }
        }
    }
    None
}

fn image_directive(inner: &str) -> Option<Inline> {
    let after = inner.strip_prefix("img")?;
    if !after.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let url = after.trim();
    is_http_url(url).then(|| Inline::Image {
        alt: "img".to_string(),
        url: url.to_string(),
    })
}

fn url_bracket(inner: &str) -> Option<Inline> {
    let parts: Vec<&str> = inner.split_whitespace().collect();
    match parts.as_slice() {
        [single] if is_http_url(single) => {
            if is_image_url(single) {
                Some(Inline::Image {
                    alt: String::new(),
                    url: (*single).to_string(),
                })
            } else {
                Some(Inline::AutoLink((*single).to_string()))
            }
        }
        [first, rest @ ..] if !rest.is_empty() => {
            if *first == "YouTube" {
                if let Some(url) = parts.iter().find(|p| is_http_url(p) && is_youtube(p)) {
                    return Some(Inline::Link {
                        label: "YouTube Video".to_string(),
                        url: (*url).to_string(),
                    });
                }
            }
            if *first == "Twitter" {
                if let Some(url) = parts.iter().find(|p| is_http_url(p) && is_twitter(p)) {
                    return Some(Inline::Link {
                        label: "Twitter Post".to_string(),
                        url: (*url).to_string(),
                    });
                }
            }

            let last = parts[parts.len() - 1];
            if is_http_url(last) {
                let label = parts[..parts.len() - 1].join(" ");
                if is_valid_label(&label) {
                    return Some(Inline::Link {
                        label,
                        url: last.to_string(),
                    });
                }
            }
            if is_http_url(first) {
                let label = parts[1..].join(" ");
                if is_valid_label(&label) {
                    return Some(Inline::Link {
                        label,
                        url: (*first).to_string(),
                    });
                }
            }
            None
        }
        _ => None,
    }
}

fn is_http_url(s: &str) -> bool {
    (s.starts_with("http://") || s.starts_with("https://")) && !s.chars().any(|c| c.is_whitespace())
}

fn is_image_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    IMAGE_EXTENSIONS
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn is_youtube(url: &str) -> bool {
    url.contains("youtube.com/watch") || url.contains("youtu.be/")
}

fn is_twitter(url: &str) -> bool {
    (url.contains("twitter.com/") || url.contains("x.com/")) && url.contains("/status/")
}

/// A link label must not contain the characters the reverse/forward link rules
/// exclude (`/ - * ]`), matching the original renderer's label class.
fn is_valid_label(label: &str) -> bool {
    !label.is_empty() && !label.chars().any(|c| matches!(c, '/' | '-' | '*' | ']'))
}

/// A bare page name of word characters, rendered as a `#name` tag. Names
/// containing `-`, `/`, `.`, `*`, whitespace, etc. fall through untouched.
fn is_tag(inner: &str) -> bool {
    !inner.is_empty() && inner.chars().all(|c| c.is_alphanumeric() || c == '_')
}

//! Tokenizer → block parser. Mirrors the two-phase design of the original
//! Python parser but is written for Rust: indentation and line slicing are done
//! by `char`, never by byte offset, so tab / full-width-space (U+3000) indents
//! and multibyte content never split a UTF-8 boundary.

use crate::ast::{Block, Document, ListItem};
use regex::Regex;
use std::sync::LazyLock;

static HEADING_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\[(\*{1,5})\s+(.*?)\]").unwrap());

const CODE_DIRECTIVE: &str = "code:";
const TABLE_DIRECTIVE: &str = "table:";
const FENCE: &str = "```";

/// Bracket-form markers a detector can spot without a full parse: the inline
/// decorations and the media directives. Heading brackets are covered by
/// `HEADING_RE`, and `code:` / `table:` by the directive constants above.
const DETECT_BRACKET_MARKERS: [&str; 8] = [
    "[*/",
    "[*-",
    "[/-",
    "[/ ",
    "[- ",
    "[img ",
    "[YouTube ",
    "[Twitter ",
];

/// Heuristic used by the MarkItDown binding to decide whether a stream is
/// Cosense. It lives beside the parser so the heading pattern and directive
/// markers have a single definition, rather than being restated in the Python
/// glue where they had drifted from the core's knowledge.
pub fn looks_like_cosense(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        !line.is_empty()
            && (HEADING_RE.is_match(line)
                || line.starts_with(CODE_DIRECTIVE)
                || line.starts_with(TABLE_DIRECTIVE)
                || DETECT_BRACKET_MARKERS
                    .iter()
                    .any(|marker| line.starts_with(marker)))
    })
}

#[derive(Debug, Clone)]
enum TokenKind {
    Blank,
    Heading { level: usize, text: String },
    Fence { language: String },
    CodeDirective { descriptor: String },
    TableDirective { title: Option<String> },
    Text,
}

#[derive(Debug, Clone)]
struct Token {
    raw: String,
    /// Number of leading whitespace *characters* (space, tab, U+3000).
    indent: usize,
    kind: TokenKind,
}

impl Token {
    fn is_blank(&self) -> bool {
        self.raw.trim().is_empty()
    }

    fn content(&self) -> &str {
        self.raw.trim()
    }
}

fn leading_ws(line: &str) -> usize {
    line.chars()
        .take_while(|c| matches!(c, ' ' | '\t' | '\u{3000}'))
        .count()
}

/// Substring starting at the `char_start`-th character (empty if out of range).
fn char_substr_from(s: &str, char_start: usize) -> &str {
    match s.char_indices().nth(char_start) {
        Some((byte, _)) => &s[byte..],
        None => "",
    }
}

fn first_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

pub fn parse(text: &str) -> Document {
    let tokens = tokenize(text);
    let mut blocks: Vec<Block> = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];
        match &token.kind {
            TokenKind::Blank => {
                flush_paragraph(&mut paragraph, &mut blocks);
                blocks.push(Block::BlankLine);
                i += 1;
            }
            TokenKind::Heading { level, text } => {
                flush_paragraph(&mut paragraph, &mut blocks);
                blocks.push(Block::Heading {
                    level: *level,
                    text: text.clone(),
                });
                i += 1;
            }
            TokenKind::Fence { language } => {
                flush_paragraph(&mut paragraph, &mut blocks);
                let (block, next) = parse_fenced_code(&tokens, i, language.clone());
                blocks.push(block);
                i = next;
            }
            TokenKind::CodeDirective { descriptor } => {
                flush_paragraph(&mut paragraph, &mut blocks);
                let (block, next) = parse_code_block(&tokens, i, descriptor.clone());
                blocks.push(block);
                i = next;
            }
            TokenKind::TableDirective { title } => {
                flush_paragraph(&mut paragraph, &mut blocks);
                let (block, next) = parse_table(&tokens, i, title.clone());
                if let Some(block) = block {
                    blocks.push(block);
                }
                i = next;
            }
            TokenKind::Text if token.indent > 0 => {
                flush_paragraph(&mut paragraph, &mut blocks);
                let (block, next) = parse_list(&tokens, i);
                blocks.push(block);
                i = next;
            }
            TokenKind::Text => {
                paragraph.push(token.raw.clone());
                i += 1;
            }
        }
    }
    flush_paragraph(&mut paragraph, &mut blocks);
    Document { blocks }
}

fn flush_paragraph(paragraph: &mut Vec<String>, blocks: &mut Vec<Block>) {
    if !paragraph.is_empty() {
        blocks.push(Block::Paragraph {
            text: paragraph.join("\n"),
        });
        paragraph.clear();
    }
}

fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for line in text.lines() {
        let raw = line.to_string();
        let indent = leading_ws(&raw);
        let stripped = char_substr_from(&raw, indent);

        if stripped.is_empty() {
            tokens.push(Token {
                raw,
                indent,
                kind: TokenKind::Blank,
            });
            continue;
        }
        if let Some(caps) = HEADING_RE.captures(&raw) {
            let level = caps.get(1).map_or(0, |m| m.as_str().chars().count());
            let heading = caps.get(2).map_or("", |m| m.as_str());
            let end = caps.get(0).map_or(0, |m| m.end());
            let trailing = &raw[end..];
            let text = format!("{}{}", heading.trim(), trailing).trim().to_string();
            tokens.push(Token {
                raw,
                indent,
                kind: TokenKind::Heading { level, text },
            });
            continue;
        }
        if let Some(rest) = stripped.strip_prefix(FENCE) {
            let language = rest.trim().to_string();
            tokens.push(Token {
                raw,
                indent,
                kind: TokenKind::Fence { language },
            });
            continue;
        }
        if let Some(rest) = stripped.strip_prefix(CODE_DIRECTIVE) {
            let descriptor = rest.trim().to_string();
            tokens.push(Token {
                raw,
                indent,
                kind: TokenKind::CodeDirective { descriptor },
            });
            continue;
        }
        if indent == 0 {
            if let Some(rest) = stripped.strip_prefix(TABLE_DIRECTIVE) {
                let trimmed = rest.trim();
                let title = (!trimmed.is_empty()).then(|| trimmed.to_string());
                tokens.push(Token {
                    raw,
                    indent,
                    kind: TokenKind::TableDirective { title },
                });
                continue;
            }
        }
        tokens.push(Token {
            raw,
            indent,
            kind: TokenKind::Text,
        });
    }
    tokens
}

fn parse_fenced_code(tokens: &[Token], start: usize, language: String) -> (Block, usize) {
    let start_token = &tokens[start];
    let base_indent = start_token.indent;
    let prefix = first_chars(&start_token.raw, base_indent);

    let mut collected = Vec::new();
    let mut i = start + 1;
    while i < tokens.len() {
        let current = &tokens[i];
        if matches!(current.kind, TokenKind::Fence { .. }) && current.indent == base_indent {
            i += 1;
            break;
        }
        if current.raw.starts_with(&prefix) {
            collected.push(char_substr_from(&current.raw, base_indent).to_string());
        } else {
            collected.push(current.raw.clone());
        }
        i += 1;
    }

    (
        Block::Code {
            language,
            lines: collected,
            indent: prefix,
        },
        i,
    )
}

fn parse_code_block(tokens: &[Token], start: usize, descriptor: String) -> (Block, usize) {
    let directive = &tokens[start];
    let base_indent = directive.indent;
    let indent = first_chars(&directive.raw, base_indent);

    let mut collected: Vec<String> = Vec::new();
    let mut i = start + 1;
    while i < tokens.len() {
        let current = &tokens[i];
        if matches!(
            current.kind,
            TokenKind::CodeDirective { .. } | TokenKind::TableDirective { .. }
        ) {
            break;
        }

        if current.is_blank() {
            let mut next = i + 1;
            while next < tokens.len() && tokens[next].is_blank() {
                next += 1;
            }
            if next >= tokens.len() {
                for token in &tokens[i..next] {
                    collected.push(token.raw.clone());
                }
                i = next;
                break;
            }
            let following = &tokens[next];
            if matches!(
                following.kind,
                TokenKind::CodeDirective { .. } | TokenKind::TableDirective { .. }
            ) || following.indent <= base_indent
            {
                break;
            }
            collected.push(current.raw.clone());
            i += 1;
            continue;
        }

        // A non-blank line belongs to the block only while it is indented
        // deeper than the directive. This is the single condition that keeps a
        // shallow line after the code from being swallowed.
        if current.indent > base_indent {
            collected.push(current.raw.clone());
            i += 1;
        } else {
            break;
        }
    }

    let lines = normalize_code_lines(&collected);
    if descriptor.to_lowercase() == "tex" {
        (Block::Math { lines, indent }, i)
    } else {
        (
            Block::Code {
                language: infer_language(&descriptor),
                lines,
                indent,
            },
            i,
        )
    }
}

fn parse_table(tokens: &[Token], start: usize, title: Option<String>) -> (Option<Block>, usize) {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut i = start + 1;
    while i < tokens.len() {
        let current = &tokens[i];
        if current.indent == 0 || current.is_blank() {
            break;
        }
        rows.push(split_cells(current.content()));
        i += 1;
    }

    if rows.is_empty() {
        return (None, i);
    }
    let header = rows.remove(0);
    (
        Some(Block::Table {
            title,
            header,
            rows,
        }),
        i,
    )
}

fn split_cells(content: &str) -> Vec<String> {
    content
        .split('\t')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn parse_list(tokens: &[Token], start: usize) -> (Block, usize) {
    let mut items: Vec<(usize, String)> = Vec::new();
    let mut i = start;
    while i < tokens.len() {
        let current = &tokens[i];
        if current.indent == 0 || current.is_blank() {
            break;
        }
        let depth = current.indent.saturating_sub(1);
        items.push((depth, current.content().to_string()));
        i += 1;
    }
    (
        Block::BulletList {
            items: build_list_tree(&items),
        },
        i,
    )
}

/// Nest items by comparing indentation depths: an item's children are the
/// following items whose depth is strictly greater, until depth returns.
fn build_list_tree(items: &[(usize, String)]) -> Vec<ListItem> {
    let mut pos = 0;
    build_children(items, &mut pos, -1)
}

fn build_children(
    items: &[(usize, String)],
    pos: &mut usize,
    parent_depth: isize,
) -> Vec<ListItem> {
    let mut result = Vec::new();
    while *pos < items.len() {
        let depth = items[*pos].0 as isize;
        if depth <= parent_depth {
            break;
        }
        let text = items[*pos].1.clone();
        *pos += 1;
        let children = build_children(items, pos, depth);
        result.push(ListItem { text, children });
    }
    result
}

fn normalize_code_lines(lines: &[String]) -> Vec<String> {
    let indent_width = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| leading_ws(line))
        .min();
    let Some(indent_width) = indent_width else {
        return Vec::new();
    };

    let mut normalized: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                char_substr_from(line, indent_width).to_string()
            }
        })
        .collect();
    while matches!(normalized.last(), Some(line) if line.is_empty()) {
        normalized.pop();
    }
    normalized
}

fn infer_language(descriptor: &str) -> String {
    if descriptor.is_empty() {
        return String::new();
    }
    match descriptor.rsplit_once('.') {
        Some((_, ext)) => ext.to_string(),
        None => descriptor.to_string(),
    }
}

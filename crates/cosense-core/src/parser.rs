use crate::ast::{Block, Document, ItemContent, ListItem};

const CODE_DIRECTIVE: &str = "code:";
const TABLE_DIRECTIVE: &str = "table:";
const FENCE: &str = "```";

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

pub fn looks_like_cosense(text: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim();
        parse_heading(line).is_some()
            || line.starts_with(CODE_DIRECTIVE)
            || line.starts_with(TABLE_DIRECTIVE)
            || DETECT_BRACKET_MARKERS
                .iter()
                .any(|marker| line.starts_with(marker))
    })
}

fn parse_heading(line: &str) -> Option<(usize, &str, &str)> {
    let rest = line.strip_prefix('[')?;
    let level = rest.chars().take_while(|&c| c == '*').count();
    if !(1..=5).contains(&level) {
        return None;
    }
    let rest = &rest[level..];
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let (text, trailing) = rest.split_once(']')?;
    Some((level, text, trailing))
}

#[derive(Debug)]
enum TokenKind {
    Blank,
    Heading { level: usize, text: String },
    Fence { language: String },
    CodeDirective { descriptor: String },
    TableDirective { title: Option<String> },
    Text,
}

#[derive(Debug)]
struct Token {
    raw: String,
    indent: usize,
    kind: TokenKind,
}

impl Token {
    fn is_blank(&self) -> bool {
        matches!(self.kind, TokenKind::Blank)
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
            TokenKind::TableDirective { title } => match parse_table(&tokens, i, title.clone()) {
                Some((block, next)) => {
                    flush_paragraph(&mut paragraph, &mut blocks);
                    blocks.push(block);
                    i = next;
                }
                None => {
                    paragraph.push(token.raw.clone());
                    i += 1;
                }
            },
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
            lines: std::mem::take(paragraph),
        });
    }
}

fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for line in text.lines() {
        let raw = line.to_string();
        let indent = leading_ws(&raw);
        let stripped = char_substr_from(&raw, indent);

        if stripped.trim().is_empty() {
            tokens.push(Token {
                raw,
                indent,
                kind: TokenKind::Blank,
            });
            continue;
        }
        if let Some((level, heading, trailing)) = parse_heading(&raw) {
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
        if indent == 0
            && let Some(rest) = stripped.strip_prefix(TABLE_DIRECTIVE)
        {
            let trimmed = rest.trim();
            let title = (!trimmed.is_empty()).then(|| trimmed.to_string());
            tokens.push(Token {
                raw,
                indent,
                kind: TokenKind::TableDirective { title },
            });
            continue;
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
    let (lines, indent, next) = collect_code_body(tokens, start);
    let block = if is_math_descriptor(&descriptor) {
        Block::Math { lines, indent }
    } else {
        Block::Code {
            language: infer_language(&descriptor),
            lines,
            indent,
        }
    };
    (block, next)
}

fn collect_code_body(tokens: &[Token], start: usize) -> (Vec<String>, String, usize) {
    let directive = &tokens[start];
    let base_indent = directive.indent;
    let indent = first_chars(&directive.raw, base_indent);

    let mut collected: Vec<String> = Vec::new();
    let mut i = start + 1;
    while let Some(offset) = tokens[i..].iter().position(|t| !t.is_blank())
        && tokens[i + offset].indent > base_indent
    {
        let end = i + offset + 1;
        collected.extend(tokens[i..end].iter().map(|t| t.raw.clone()));
        i = end;
    }

    (normalize_code_lines(&collected), indent, i)
}

fn parse_table(tokens: &[Token], start: usize, title: Option<String>) -> Option<(Block, usize)> {
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

    let mut rows = rows.into_iter();
    let header = rows.next()?;
    Some((
        Block::Table {
            title,
            header,
            rows: rows.collect(),
        },
        i,
    ))
}

fn split_cells(content: &str) -> Vec<String> {
    content
        .split('\t')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn parse_list(tokens: &[Token], start: usize) -> (Block, usize) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut ancestors: Vec<usize> = Vec::new();
    let mut i = start;
    while i < tokens.len() {
        let current = &tokens[i];
        if current.indent == 0 || current.is_blank() {
            break;
        }
        while ancestors.last().is_some_and(|&d| d >= current.indent) {
            ancestors.pop();
        }
        let level = ancestors.len();
        if let TokenKind::CodeDirective { descriptor } = &current.kind {
            let (lines, _, next) = collect_code_body(tokens, i);
            let content = if is_math_descriptor(descriptor) {
                ItemContent::Math { lines }
            } else {
                ItemContent::Code {
                    language: infer_language(descriptor),
                    lines,
                }
            };
            items.push(ListItem { level, content });
            i = next;
            continue;
        }
        items.push(ListItem {
            level,
            content: ItemContent::Text(current.content().to_string()),
        });
        ancestors.push(current.indent);
        i += 1;
    }
    (Block::BulletList { items }, i)
}

fn normalize_code_lines(lines: &[String]) -> Vec<String> {
    let indent_width = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| leading_ws(line))
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                char_substr_from(line, indent_width).to_string()
            }
        })
        .collect()
}

fn is_math_descriptor(descriptor: &str) -> bool {
    descriptor.eq_ignore_ascii_case("tex")
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

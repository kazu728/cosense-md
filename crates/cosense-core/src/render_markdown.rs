use crate::ast::{Block, Document, Emphasis, Inline, ItemContent, ListItem};
use crate::inline::parse_inline;

pub fn render_markdown(document: &Document) -> String {
    let mut lines: Vec<String> = Vec::new();
    for block in &document.blocks {
        lines.extend(render_block(block));
    }
    lines.join("\n").trim_matches('\n').to_string()
}

fn render_block(block: &Block) -> Vec<String> {
    match block {
        Block::Heading { level, text } => {
            let heading = render_inline(text);
            vec![
                format!("{} {}", "#".repeat(*level), heading)
                    .trim_end()
                    .to_string(),
            ]
        }
        Block::Paragraph { lines } => lines.iter().map(|line| render_inline(line)).collect(),
        Block::BlankLine => vec![String::new()],
        Block::Code { language, lines } => render_code(language, lines, ""),
        Block::Math { lines } => render_math(lines, ""),
        Block::BulletList { items } => render_list(items),
        Block::Table {
            title,
            header,
            rows,
        } => render_table(title.as_deref(), header, rows),
    }
}

fn render_code(language: &str, lines: &[String], indent: &str) -> Vec<String> {
    let fence = if language.is_empty() {
        "```".to_string()
    } else {
        format!("```{language}")
    };
    let mut body = vec![format!("{indent}{fence}")];
    body.extend(lines.iter().map(|line| format!("{indent}{line}")));
    body.push(format!("{indent}```"));
    body
}

fn render_math(lines: &[String], indent: &str) -> Vec<String> {
    let mut body = vec![format!("{indent}$$")];
    body.extend(lines.iter().map(|line| format!("{indent}{line}")));
    body.push(format!("{indent}$$"));
    body
}

fn render_list(items: &[ListItem]) -> Vec<String> {
    let mut lines = Vec::new();
    for item in items {
        let indent = "  ".repeat(item.level);
        match &item.content {
            ItemContent::Text(text) => lines.push(format!("{indent}- {}", render_inline(text))),
            ItemContent::Code {
                language,
                lines: body,
            } => lines.extend(render_code(language, body, &indent)),
            ItemContent::Math { lines: body } => lines.extend(render_math(body, &indent)),
        }
    }
    lines
}

fn render_table(title: Option<&str>, header: &[String], rows: &[Vec<String>]) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(title) = title {
        lines.push(format!("## {}", render_inline(title)));
        lines.push(String::new());
    }

    let width = header
        .len()
        .max(rows.iter().map(Vec::len).max().unwrap_or(0));

    lines.push(render_row(header, width));
    lines.push(format!("|{}", "---|".repeat(width)));
    for row in rows {
        lines.push(render_row(row, width));
    }
    lines
}

fn render_row(cells: &[String], width: usize) -> String {
    let mut rendered: Vec<String> = cells.iter().map(|cell| render_inline(cell)).collect();
    while rendered.len() < width {
        rendered.push(String::new());
    }
    format!("| {} |", rendered.join(" | "))
}

fn render_inline(text: &str) -> String {
    let mut out = String::new();
    for node in parse_inline(text) {
        match node {
            Inline::Text(t) => out.push_str(&t),
            Inline::Emphasis(e) => out.push_str(&render_emphasis(&e)),
            Inline::Tag(name) => out.push_str(&format!("#{name}")),
            Inline::Image { alt, url } => out.push_str(&format!("![{alt}]({url})")),
            Inline::Link { label, url } => out.push_str(&format!("[{label}]({url})")),
            Inline::AutoLink(url) => out.push_str(&format!("<{url}>")),
        }
    }
    out
}

fn render_emphasis(emphasis: &Emphasis) -> String {
    let mut text = emphasis.text.clone();
    if emphasis.strike {
        text = format!("~~{text}~~");
    }
    if emphasis.italic {
        text = format!("*{text}*");
    }
    if emphasis.bold {
        text = format!("**{text}**");
    }
    text
}

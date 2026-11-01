use crate::ast::{Block, Emphasis, Inline, ItemContent, ListItem};
use crate::inline::parse_inline;

pub fn render_markdown(blocks: &[Block]) -> String {
    let lines: Vec<String> = blocks.iter().flat_map(render_block).collect();
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
        Block::Code { language, lines } => fenced(format!("```{language}"), lines, "```"),
        Block::Math { lines } => fenced("$$".to_string(), lines, "$$"),
        Block::BulletList { items } => render_list(items),
        Block::Table {
            title,
            header,
            rows,
        } => render_table(title.as_deref(), header, rows),
    }
}

fn fenced(open: String, lines: &[String], close: &str) -> Vec<String> {
    let mut body = vec![open];
    body.extend(lines.iter().cloned());
    body.push(close.to_string());
    body
}

fn render_list(items: &[ListItem]) -> Vec<String> {
    let mut lines = Vec::new();
    for item in items {
        let indent = "  ".repeat(item.level);
        match &item.content {
            ItemContent::Text(text) => lines.push(format!("{indent}- {}", render_inline(text))),
            ItemContent::Block(block) => lines.extend(
                render_block(block)
                    .into_iter()
                    .map(|line| format!("{indent}{line}")),
            ),
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
            Inline::Image(url) => out.push_str(&format!("![]({url})")),
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

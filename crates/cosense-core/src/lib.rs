mod inline;

const CODE_DIRECTIVE: &str = "code:";
const TABLE_DIRECTIVE: &str = "table:";
const FENCE: &str = "```";
const INDENT: [char; 3] = [' ', '\t', '\u{3000}'];

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
        heading(line).is_some()
            || line.starts_with(CODE_DIRECTIVE)
            || line.starts_with(TABLE_DIRECTIVE)
            || DETECT_BRACKET_MARKERS
                .iter()
                .any(|marker| line.starts_with(marker))
    })
}

pub fn convert(text: &str) -> String {
    let lines: Vec<Line> = text.lines().map(Line::new).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(line) = lines.get(i) {
        i = if line.is_blank() {
            out.push(String::new());
            i + 1
        } else if line.indent > 0 {
            list(&lines, i, &mut out)
        } else if let Some((level, text)) = heading(line.raw) {
            let heading = format!("{} {}", "#".repeat(level), inline::convert(&text));
            out.push(heading.trim_end().to_string());
            i + 1
        } else if let Some(language) = line.raw.strip_prefix(FENCE) {
            fenced_code(&lines, i, language.trim(), &mut out)
        } else if line.raw.starts_with(CODE_DIRECTIVE) {
            code_block(&lines, i, "", &mut out)
        } else if let Some(next) = line
            .raw
            .strip_prefix(TABLE_DIRECTIVE)
            .and_then(|title| table(&lines, i, title.trim(), &mut out))
        {
            next
        } else {
            out.push(inline::convert(line.raw));
            i + 1
        };
    }
    out.join("\n").trim_matches('\n').to_string()
}

struct Line<'a> {
    raw: &'a str,
    body: &'a str,
    indent: usize,
}

impl<'a> Line<'a> {
    fn new(raw: &'a str) -> Self {
        let body = raw.trim_start_matches(INDENT);
        let indent = raw[..raw.len() - body.len()].chars().count();
        Line { raw, body, indent }
    }

    fn is_blank(&self) -> bool {
        self.body.trim().is_empty()
    }
}

fn heading(line: &str) -> Option<(usize, String)> {
    let rest = line.strip_prefix('[')?;
    let level = rest.chars().take_while(|&c| c == '*').count();
    let rest = &rest[level..];
    if !(1..=5).contains(&level) || !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let (text, trailing) = rest.split_once(']')?;
    Some((
        level,
        format!("{}{trailing}", text.trim()).trim().to_string(),
    ))
}

fn list(lines: &[Line], start: usize, out: &mut Vec<String>) -> usize {
    let mut ancestors: Vec<usize> = Vec::new();
    let mut i = start;
    while let Some(line) = lines.get(i)
        && line.indent > 0
        && !line.is_blank()
    {
        while ancestors.last().is_some_and(|&d| d >= line.indent) {
            ancestors.pop();
        }
        let indent = "  ".repeat(ancestors.len());
        if line.body.starts_with(CODE_DIRECTIVE) {
            i = code_block(lines, i, &indent, out);
        } else {
            out.push(format!("{indent}- {}", inline::convert(line.body.trim())));
            ancestors.push(line.indent);
            i += 1;
        }
    }
    i
}

fn fenced_code(lines: &[Line], start: usize, language: &str, out: &mut Vec<String>) -> usize {
    let body: Vec<&str> = lines[start + 1..]
        .iter()
        .map(|line| line.raw)
        .take_while(|raw| !raw.starts_with(FENCE))
        .collect();
    push_code(out, "", language, &body);
    (start + body.len() + 2).min(lines.len())
}

fn code_block(lines: &[Line], start: usize, indent: &str, out: &mut Vec<String>) -> usize {
    let directive = &lines[start];
    let descriptor = directive.body[CODE_DIRECTIVE.len()..].trim();
    let mut end = start + 1;
    while let Some(offset) = lines[end..].iter().position(|line| !line.is_blank())
        && lines[end + offset].indent > directive.indent
    {
        end += offset + 1;
    }

    let body = &lines[start + 1..end];
    let width = body
        .iter()
        .filter(|line| !line.is_blank())
        .map(|line| line.indent)
        .min()
        .unwrap_or(0);
    let body: Vec<&str> = body
        .iter()
        .map(|line| match line.raw.char_indices().nth(width) {
            Some((byte, _)) if !line.is_blank() => &line.raw[byte..],
            _ => "",
        })
        .collect();

    if descriptor.eq_ignore_ascii_case("tex") {
        push_block(out, indent, "$$", &body, "$$");
    } else {
        let language = descriptor
            .rsplit_once('.')
            .map_or(descriptor, |(_, ext)| ext);
        push_code(out, indent, language, &body);
    }
    end
}

fn push_code(out: &mut Vec<String>, indent: &str, language: &str, body: &[&str]) {
    let longest_run = body
        .iter()
        .map(|line| {
            let line = line.trim_start();
            line.len() - line.trim_start_matches('`').len()
        })
        .max()
        .unwrap_or(0);
    let fence = "`".repeat(longest_run.max(2) + 1);
    push_block(out, indent, &format!("{fence}{language}"), body, &fence);
}

fn push_block(out: &mut Vec<String>, indent: &str, open: &str, body: &[&str], close: &str) {
    out.push(format!("{indent}{open}"));
    out.extend(body.iter().map(|line| format!("{indent}{line}")));
    out.push(format!("{indent}{close}"));
}

fn table(lines: &[Line], start: usize, title: &str, out: &mut Vec<String>) -> Option<usize> {
    let rows: Vec<Vec<&str>> = lines[start + 1..]
        .iter()
        .take_while(|line| line.indent > 0 && !line.is_blank())
        .map(|line| line.body.trim().split('\t').map(str::trim).collect())
        .collect();
    let width = rows.iter().map(Vec::len).max()?;

    if !title.is_empty() {
        out.push(format!("## {}", inline::convert(title)));
        out.push(String::new());
    }
    out.push(table_row(&rows[0], width));
    out.push(format!("|{}", "---|".repeat(width)));
    out.extend(rows[1..].iter().map(|row| table_row(row, width)));
    Some(start + 1 + rows.len())
}

fn table_row(cells: &[&str], width: usize) -> String {
    let mut cells: Vec<String> = cells
        .iter()
        .map(|cell| inline::convert(cell).replace('|', "\\|"))
        .collect();
    cells.resize(width, String::new());
    format!("| {} |", cells.join(" | "))
}

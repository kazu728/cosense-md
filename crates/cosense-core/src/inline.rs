const IMAGE_EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "gif", "svg", "webp"];

pub fn convert(line: &str) -> String {
    let mut out = String::new();
    let mut plain = 0;
    let mut search = 0;
    while let Some(offset) = line[search..].find('[') {
        let start = search + offset;
        match bracket(&line[start..]) {
            Some((markdown, len)) => {
                push_autolinked(&line[plain..start], &mut out);
                out.push_str(&markdown);
                plain = start + len;
                search = plain;
            }
            None => search = start + 1,
        }
    }
    push_autolinked(&line[plain..], &mut out);
    out
}

fn push_autolinked(text: &str, out: &mut String) {
    let mut last = 0;
    let mut from = 0;
    while let Some((start, end)) = next_url(text, from) {
        from = end;
        if text[..start].ends_with(['<', '(']) {
            continue;
        }
        out.push_str(&text[last..start]);
        out.push_str(&format!("<{}>", &text[start..end]));
        last = end;
    }
    out.push_str(&text[last..]);
}

fn next_url(text: &str, from: usize) -> Option<(usize, usize)> {
    text[from..].match_indices("http").find_map(|(offset, _)| {
        let start = from + offset;
        let rest = &text[start..];
        let scheme = ["https://", "http://"]
            .iter()
            .find(|scheme| rest.starts_with(*scheme))?
            .len();
        let len = rest[scheme..]
            .find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'' | ']'))
            .unwrap_or(rest.len() - scheme);
        (len > 0).then_some((start, start + scheme + len))
    })
}

fn bracket(rest: &str) -> Option<(String, usize)> {
    if let Some(after) = rest.strip_prefix("[[")
        && let Some(close) = after.find("]]")
    {
        return Some((format!("**{}**", &after[..close]), close + 4));
    }

    let len = rest.find(']')? + 1;
    if rest[len..].starts_with('(') {
        return None;
    }
    let inner = &rest[1..len - 1];
    let markdown = decoration(inner)
        .or_else(|| link(inner))
        .or_else(|| is_tag(inner).then(|| format!("#{inner}")))
        .unwrap_or_else(|| rest[..len].to_string());
    Some((markdown, len))
}

fn decoration(inner: &str) -> Option<String> {
    let (marker, text) = inner.split_once(char::is_whitespace)?;
    let (open, close) = match marker {
        "/" => ("*", "*"),
        "-" => ("~~", "~~"),
        "*/" => ("***", "***"),
        "*-" => ("**~~", "~~**"),
        "/-" => ("*~~", "~~*"),
        _ if !marker.is_empty() && marker.trim_start_matches('*').is_empty() => ("**", "**"),
        _ => return None,
    };
    Some(format!("{open}{}{close}", text.trim_start()))
}

fn link(inner: &str) -> Option<String> {
    let parts: Vec<&str> = inner.split_whitespace().collect();
    match parts.as_slice() {
        [url] if is_url(url) => Some(if is_image(url) {
            format!("![]({url})")
        } else {
            format!("<{url}>")
        }),
        ["img", url] if is_url(url) => Some(format!("![img]({url})")),
        [first, .., last] => {
            if *first == "YouTube"
                && let Some(url) = parts.iter().find(|part| is_youtube(part))
            {
                return Some(format!("[YouTube Video]({url})"));
            }
            if *first == "Twitter"
                && let Some(url) = parts.iter().find(|part| is_twitter(part))
            {
                return Some(format!("[Twitter Post]({url})"));
            }
            if is_url(last) {
                return Some(format!("[{}]({last})", parts[..parts.len() - 1].join(" ")));
            }
            is_url(first).then(|| format!("[{}]({first})", parts[1..].join(" ")))
        }
        _ => None,
    }
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn is_image(url: &str) -> bool {
    url.rsplit_once('.').is_some_and(|(_, ext)| {
        IMAGE_EXTENSIONS
            .iter()
            .any(|image| ext.eq_ignore_ascii_case(image))
    })
}

fn is_youtube(part: &str) -> bool {
    is_url(part) && (part.contains("youtube.com/watch") || part.contains("youtu.be/"))
}

fn is_twitter(part: &str) -> bool {
    is_url(part)
        && (part.contains("twitter.com/") || part.contains("x.com/"))
        && part.contains("/status/")
}

fn is_tag(inner: &str) -> bool {
    !inner.is_empty() && inner.chars().all(|c| c.is_alphanumeric() || c == '_')
}

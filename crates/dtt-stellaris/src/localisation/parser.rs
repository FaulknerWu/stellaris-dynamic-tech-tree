pub(super) fn parse_entries(src: &str) -> Vec<(String, String)> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    let mut entries = Vec::new();
    for line in src.lines() {
        if let Some((key, value)) = parse_entry_line(line) {
            entries.push((key, value));
        }
    }
    entries
}

fn parse_entry_line(line: &str) -> Option<(String, String)> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let colon = line.find(':')?;
    let key = line[..colon].trim();
    if key.is_empty() || key.starts_with("l_") {
        return None;
    }

    let mut value = line[colon + 1..].trim_start();
    let version_len = value
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .map(char::len_utf8)
        .sum();
    value = value[version_len..].trim_start();
    if value.is_empty() {
        return None;
    }

    let value = if let Some(rest) = value.strip_prefix('"') {
        let end = find_closing_quote(rest).unwrap_or(rest.len());
        rest[..end].to_string()
    } else {
        strip_comment(value).trim().to_string()
    };

    Some((key.to_string(), value))
}

fn find_closing_quote(s: &str) -> Option<usize> {
    let mut backslashes = 0usize;
    for (index, ch) in s.char_indices() {
        match ch {
            '\\' => backslashes += 1,
            '"' if backslashes.is_multiple_of(2) => return Some(index),
            _ => backslashes = 0,
        }
    }
    None
}

fn strip_comment(s: &str) -> &str {
    match s.find('#') {
        Some(index) => &s[..index],
        None => s,
    }
}

pub(super) fn normalise_whitespace(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            in_space = true;
        } else {
            if in_space && !out.is_empty() {
                out.push(' ');
            }
            out.push(ch);
            in_space = false;
        }
    }
    out
}

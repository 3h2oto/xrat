use super::LogLine;

#[derive(Debug, PartialEq)]
pub struct ParsedLine {
    pub timestamp: String,
    pub normalized_time: Option<String>,
    pub level: String,
    pub message: String,
    pub raw: String,
}
pub fn severity(value: &str) -> Option<&'static str> {
    let value = value.trim_matches(['[', ']', ':']).to_ascii_lowercase();
    let value = value.split('[').next().unwrap_or(&value);
    match value {
        "fatal" | "panic" | "error" | "err" => Some("error"),
        "warn" | "warning" => Some("warn"),
        "info" | "information" => Some("info"),
        "debug" | "dbg" => Some("debug"),
        "trace" => Some("trace"),
        _ => None,
    }
}
pub fn strip_ansi(input: &str) -> String {
    let mut output = String::new();
    let mut mode = 0;
    for ch in input.chars() {
        match mode {
            0 => match ch {
                '\u{1b}' => mode = 1,
                ch if !ch.is_control() || ch == '\t' => output.push(ch),
                _ => {}
            },
            1 => {
                mode = match ch {
                    '[' => 2,
                    ']' => 3,
                    _ => 0,
                }
            }
            2 => {
                if ('@'..='~').contains(&ch) {
                    mode = 0;
                }
            }
            3 => match ch {
                '\u{7}' => mode = 0,
                '\u{1b}' => mode = 4,
                _ => {}
            },
            _ => mode = if ch == '\\' { 0 } else { 3 },
        }
    }
    output
}
fn date_token(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[..4].iter().all(u8::is_ascii_digit)
        && matches!(value.as_bytes()[4], b'-' | b'/')
        && matches!(value.as_bytes()[7], b'-' | b'/')
}
fn offset(value: &str) -> Option<String> {
    if value.len() == 5
        && value.starts_with(['+', '-'])
        && value.as_bytes()[1..].iter().all(u8::is_ascii_digit)
    {
        Some(format!("{}:{}", &value[..3], &value[3..]))
    } else if value.len() == 6
        && value.starts_with(['+', '-'])
        && value.as_bytes()[3] == b':'
        && value.as_bytes()[1..3]
            .iter()
            .chain(value.as_bytes()[4..].iter())
            .all(u8::is_ascii_digit)
    {
        Some(value.into())
    } else {
        None
    }
}
pub fn parse_engine_line(input: &str) -> ParsedLine {
    let raw = strip_ansi(input);
    let clean = raw.trim();
    let tokens = clean.split_whitespace().take(4).collect::<Vec<_>>();
    let mut consumed = 0;
    let mut timestamp = String::new();
    let mut normalized = None;
    if let Some(first) = tokens.first() {
        let first = first.trim_matches(['[', ']']);
        if first.len() >= 19 && first.as_bytes().get(10) == Some(&b'T') && date_token(&first[..10])
        {
            timestamp = first.into();
            normalized = Some(first.replace('/', "-"));
            consumed = 1;
        } else {
            let (start, zone) = offset(first)
                .map(|zone| (1, Some(zone)))
                .unwrap_or((0, None));
            if let (Some(date), Some(time)) = (tokens.get(start), tokens.get(start + 1))
                && date_token(date)
                && time.as_bytes().get(2) == Some(&b':')
                && time.as_bytes().get(5) == Some(&b':')
            {
                timestamp = tokens[..start + 2].join(" ");
                normalized = Some(format!(
                    "{}T{}{}",
                    date.replace('/', "-"),
                    time.trim_end_matches(']'),
                    zone.unwrap_or_default()
                ));
                consumed = start + 2;
            }
        }
    }
    let mut remainder = clean;
    for _ in 0..consumed {
        remainder = remainder
            .split_once(char::is_whitespace)
            .map(|(_, rest)| rest.trim_start())
            .unwrap_or_default();
    }
    let token = remainder.split_whitespace().next().unwrap_or_default();
    let level = severity(token).unwrap_or("unknown").to_string();
    if level != "unknown" {
        remainder = remainder
            .strip_prefix(token)
            .unwrap_or(remainder)
            .trim_start();
    }
    ParsedLine {
        timestamp,
        normalized_time: normalized,
        level,
        message: remainder.into(),
        raw,
    }
}
pub fn continuation(input: &str) -> bool {
    input.starts_with(char::is_whitespace)
        || input.starts_with("goroutine ")
        || input.starts_with("stack backtrace:")
        || input.starts_with("Caused by:")
        || input.starts_with("at ")
}
pub fn engine_lines(content: &str, source: &str) -> Vec<LogLine> {
    let mut rows: Vec<LogLine> = Vec::new();
    for input in content.lines() {
        let parsed = parse_engine_line(input);
        if parsed.normalized_time.is_none()
            && parsed.level == "unknown"
            && (continuation(&parsed.raw) || parsed.raw.is_empty())
            && let Some(previous) = rows.last_mut()
        {
            previous.message.push('\n');
            previous.message.push_str(&parsed.raw);
            previous.text.push('\n');
            previous.text.push_str(&parsed.raw);
            continue;
        }
        if parsed.raw.is_empty() {
            continue;
        }
        rows.push(LogLine {
            time: parsed
                .normalized_time
                .as_deref()
                .and_then(super::super::history::parse_time),
            timestamp: parsed.timestamp,
            level: parsed.level,
            source: source_label(source).into(),
            message: parsed.message,
            text: format!("[{source}] {}", parsed.raw),
        });
    }
    rows.reverse();
    rows
}

fn source_label(source: &str) -> &str {
    match source.rsplit(" · ").next().unwrap_or(source) {
        "out.log" => "stdout",
        "err.log" => "stderr",
        "singbox.out.log" => "sing-box stdout",
        "singbox.err.log" => "sing-box stderr",
        _ => source,
    }
}

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::tui::app::{TuiApp, TuiLogTab};
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, render_scroll_panel};

const EVENT_TIME_WIDTH: usize = 19;
const EVENT_LEVEL_WIDTH: usize = 5;
const EVENT_SOURCE_WIDTH: usize = 12;
const EVENT_KIND_WIDTH: usize = 16;
const COLUMN_GAP: usize = 2;

const EVENT_PREFIX_WIDTH: usize = EVENT_TIME_WIDTH
    + COLUMN_GAP
    + EVENT_LEVEL_WIDTH
    + COLUMN_GAP
    + EVENT_SOURCE_WIDTH
    + COLUMN_GAP
    + EVENT_KIND_WIDTH
    + COLUMN_GAP;

const API_METHOD_WIDTH: usize = 7;
const API_PATH_WIDTH: usize = 20;
const API_CODE_WIDTH: usize = 4;
const API_PREFIX_WIDTH: usize = EVENT_TIME_WIDTH
    + COLUMN_GAP
    + EVENT_LEVEL_WIDTH
    + COLUMN_GAP
    + API_METHOD_WIDTH
    + COLUMN_GAP
    + API_PATH_WIDTH
    + COLUMN_GAP
    + API_CODE_WIDTH
    + COLUMN_GAP;

const PROXY_TIME_WIDTH: usize = 19;
const PROXY_LEVEL_WIDTH: usize = 7;
const PROXY_SOURCE_WIDTH: usize = 16;
const PROXY_PREFIX_WIDTH: usize = PROXY_TIME_WIDTH
    + COLUMN_GAP
    + PROXY_LEVEL_WIDTH
    + COLUMN_GAP
    + PROXY_SOURCE_WIDTH
    + COLUMN_GAP;

const MIN_INLINE_MESSAGE_WIDTH: usize = 16;
const SUBROW_INDENT: usize = 2;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    if app.active_log_tab == TuiLogTab::Stats {
        super::stats::render(
            frame,
            area,
            app,
            log_title(TuiLogTab::Stats, active_engine_label(app)),
            focused,
        );
        app.panel_viewport.log.set(area.height.saturating_sub(2));
        return;
    }

    let content_width = area.width.saturating_sub(2) as usize;
    let lines = match app.active_log_tab {
        TuiLogTab::XratEvents => event_lines(app, content_width),
        TuiLogTab::ProxyEngine => proxy_lines(app, content_width),
        TuiLogTab::Api => api_lines(app, content_width),
        TuiLogTab::Stats => unreachable!("stats tab handled above"),
    };

    let viewport = render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.log,
        PanelStyle {
            title: log_title(app.active_log_tab, active_engine_label(app)),
            focused,
            right_pad: 0,
            wrap_trim: false,
            header: log_header(app.active_log_tab, content_width),
        },
    );
    app.panel_viewport.log.set(viewport);
}

fn event_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
    let events: Vec<_> = app
        .data
        .logs
        .events
        .iter()
        .filter(|event| app.event_visible(event.id))
        .collect();
    if events.is_empty() {
        return vec![Line::styled(
            "No xrat events recorded yet.",
            theme::muted_style(),
        )];
    }

    let mut lines = Vec::new();
    for event in events {
        let prefix = vec![
            Span::styled(
                format!("{:<EVENT_TIME_WIDTH$}", compact_time(&event.time)),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<EVENT_LEVEL_WIDTH$}",
                    truncate(&event.level, EVENT_LEVEL_WIDTH)
                ),
                theme::severity_style(&event.level),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<EVENT_SOURCE_WIDTH$}",
                    truncate(&event.source, EVENT_SOURCE_WIDTH)
                ),
                theme::secondary_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<EVENT_KIND_WIDTH$}",
                    truncate(&event.kind, EVENT_KIND_WIDTH)
                ),
                theme::chrome_style(),
            ),
            Span::raw("  "),
        ];
        push_wrapped_row(
            &mut lines,
            prefix,
            EVENT_PREFIX_WIDTH,
            &event.message,
            content_width,
        );
    }
    lines
}

fn api_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
    let events: Vec<_> = app
        .data
        .logs
        .events
        .iter()
        .filter(|event| event.source == "api" && app.event_visible(event.id))
        .collect();
    if events.is_empty() {
        return vec![Line::styled(
            "No API requests recorded yet.",
            theme::muted_style(),
        )];
    }

    let mut lines = Vec::new();
    for event in events {
        let (method, path, code, message) = parse_api_request(&event.kind, &event.message);
        let prefix = vec![
            Span::styled(
                format!("{:<EVENT_TIME_WIDTH$}", compact_time(&event.time)),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<EVENT_LEVEL_WIDTH$}",
                    truncate(&event.level, EVENT_LEVEL_WIDTH)
                ),
                theme::severity_style(&event.level),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<API_METHOD_WIDTH$}", truncate(&method, API_METHOD_WIDTH)),
                theme::secondary_style().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<API_PATH_WIDTH$}", truncate(&path, API_PATH_WIDTH)),
                theme::chrome_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<API_CODE_WIDTH$}", truncate(&code, API_CODE_WIDTH)),
                theme::severity_style(&event.level),
            ),
            Span::raw("  "),
        ];
        push_wrapped_row(
            &mut lines,
            prefix,
            API_PREFIX_WIDTH,
            &message,
            content_width,
        );
    }
    lines
}

/// Split a recorded API event into table columns. The server middleware records
/// each request as the synthetic line `"{method} {path} -> {code}"`, which is not
/// a real axum/handler message, so its MESSAGE column shows `-`. Anything that
/// does not match that shape is treated as an actual message and shown verbatim.
fn parse_api_request(kind: &str, message: &str) -> (String, String, String, String) {
    if let Some((request, code)) = message.rsplit_once(" -> ")
        && code.chars().all(|ch| ch.is_ascii_digit())
        && let Some((method, path)) = request.split_once(' ')
    {
        return (
            method.to_string(),
            path.to_string(),
            code.to_string(),
            "-".to_string(),
        );
    }
    (
        kind.to_string(),
        "-".to_string(),
        "-".to_string(),
        message.to_string(),
    )
}

fn proxy_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
    use crate::tui::data::ProxyStream;

    let visible: Vec<_> = visible_proxy_rows(app)
        .iter()
        .filter(|row| !row.is_self_api_traffic())
        .collect();
    if visible.is_empty() {
        return vec![Line::styled(
            "No proxy engine logs found for the latest runtime session.",
            theme::muted_style(),
        )];
    }

    let mut lines = Vec::new();
    for row in visible.iter().rev() {
        let source_style = if row.stream == ProxyStream::Stderr {
            theme::warning_style()
        } else {
            theme::muted_style()
        };
        let prefix = vec![
            Span::styled(
                format!(
                    "{:<PROXY_TIME_WIDTH$}",
                    row.time.as_deref().map(compact_time).unwrap_or_default()
                ),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<PROXY_LEVEL_WIDTH$}", row.level.as_deref().unwrap_or("")),
                proxy_severity_style(row),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<PROXY_SOURCE_WIDTH$}",
                    truncate(row.component.as_deref().unwrap_or(""), PROXY_SOURCE_WIDTH)
                ),
                source_style,
            ),
            Span::raw("  "),
        ];
        push_wrapped_row(
            &mut lines,
            prefix,
            PROXY_PREFIX_WIDTH,
            &row.message,
            content_width,
        );
    }
    lines
}

/// Proxy rows after the view-only clear boundary. With a clear signature set,
/// rows up to and including its most recent occurrence stay hidden; a rotated
/// log whose signature is gone shows everything again.
fn visible_proxy_rows(app: &TuiApp) -> &[crate::tui::data::TuiProxyLogRow] {
    let rows = &app.data.logs.proxy;
    let Some(signature) = &app.proxy_clear_signature else {
        return rows;
    };
    match rows.iter().rposition(|row| &row.signature() == signature) {
        Some(index) => &rows[index + 1..],
        None => rows,
    }
}

fn push_wrapped_row<'a>(
    lines: &mut Vec<Line<'a>>,
    mut prefix: Vec<Span<'a>>,
    prefix_width: usize,
    message: &str,
    content_width: usize,
) {
    let clean_message = message.trim();
    let has_meaningful_message = !clean_message.is_empty() && clean_message != "-";

    let inline_avail = content_width.saturating_sub(prefix_width);
    if inline_avail >= MIN_INLINE_MESSAGE_WIDTH {
        let chunks = if has_meaningful_message {
            wrap_message(clean_message, inline_avail)
        } else if clean_message == "-" {
            vec!["-".to_string()]
        } else {
            Vec::new()
        };

        if chunks.is_empty() {
            if let Some(Span { content, .. }) = prefix.last()
                && content.trim().is_empty()
            {
                prefix.pop();
            }
            lines.push(Line::from(prefix));
        } else {
            let mut first = prefix;
            first.push(Span::styled(chunks[0].clone(), theme::chrome_style()));
            lines.push(Line::from(first));

            for chunk in &chunks[1..] {
                lines.push(Line::from(vec![
                    Span::raw(" ".repeat(prefix_width)),
                    Span::styled(chunk.clone(), theme::chrome_style()),
                ]));
            }
        }
    } else {
        if let Some(Span { content, .. }) = prefix.last()
            && content.trim().is_empty()
        {
            prefix.pop();
        }
        lines.push(Line::from(prefix));

        if has_meaningful_message {
            let subrow_avail = content_width.saturating_sub(SUBROW_INDENT).max(1);
            let chunks = wrap_message(clean_message, subrow_avail);
            for chunk in chunks {
                lines.push(Line::from(vec![
                    Span::raw(" ".repeat(SUBROW_INDENT)),
                    Span::styled(chunk, theme::chrome_style()),
                ]));
            }
        }
    }
}

fn wrap_message(message: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![message.to_string()];
    }

    let mut tokens: Vec<String> = Vec::new();
    for word in message.split_whitespace() {
        if word.width() > width {
            tokens.extend(hard_break(word, width));
        } else {
            tokens.push(word.to_string());
        }
    }

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    for token in tokens {
        let token_width = token.width();
        let separator = usize::from(!current.is_empty());
        if !current.is_empty() && current_width + separator + token_width > width {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
        }
        if !current.is_empty() {
            current.push(' ');
            current_width += 1;
        }
        current.push_str(&token);
        current_width += token_width;
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

fn hard_break(word: &str, width: usize) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    for ch in word.chars() {
        let char_width = ch.to_string().width();
        if current_width + char_width > width && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(ch);
        current_width += char_width;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Severity color for a proxy engine row. Uses the parsed level when present;
/// otherwise infers severity from message keywords and stderr context so
/// unparseable lines still get a sensible color without hiding the raw text.
fn proxy_severity_style(row: &crate::tui::data::TuiProxyLogRow) -> ratatui::style::Style {
    use crate::tui::data::ProxyStream;

    if let Some(level) = row.level.as_deref().filter(|level| !level.is_empty()) {
        return theme::severity_style(level);
    }

    let message = row.message.to_ascii_lowercase();
    if message.contains("panic") || message.contains("fatal") || message.contains("error") {
        theme::failure_style()
    } else if message.contains("warn") || row.stream == ProxyStream::Stderr {
        theme::warning_style()
    } else {
        theme::muted_style()
    }
}

/// Column header pinned above the scrolling body for the tabular tabs. The
/// stats tab has no columns, so it returns `None`. When width is constrained,
/// the header omits the MESSAGE column to match subrow formatting.
fn log_header(tab: TuiLogTab, content_width: usize) -> Option<Line<'static>> {
    let text = match tab {
        TuiLogTab::XratEvents => {
            if content_width.saturating_sub(EVENT_PREFIX_WIDTH) >= MIN_INLINE_MESSAGE_WIDTH {
                format!(
                    "{:<EVENT_TIME_WIDTH$}  {:<EVENT_LEVEL_WIDTH$}  {:<EVENT_SOURCE_WIDTH$}  {:<EVENT_KIND_WIDTH$}  {}",
                    "TIME", "LEVEL", "SOURCE", "KIND", "MESSAGE"
                )
            } else {
                format!(
                    "{:<EVENT_TIME_WIDTH$}  {:<EVENT_LEVEL_WIDTH$}  {:<EVENT_SOURCE_WIDTH$}  {}",
                    "TIME", "LEVEL", "SOURCE", "KIND"
                )
            }
        }
        TuiLogTab::ProxyEngine => {
            if content_width.saturating_sub(PROXY_PREFIX_WIDTH) >= MIN_INLINE_MESSAGE_WIDTH {
                format!(
                    "{:<PROXY_TIME_WIDTH$}  {:<PROXY_LEVEL_WIDTH$}  {:<PROXY_SOURCE_WIDTH$}  {}",
                    "TIME", "LEVEL", "SOURCE", "MESSAGE"
                )
            } else {
                format!(
                    "{:<PROXY_TIME_WIDTH$}  {:<PROXY_LEVEL_WIDTH$}  {}",
                    "TIME", "LEVEL", "SOURCE"
                )
            }
        }
        TuiLogTab::Api => {
            if content_width.saturating_sub(API_PREFIX_WIDTH) >= MIN_INLINE_MESSAGE_WIDTH {
                format!(
                    "{:<EVENT_TIME_WIDTH$}  {:<EVENT_LEVEL_WIDTH$}  {:<API_METHOD_WIDTH$}  {:<API_PATH_WIDTH$}  {:<API_CODE_WIDTH$}  {}",
                    "TIME", "LEVEL", "METHOD", "PATH", "CODE", "MESSAGE"
                )
            } else {
                format!(
                    "{:<EVENT_TIME_WIDTH$}  {:<EVENT_LEVEL_WIDTH$}  {:<API_METHOD_WIDTH$}  {:<API_PATH_WIDTH$}  {}",
                    "TIME", "LEVEL", "METHOD", "PATH", "CODE"
                )
            }
        }
        TuiLogTab::Stats => return None,
    };
    Some(Line::styled(text, theme::muted_style()))
}

fn log_title(tab: TuiLogTab, engine_label: Option<String>) -> Line<'static> {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("3:", theme::accent_style().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
    ];
    for (index, (log_tab, label)) in [
        (TuiLogTab::XratEvents, "Events"),
        (TuiLogTab::ProxyEngine, "Engine"),
        (TuiLogTab::Stats, "Traffic"),
        (TuiLogTab::Api, "API"),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        if tab == log_tab {
            spans.push(Span::styled(
                format!("[{label}]"),
                theme::accent_style().add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(label.to_string(), theme::muted_style()));
        }
    }
    if matches!(tab, TuiLogTab::ProxyEngine | TuiLogTab::Stats)
        && let Some(engine_label) = engine_label
    {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("· {engine_label}"),
            theme::accent_style(),
        ));
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}

/// Engine name (and version, if probed) currently writing to the engine tab,
/// taken from the most recent proxy row. `None` when no engine logs are loaded.
fn active_engine_label(app: &TuiApp) -> Option<String> {
    let engine = app.data.logs.proxy.last().map(|row| row.engine.clone())?;
    let version = app
        .engines
        .iter()
        .find(|info| info.name == engine)
        .and_then(|info| info.version.clone());
    Some(match version {
        Some(version) => format!("{engine} {version}"),
        None => engine,
    })
}

/// Normalize a timestamp to `YYYY-MM-DD HH:MM:SS` (19 chars): drop a trailing
/// `Z`, swap the ISO `T`, map xray's `/` date separators to `-`, and trim any
/// sub-second fraction. Shared by every log tab so columns line up identically.
fn compact_time(value: &str) -> String {
    value
        .trim_end_matches('Z')
        .replace('T', " ")
        .replace('/', "-")
        .chars()
        .take(19)
        .collect()
}

fn truncate(value: &str, width: usize) -> String {
    if value.width() <= width {
        return value.to_string();
    }

    let mut result = String::new();
    let mut used = 0usize;
    for ch in value.chars() {
        let char_width = ch.to_string().width();
        if used + char_width > width.saturating_sub(1) {
            break;
        }
        result.push(ch);
        used += char_width;
    }
    result.push('…');
    result
}

#[cfg(test)]
mod tests {
    use super::{
        API_PREFIX_WIDTH, EVENT_PREFIX_WIDTH, MIN_INLINE_MESSAGE_WIDTH, PROXY_PREFIX_WIDTH,
        hard_break, log_header, push_wrapped_row, wrap_message,
    };
    use crate::tui::app::TuiLogTab;
    use ratatui::text::Span;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn wraps_message_on_word_boundaries_within_width() {
        let chunks = wrap_message("Reconnected config 3 after stale runtime PID", 20);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.width() <= 20, "chunk too wide: {chunk:?}");
        }
        assert_eq!(
            chunks.join(" "),
            "Reconnected config 3 after stale runtime PID"
        );
    }

    #[test]
    fn hard_breaks_words_longer_than_width() {
        let chunks = hard_break("daemon_restart_stale_pid_recovered", 10);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.width() <= 10);
        }
        assert_eq!(chunks.concat(), "daemon_restart_stale_pid_recovered");
    }

    #[test]
    fn wrap_returns_single_chunk_for_short_message() {
        assert_eq!(wrap_message("ok", 40), vec!["ok".to_string()]);
    }

    #[test]
    fn wrap_zero_width_returns_message_unchanged() {
        assert_eq!(wrap_message("anything", 0), vec!["anything".to_string()]);
    }

    #[test]
    fn push_wrapped_row_inline_when_width_is_sufficient() {
        let mut lines = Vec::new();
        let prefix = vec![Span::raw("PREFIX: "), Span::raw("  ")];
        let prefix_width = 10;
        let content_width = prefix_width + MIN_INLINE_MESSAGE_WIDTH + 10;

        push_wrapped_row(
            &mut lines,
            prefix,
            prefix_width,
            "Tested 8 configs",
            content_width,
        );

        assert_eq!(lines.len(), 1);
        let rendered = lines[0].to_string();
        assert!(rendered.starts_with("PREFIX: "));
        assert!(rendered.contains("Tested 8 configs"));
    }

    #[test]
    fn push_wrapped_row_subrow_when_width_is_narrow() {
        let mut lines = Vec::new();
        let prefix = vec![Span::raw("PREFIX: "), Span::raw("  ")];
        let prefix_width = 50;
        let content_width = 55; // remaining space is 5, less than MIN_INLINE_MESSAGE_WIDTH (16)

        push_wrapped_row(
            &mut lines,
            prefix,
            prefix_width,
            "Tested 8 configs (7 passed, 1 failed)",
            content_width,
        );

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].to_string(), "PREFIX: ");
        assert_eq!(
            lines[1].to_string(),
            "  Tested 8 configs (7 passed, 1 failed)"
        );
    }

    #[test]
    fn push_wrapped_row_omits_subrow_when_message_is_empty_or_dash_in_narrow_mode() {
        let mut lines = Vec::new();
        let prefix = vec![Span::raw("PREFIX: "), Span::raw("  ")];
        push_wrapped_row(&mut lines, prefix.clone(), 50, "-", 55);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].to_string(), "PREFIX: ");

        let mut lines = Vec::new();
        push_wrapped_row(&mut lines, prefix, 50, "   ", 55);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].to_string(), "PREFIX: ");
    }

    #[test]
    fn log_header_adapts_to_content_width() {
        // Wide: includes MESSAGE
        let wide_events = log_header(
            TuiLogTab::XratEvents,
            EVENT_PREFIX_WIDTH + MIN_INLINE_MESSAGE_WIDTH + 5,
        )
        .unwrap()
        .to_string();
        assert!(wide_events.contains("MESSAGE"));
        assert!(wide_events.contains("KIND"));

        // Narrow: omits MESSAGE
        let narrow_events = log_header(TuiLogTab::XratEvents, 50).unwrap().to_string();
        assert!(!narrow_events.contains("MESSAGE"));
        assert!(narrow_events.contains("KIND"));

        // ProxyEngine
        let wide_proxy = log_header(
            TuiLogTab::ProxyEngine,
            PROXY_PREFIX_WIDTH + MIN_INLINE_MESSAGE_WIDTH + 5,
        )
        .unwrap()
        .to_string();
        assert!(wide_proxy.contains("MESSAGE"));

        let narrow_proxy = log_header(TuiLogTab::ProxyEngine, 50).unwrap().to_string();
        assert!(!narrow_proxy.contains("MESSAGE"));
        assert!(narrow_proxy.contains("SOURCE"));

        // API
        let wide_api = log_header(
            TuiLogTab::Api,
            API_PREFIX_WIDTH + MIN_INLINE_MESSAGE_WIDTH + 5,
        )
        .unwrap()
        .to_string();
        assert!(wide_api.contains("MESSAGE"));

        let narrow_api = log_header(TuiLogTab::Api, 50).unwrap().to_string();
        assert!(!narrow_api.contains("MESSAGE"));
        assert!(narrow_api.contains("CODE"));
    }
}

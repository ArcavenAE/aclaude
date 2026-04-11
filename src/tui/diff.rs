//! Tool call rendering with diff display.
//!
//! Dispatches rendering by tool name: Edit shows unified diffs, Read shows
//! file content preview, Bash shows command and output, Write shows path.
//! Uses `similar` for text diffing.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use similar::{ChangeTag, TextDiff};

use super::app::ToolCallItem;

/// Maximum lines of tool output to show when collapsed.
const MAX_PREVIEW_LINES: usize = 12;

/// Context lines around diff changes.
const DIFF_CONTEXT: usize = 3;

/// Render a completed tool call into displayable lines.
///
/// Dispatches by tool name for specialized rendering. Returns lines
/// suitable for embedding in the conversation viewport.
pub fn render_tool_call(item: &ToolCallItem) -> Vec<Line<'static>> {
    let input: serde_json::Value =
        serde_json::from_str(&item.input_json).unwrap_or(serde_json::Value::Null);

    match item.name.as_str() {
        "Edit" => render_edit(&input),
        "Read" => render_read(&input, &item.result_preview, item.is_expanded),
        "Write" => render_write(&input),
        "Bash" => render_bash(&input, &item.result_preview, item.is_expanded),
        "Grep" => render_grep(&input, &item.result_preview, item.is_expanded),
        "Glob" => render_glob(&input, &item.result_preview, item.is_expanded),
        _ => render_generic(&item.name, &input),
    }
}

/// Render a tool result (output) into displayable lines.
pub fn render_result_preview(result: &str, expanded: bool) -> Vec<Line<'static>> {
    let max = if expanded {
        usize::MAX
    } else {
        MAX_PREVIEW_LINES
    };
    let total = result.lines().count();
    let mut lines: Vec<Line<'static>> = result
        .lines()
        .take(max)
        .map(|l| {
            Line::from(Span::styled(
                format!("    {l}"),
                Style::default().fg(Color::DarkGray),
            ))
        })
        .collect();

    if !expanded && total > max {
        lines.push(Line::from(Span::styled(
            format!("    [... {} more lines]", total - max),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
    }
    lines
}

// ── Tool-specific renderers ──────────────────────────────────────────────

fn render_edit(input: &serde_json::Value) -> Vec<Line<'static>> {
    let file_path = input
        .get("file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");
    let old_string = input
        .get("old_string")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let new_string = input
        .get("new_string")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut lines = vec![Line::from(vec![
        Span::styled("    ~ ", Style::default().fg(Color::Yellow)),
        Span::styled(
            file_path.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    if old_string.is_empty() && new_string.is_empty() {
        return lines;
    }

    lines.extend(render_unified_diff(old_string, new_string));
    lines
}

fn render_read(input: &serde_json::Value, result: &str, expanded: bool) -> Vec<Line<'static>> {
    let file_path = input
        .get("file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");

    let mut lines = vec![Line::from(vec![
        Span::styled("      ", Style::default()),
        Span::styled(file_path.to_string(), Style::default().fg(Color::White)),
    ])];

    if !result.is_empty() {
        lines.extend(render_result_preview(result, expanded));
    }
    lines
}

fn render_write(input: &serde_json::Value) -> Vec<Line<'static>> {
    let file_path = input
        .get("file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");

    vec![Line::from(vec![
        Span::styled("    + ", Style::default().fg(Color::Green)),
        Span::styled(
            file_path.to_string(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ])]
}

fn render_bash(input: &serde_json::Value, result: &str, expanded: bool) -> Vec<Line<'static>> {
    let command = input
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("<unknown>");

    // Truncate long commands to first line
    let cmd_display: String = command
        .lines()
        .next()
        .unwrap_or(command)
        .chars()
        .take(80)
        .collect();
    let truncated = command.lines().count() > 1 || command.len() > 80;

    let mut lines = vec![Line::from(vec![
        Span::styled("    $ ", Style::default().fg(Color::Magenta)),
        Span::styled(
            if truncated {
                format!("{cmd_display}...")
            } else {
                cmd_display
            },
            Style::default().fg(Color::Magenta),
        ),
    ])];

    if !result.is_empty() {
        lines.extend(render_result_preview(result, expanded));
    }
    lines
}

fn render_grep(input: &serde_json::Value, result: &str, expanded: bool) -> Vec<Line<'static>> {
    let pattern = input.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
    let path = input.get("path").and_then(|v| v.as_str()).unwrap_or(".");

    let mut lines = vec![Line::from(vec![
        Span::styled("    🔍 ", Style::default()),
        Span::styled(
            format!("/{pattern}/ in {path}"),
            Style::default().fg(Color::Cyan),
        ),
    ])];

    if !result.is_empty() {
        lines.extend(render_result_preview(result, expanded));
    }
    lines
}

fn render_glob(input: &serde_json::Value, result: &str, expanded: bool) -> Vec<Line<'static>> {
    let pattern = input.get("pattern").and_then(|v| v.as_str()).unwrap_or("");

    let mut lines = vec![Line::from(vec![
        Span::styled("    📁 ", Style::default()),
        Span::styled(pattern.to_string(), Style::default().fg(Color::Cyan)),
    ])];

    if !result.is_empty() {
        lines.extend(render_result_preview(result, expanded));
    }
    lines
}

fn render_generic(name: &str, input: &serde_json::Value) -> Vec<Line<'static>> {
    let preview: String = serde_json::to_string(input)
        .unwrap_or_default()
        .chars()
        .take(120)
        .collect();

    vec![Line::from(vec![
        Span::styled(
            format!("    {name}: "),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(preview, Style::default().fg(Color::DarkGray)),
    ])]
}

// ── Diff rendering ───────────────────────────────────────────────────────

/// Render a unified diff of old_string vs new_string.
fn render_unified_diff(old: &str, new: &str) -> Vec<Line<'static>> {
    let diff = TextDiff::from_lines(old, new);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut has_changes = false;

    for group in diff.grouped_ops(DIFF_CONTEXT) {
        for op in &group {
            for change in diff.iter_changes(op) {
                has_changes = true;
                let text = change.value().trim_end_matches('\n').to_string();
                let (prefix, style) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::default().fg(Color::Red)),
                    ChangeTag::Insert => ("+", Style::default().fg(Color::Green)),
                    ChangeTag::Equal => (" ", Style::default().fg(Color::DarkGray)),
                };
                lines.push(Line::from(Span::styled(
                    format!("      {prefix} {text}"),
                    style,
                )));
            }
        }

        // Separator between groups
        if lines.len() < 200 {
            lines.push(Line::from(Span::styled(
                "      ───",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Remove trailing separator
    if lines
        .last()
        .is_some_and(|l| l.spans.first().is_some_and(|s| s.content.contains("───")))
    {
        lines.pop();
    }

    if !has_changes {
        lines.push(Line::from(Span::styled(
            "      (no changes)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_diff_shows_changes() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\n";
        let lines = render_unified_diff(old, new);

        let text: Vec<String> = lines.iter().map(ToString::to_string).collect();
        assert!(text.iter().any(|l| l.contains("- line2")));
        assert!(text.iter().any(|l| l.contains("+ modified")));
    }

    #[test]
    fn unified_diff_empty_inputs() {
        let lines = render_unified_diff("", "");
        assert_eq!(lines.len(), 1); // "(no changes)"
    }

    #[test]
    fn render_edit_parses_input() {
        let input: serde_json::Value = serde_json::json!({
            "file_path": "src/main.rs",
            "old_string": "fn old() {}",
            "new_string": "fn new() {}"
        });
        let lines = render_edit(&input);
        let text: Vec<String> = lines.iter().map(ToString::to_string).collect();
        assert!(text[0].contains("src/main.rs"));
        assert!(text.iter().any(|l| l.contains("- fn old")));
        assert!(text.iter().any(|l| l.contains("+ fn new")));
    }

    #[test]
    fn render_bash_truncates_long_commands() {
        let long_cmd = "a".repeat(200);
        let input = serde_json::json!({ "command": long_cmd });
        let lines = render_bash(&input, "", false);
        let text = lines[0].to_string();
        assert!(text.contains("..."));
        assert!(text.len() < 200);
    }

    #[test]
    fn render_result_preview_truncates() {
        let result: String = (0..50)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let lines = render_result_preview(&result, false);
        // 12 lines + 1 "[... N more lines]"
        assert_eq!(lines.len(), MAX_PREVIEW_LINES + 1);
        let last = lines.last().unwrap().to_string();
        assert!(last.contains("more lines"));
    }

    #[test]
    fn render_result_preview_expanded_shows_all() {
        let result: String = (0..50)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let lines = render_result_preview(&result, true);
        assert_eq!(lines.len(), 50);
    }

    #[test]
    fn render_generic_shows_truncated_json() {
        let input = serde_json::json!({ "key": "value" });
        let lines = render_generic("CustomTool", &input);
        assert_eq!(lines.len(), 1);
        let text = lines[0].to_string();
        assert!(text.contains("CustomTool"));
    }
}

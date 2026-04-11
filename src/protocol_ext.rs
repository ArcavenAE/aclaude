//! Extended NDJSON event parsing for the session bridge.
//!
//! Wraps the existing `protocol::ClaudeEvent` with additional event types
//! needed for interactive TUI and diagnostic consumers: streaming text deltas,
//! tool results, rate limit events, and permission requests.
//!
//! This module is TUI-agnostic — no ratatui types. Both human TUI and future
//! marvel diagnostic view consume these events.

use crate::protocol::{self, ClaudeEvent};

/// Extended event type for the session bridge.
///
/// Wraps `ClaudeEvent` with additional event types that the core parser
/// doesn't handle (streaming deltas, rate limits, permissions).
#[derive(Debug)]
#[non_exhaustive]
pub enum BridgeEvent {
    /// Core event from the existing protocol parser.
    Core(ClaudeEvent),

    /// Streaming text chunk from `content_block_delta`.
    TextDelta { text: String },

    /// Tool result from a `user` type event containing tool_result blocks.
    ToolResult {
        tool_use_id: String,
        content: String,
    },

    /// Rate limit status change.
    RateLimit {
        status: String,
        resets_at: Option<String>,
    },

    /// Permission request from Claude Code (tool approval prompt).
    PermissionRequest { tool: String, description: String },
}

/// Aggregated session metrics, readable by any consumer.
///
/// Updated by the bridge from NDJSON events. The TUI reads this for its
/// status bar, tmux statusline reads it for the pane status, and a future
/// marvel sidecar could serialize it to a JSON file for control plane polling.
#[derive(Debug, Default, Clone)]
pub struct SessionMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cost_usd: f64,
    pub context_pct: f64,
    pub num_turns: u64,
    pub tool_use_count: u64,
    pub active_tool: Option<String>,
    pub rate_limit_status: Option<String>,
    pub model: String,
    pub session_id: Option<String>,
}

impl SessionMetrics {
    /// Estimate context window usage percentage.
    pub fn update_context_pct(&mut self, context_window: u64) {
        if context_window == 0 {
            self.context_pct = 0.0;
        } else {
            self.context_pct = (self.input_tokens as f64 / context_window as f64) * 100.0;
        }
    }
}

/// Parse a single NDJSON line into a `BridgeEvent`.
///
/// Tries the core parser first, then handles additional event types:
/// - `content_block_delta` → `TextDelta`
/// - `rate_limit_event` → `RateLimit`
/// - `user` with tool_result → `ToolResult`
///
/// Returns `None` for unparseable lines or empty lines.
pub fn parse_bridge_event(line: &str) -> Option<BridgeEvent> {
    if line.is_empty() {
        return None;
    }

    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let event_type = v.get("type")?.as_str()?;

    // Handle streaming text deltas (partial messages)
    if event_type == "content_block_delta" {
        let delta = v.get("delta")?;
        if delta.get("type").and_then(|t| t.as_str()) == Some("text_delta") {
            let text = delta.get("text")?.as_str()?.to_string();
            return Some(BridgeEvent::TextDelta { text });
        }
        return None;
    }

    // Handle rate limit events
    if event_type == "rate_limit_event" {
        let status = v
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();
        let resets_at = v
            .get("resets_at")
            .and_then(|s| s.as_str())
            .map(String::from);
        return Some(BridgeEvent::RateLimit { status, resets_at });
    }

    // Handle user events with tool results
    if event_type == "user" {
        if let Some(message) = v.get("message") {
            if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                for block in content {
                    if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                        let tool_use_id = block
                            .get("tool_use_id")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string();
                        // Content can be a string or array of blocks
                        let content_text = if let Some(s) =
                            block.get("content").and_then(|c| c.as_str())
                        {
                            s.to_string()
                        } else if let Some(arr) = block.get("content").and_then(|c| c.as_array()) {
                            arr.iter()
                                .filter_map(|b| {
                                    if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                                        b.get("text").and_then(|t| t.as_str()).map(String::from)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        } else {
                            String::new()
                        };
                        return Some(BridgeEvent::ToolResult {
                            tool_use_id,
                            content: content_text,
                        });
                    }
                }
            }
        }
    }

    // Fall through to core parser
    protocol::parse_event(line).map(BridgeEvent::Core)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_delta() {
        let line = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
        match parse_bridge_event(line) {
            Some(BridgeEvent::TextDelta { text }) => assert_eq!(text, "Hello"),
            other => panic!("expected TextDelta, got {other:?}"),
        }
    }

    #[test]
    fn parse_rate_limit() {
        let line = r#"{"type":"rate_limit_event","status":"rate_limited","resets_at":"2026-04-10T12:00:00Z"}"#;
        match parse_bridge_event(line) {
            Some(BridgeEvent::RateLimit { status, resets_at }) => {
                assert_eq!(status, "rate_limited");
                assert_eq!(resets_at.as_deref(), Some("2026-04-10T12:00:00Z"));
            }
            other => panic!("expected RateLimit, got {other:?}"),
        }
    }

    #[test]
    fn parse_tool_result_string_content() {
        let line = r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"abc123","content":"file contents here"}]}}"#;
        match parse_bridge_event(line) {
            Some(BridgeEvent::ToolResult {
                tool_use_id,
                content,
            }) => {
                assert_eq!(tool_use_id, "abc123");
                assert_eq!(content, "file contents here");
            }
            other => panic!("expected ToolResult, got {other:?}"),
        }
    }

    #[test]
    fn parse_core_system_event() {
        let line = r#"{"type":"system","session_id":"sess-123","tools":[]}"#;
        match parse_bridge_event(line) {
            Some(BridgeEvent::Core(ClaudeEvent::System { session_id })) => {
                assert_eq!(session_id, "sess-123");
            }
            other => panic!("expected Core(System), got {other:?}"),
        }
    }

    #[test]
    fn parse_empty_line_returns_none() {
        assert!(parse_bridge_event("").is_none());
    }

    #[test]
    fn parse_invalid_json_returns_none() {
        assert!(parse_bridge_event("not json").is_none());
    }

    #[test]
    fn session_metrics_context_pct() {
        let mut m = SessionMetrics {
            input_tokens: 100_000,
            ..SessionMetrics::default()
        };
        m.update_context_pct(200_000);
        assert!((m.context_pct - 50.0).abs() < 0.01);
    }
}

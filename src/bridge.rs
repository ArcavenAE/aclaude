//! Session bridge — async subprocess lifecycle for Claude Code.
//!
//! Spawns `claude` as a headless subprocess with bidirectional NDJSON streaming.
//! Parses events, updates `SessionMetrics`, and sends events to consumers via
//! an mpsc channel. The bridge is TUI-agnostic — no ratatui types.
//!
//! Both human TUI and future marvel diagnostic view consume the same bridge.
//! tmux statusline updates happen here so any consumer gets status for free.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::config::AclaudeConfig;
use crate::error::{AclaudeError, Result};
use crate::persona;
use crate::protocol::ClaudeEvent;
use crate::protocol_ext::{self, BridgeEvent, SessionMetrics};
use crate::session::find_claude;
use crate::statusline;

/// A running Claude Code subprocess with event streaming.
///
/// The bridge owns the child process and provides:
/// - An event receiver (mpsc) for consuming NDJSON events
/// - Shared metrics via `Arc<Mutex<SessionMetrics>>`
/// - Methods to send user messages and shut down
pub struct Session {
    child: Child,
    event_rx: mpsc::Receiver<BridgeEvent>,
    metrics: Arc<Mutex<SessionMetrics>>,
    stdin_tx: mpsc::Sender<String>,
}

impl Session {
    /// Spawn a Claude Code subprocess with NDJSON streaming.
    ///
    /// Starts `claude` with `--output-format stream-json --input-format stream-json
    /// --verbose --include-partial-messages` and the persona system prompt.
    ///
    /// Returns a `Session` that produces `BridgeEvent`s via its event receiver
    /// and accepts user messages via `send_user_message`.
    pub async fn spawn(config: &AclaudeConfig) -> Result<Self> {
        let claude_path = find_claude()?;

        let system_prompt = {
            let theme = persona::load_theme(&config.persona.theme)?;
            let agent = persona::get_agent(&theme, &config.persona.role)?;
            persona::build_system_prompt(&theme, agent, &config.persona.immersion)
        };

        let mut cmd = Command::new(&claude_path);
        cmd.args(["--output-format", "stream-json"])
            .args(["--input-format", "stream-json"])
            .args(["--verbose"])
            .args(["--include-partial-messages"])
            .args(["--model", &config.session.model])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        if !system_prompt.is_empty() {
            cmd.args(["--append-system-prompt", &system_prompt]);
        }

        let mut child = cmd.spawn().map_err(|e| AclaudeError::Session {
            message: format!("failed to start claude: {e}"),
        })?;

        let stdout = child.stdout.take().ok_or_else(|| AclaudeError::Session {
            message: "failed to capture claude stdout".to_string(),
        })?;

        let child_stdin = child.stdin.take().ok_or_else(|| AclaudeError::Session {
            message: "failed to capture claude stdin".to_string(),
        })?;

        let metrics = Arc::new(Mutex::new(SessionMetrics {
            model: config.session.model.clone(),
            ..SessionMetrics::default()
        }));

        let (event_tx, event_rx) = mpsc::channel::<BridgeEvent>(256);
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(32);

        // Reader task: stdout → parse → metrics update → event channel
        let reader_metrics = Arc::clone(&metrics);
        let statusline_config = config.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.is_empty() {
                    continue;
                }
                if let Some(event) = protocol_ext::parse_bridge_event(&line) {
                    // Update shared metrics from event data
                    update_metrics(&reader_metrics, &event);

                    // Push tmux statusline (bridge-level, any consumer gets this)
                    push_statusline_from_metrics(&reader_metrics, &statusline_config);

                    // Send to consumer — if channel is full or closed, drop the event
                    if event_tx.send(event).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Writer task: stdin channel → child stdin
        tokio::spawn(async move {
            let mut stdin = child_stdin;
            while let Some(msg) = stdin_rx.recv().await {
                if stdin.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        Ok(Session {
            child,
            event_rx,
            metrics,
            stdin_tx,
        })
    }

    /// Get the event receiver for consuming bridge events.
    ///
    /// The receiver yields `BridgeEvent`s as they arrive from the subprocess.
    /// When the subprocess exits, the channel closes.
    pub fn event_rx(&mut self) -> &mut mpsc::Receiver<BridgeEvent> {
        &mut self.event_rx
    }

    /// Get a shared reference to session metrics.
    ///
    /// Any consumer can clone this Arc and read metrics independently of
    /// the event channel. The TUI reads it for the status bar. A future
    /// marvel sidecar could serialize it to a JSON file.
    pub fn metrics(&self) -> Arc<Mutex<SessionMetrics>> {
        Arc::clone(&self.metrics)
    }

    /// Send a user message to the Claude Code subprocess.
    ///
    /// Writes the NDJSON user message format to stdin.
    pub async fn send_user_message(&self, text: &str) -> Result<()> {
        let msg = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": text
            }
        });
        let line = format!(
            "{}\n",
            serde_json::to_string(&msg).map_err(|e| {
                AclaudeError::Session {
                    message: format!("failed to serialize message: {e}"),
                }
            })?
        );
        self.stdin_tx
            .send(line)
            .await
            .map_err(|_| AclaudeError::Session {
                message: "subprocess stdin closed".to_string(),
            })
    }

    /// Gracefully shut down the subprocess.
    pub async fn shutdown(&mut self) {
        // Drop the stdin sender to signal EOF
        // (already dropped when Session is dropped, but explicit is clearer)
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
    }
}

/// Update `SessionMetrics` from a `BridgeEvent`.
fn update_metrics(metrics: &Arc<Mutex<SessionMetrics>>, event: &BridgeEvent) {
    let mut m = match metrics.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };
    match event {
        BridgeEvent::Core(ClaudeEvent::System { session_id }) => {
            m.session_id = Some(session_id.clone());
        }
        BridgeEvent::Core(ClaudeEvent::Assistant { message }) => {
            // Track tool uses
            for block in &message.content {
                if block.block_type == "tool_use" {
                    m.tool_use_count += 1;
                    m.active_tool = block.name.clone();
                }
            }
            // Update token counts
            if let Some(usage) = &message.usage {
                m.input_tokens += usage.input_tokens;
                m.output_tokens += usage.output_tokens;
                m.cache_read_tokens += usage.cache_read_input_tokens;
                m.cache_creation_tokens += usage.cache_creation_input_tokens;
                m.update_context_pct(200_000);
            }
        }
        BridgeEvent::Core(ClaudeEvent::Result { payload }) => {
            m.cost_usd = payload.cost_usd;
            m.num_turns = payload.num_turns;
            m.active_tool = None;
        }
        BridgeEvent::RateLimit { status, .. } => {
            m.rate_limit_status = Some(status.clone());
        }
        BridgeEvent::ToolResult { .. } => {
            // Tool completed — clear active tool
            m.active_tool = None;
        }
        _ => {}
    }
}

/// Push tmux statusline from current metrics.
///
/// Continues the pattern from `session.rs` — statusline updates happen at
/// bridge level so any consumer (TUI, headless, diagnostic) gets tmux
/// status for free.
fn push_statusline_from_metrics(metrics: &Arc<Mutex<SessionMetrics>>, config: &AclaudeConfig) {
    if !config.statusline.enabled {
        return;
    }
    let m = match metrics.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    // Only update when we have token data
    if m.input_tokens == 0 {
        return;
    }

    let character_name = config
        .persona
        .theme
        .split('/')
        .next_back()
        .unwrap_or("aclaude");
    let left = statusline::render_statusline(config, character_name, Some(m.context_pct));
    let right = statusline::build_progress_bar(m.context_pct, 10);
    statusline::write_tmux_cache(&left, &right);
}

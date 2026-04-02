#![allow(dead_code)]

use std::collections::HashMap;

use chrono::Utc;
use serde::Serialize;

/// A recorded event in the session audit trail.
#[derive(Debug, Clone, Serialize)]
pub struct HookEvent {
    pub timestamp: String,
    pub event: String,
    pub session_id: String,
    pub data: serde_json::Value,
}

/// A record of a tool invocation.
#[derive(Debug, Clone, Serialize)]
pub struct ToolUsageRecord {
    pub tool_name: String,
    pub tool_use_id: Option<String>,
    pub input_summary: String,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
    pub success: Option<bool>,
}

/// Tracks tool usage and session events.
pub struct SessionHooks {
    events: Vec<HookEvent>,
    tool_usage: Vec<ToolUsageRecord>,
    session_id: String,
}

impl SessionHooks {
    pub fn new(session_id: String) -> Self {
        Self {
            events: Vec::new(),
            tool_usage: Vec::new(),
            session_id,
        }
    }

    pub fn record_event(&mut self, event: &str, data: serde_json::Value) {
        self.events.push(HookEvent {
            timestamp: Utc::now().to_rfc3339(),
            event: event.to_string(),
            session_id: self.session_id.clone(),
            data,
        });
    }

    pub fn record_tool_use(
        &mut self,
        tool_name: &str,
        input_summary: &str,
        tool_use_id: Option<&str>,
    ) {
        self.tool_usage.push(ToolUsageRecord {
            tool_name: tool_name.to_string(),
            tool_use_id: tool_use_id.map(String::from),
            input_summary: input_summary.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms: None,
            success: None,
        });
    }

    pub fn tool_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for record in &self.tool_usage {
            *counts.entry(record.tool_name.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn failed_tools(&self) -> Vec<&ToolUsageRecord> {
        self.tool_usage
            .iter()
            .filter(|r| r.success == Some(false))
            .collect()
    }

    pub fn events(&self) -> &[HookEvent] {
        &self.events
    }

    pub fn tool_usage(&self) -> &[ToolUsageRecord] {
        &self.tool_usage
    }
}

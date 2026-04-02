#![allow(dead_code)]

use std::process::{Command, Stdio};

use crate::config::AclaudeConfig;
use crate::error::{AclaudeError, Result};
use crate::persona;

/// Check that the `claude` CLI is available.
pub fn find_claude() -> Result<String> {
    let output = Command::new("sh")
        .args(["-c", "command -v claude"])
        .output()
        .map_err(|_| AclaudeError::ClaudeNotFound)?;

    if !output.status.success() {
        return Err(AclaudeError::ClaudeNotFound);
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err(AclaudeError::ClaudeNotFound);
    }

    Ok(path)
}

/// Start an interactive session with Claude Code.
///
/// Spawns `claude` as a subprocess using the NDJSON streaming protocol.
/// The persona system prompt is injected via --append-system-prompt.
pub fn start_session(config: &AclaudeConfig) -> Result<()> {
    let claude_path = find_claude()?;

    // Build persona prompt
    let system_prompt = {
        let theme = persona::load_theme(&config.persona.theme)?;
        let agent = persona::get_agent(&theme, &config.persona.role)?;
        persona::build_system_prompt(&theme, agent, &config.persona.immersion)
    };

    let mut cmd = Command::new(&claude_path);
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Pass model
    cmd.args(["--model", &config.session.model]);

    // Inject persona as appended system prompt
    if !system_prompt.is_empty() {
        cmd.args(["--append-system-prompt", &system_prompt]);
    }

    let status = cmd.status().map_err(|e| AclaudeError::Session {
        message: format!("failed to start claude: {e}"),
    })?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        if code != 0 {
            return Err(AclaudeError::Session {
                message: format!("claude exited with code {code}"),
            });
        }
    }

    Ok(())
}

/// Run a one-shot prompt (non-interactive).
pub fn run_prompt(config: &AclaudeConfig, prompt: &str) -> Result<String> {
    let claude_path = find_claude()?;

    let system_prompt = {
        let theme = persona::load_theme(&config.persona.theme)?;
        let agent = persona::get_agent(&theme, &config.persona.role)?;
        persona::build_system_prompt(&theme, agent, &config.persona.immersion)
    };

    let mut cmd = Command::new(&claude_path);
    cmd.args(["-p", prompt])
        .args(["--model", &config.session.model])
        .args(["--output-format", "json"]);

    if !system_prompt.is_empty() {
        cmd.args(["--append-system-prompt", &system_prompt]);
    }

    let output = cmd.output().map_err(|e| AclaudeError::Session {
        message: format!("failed to run claude: {e}"),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AclaudeError::Session {
            message: format!("claude error: {stderr}"),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

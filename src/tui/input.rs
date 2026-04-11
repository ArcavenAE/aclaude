//! Key handling and slash command parsing for the TUI.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Action resulting from a key press.
#[derive(Debug)]
pub enum InputAction {
    /// Send the input buffer as a user message.
    SendMessage(String),
    /// Process a local slash command.
    SlashCommand(SlashCmd),
    /// Quit the TUI.
    Quit,
    /// Scroll conversation up one line.
    ScrollUp,
    /// Scroll conversation down one line.
    ScrollDown,
    /// Scroll conversation up one page.
    PageUp,
    /// Scroll conversation down one page.
    PageDown,
    /// Scroll to bottom of conversation.
    ScrollEnd,
    /// No action (key consumed but no effect).
    None,
}

/// Parsed slash commands.
#[derive(Debug)]
pub enum SlashCmd {
    /// Set portrait size: /persona portrait size [small|medium|large|original]
    PortraitSize(String),
    /// Unknown slash command.
    Unknown(String),
}

/// Handle a key event against the current input buffer.
///
/// Modifies `input_buffer` in place (for character input, backspace).
/// Returns an `InputAction` describing what the TUI should do.
pub fn handle_key(event: KeyEvent, input_buffer: &mut String) -> InputAction {
    match (event.modifiers, event.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => InputAction::Quit,

        // Submit
        (_, KeyCode::Enter) => {
            let text = input_buffer.trim().to_string();
            input_buffer.clear();
            if text.is_empty() {
                return InputAction::None;
            }
            if let Some(cmd) = parse_slash_command(&text) {
                InputAction::SlashCommand(cmd)
            } else {
                InputAction::SendMessage(text)
            }
        }

        // Editing
        (_, KeyCode::Backspace) => {
            input_buffer.pop();
            InputAction::None
        }

        // Scrolling
        (_, KeyCode::Up) => InputAction::ScrollUp,
        (_, KeyCode::Down) => InputAction::ScrollDown,
        (_, KeyCode::PageUp) => InputAction::PageUp,
        (_, KeyCode::PageDown) => InputAction::PageDown,
        (_, KeyCode::End) => InputAction::ScrollEnd,
        (_, KeyCode::Home) => InputAction::ScrollEnd,

        // Character input
        (_, KeyCode::Char(c)) => {
            input_buffer.push(c);
            InputAction::None
        }

        _ => InputAction::None,
    }
}

/// Parse a slash command from input text.
fn parse_slash_command(text: &str) -> Option<SlashCmd> {
    if !text.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = text.split_whitespace().collect();

    // /persona portrait size <size>
    if parts.len() == 4 && parts[0] == "/persona" && parts[1] == "portrait" && parts[2] == "size" {
        let size = parts[3].to_lowercase();
        if ["small", "medium", "large", "original"].contains(&size.as_str()) {
            return Some(SlashCmd::PortraitSize(size));
        }
    }

    Some(SlashCmd::Unknown(text.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_portrait_size_command() {
        match parse_slash_command("/persona portrait size large") {
            Some(SlashCmd::PortraitSize(s)) => assert_eq!(s, "large"),
            other => panic!("expected PortraitSize, got {other:?}"),
        }
    }

    #[test]
    fn parse_unknown_slash_command() {
        match parse_slash_command("/unknown") {
            Some(SlashCmd::Unknown(s)) => assert_eq!(s, "/unknown"),
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn non_slash_returns_none() {
        assert!(parse_slash_command("hello").is_none());
    }
}

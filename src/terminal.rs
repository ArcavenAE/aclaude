//! Terminal graphics capability detection.
//!
//! Three-tier detection: known-good terminals → known-bad → unknown.
//! No I/O or subprocesses for detection — env vars only. Display tool
//! selection walks PATH to find available image renderers.

use std::env;

/// Terminal inline image support detection result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImageSupport {
    /// Terminal is known to support inline images. Use the given tool.
    Supported(DisplayTool),
    /// Terminal is known NOT to support inline images.
    Unsupported,
    /// Terminal support is unknown. Caller may attempt with [`best_available_tool`].
    Unknown,
}

/// A display tool that can render inline images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DisplayTool {
    /// Kitty graphics protocol via `kitten icat`.
    KittenIcat,
    /// iTerm2 inline images protocol via `wezterm imgcat`.
    WeztermImgcat,
}

/// Detect terminal image support from environment variables.
///
/// Reads only env vars — no I/O, no subprocesses.
pub fn detect_image_support() -> ImageSupport {
    detect_from_env(|key| env::var(key).ok())
}

/// Core detection logic, parameterized for testability.
fn detect_from_env(lookup: impl Fn(&str) -> Option<String>) -> ImageSupport {
    let term = lookup("TERM").unwrap_or_default().to_lowercase();
    let term_program = lookup("TERM_PROGRAM").unwrap_or_default().to_lowercase();

    // ── Known-bad ────────────────────────────────────────────────────────
    if term == "dumb" || term == "linux" {
        return ImageSupport::Unsupported;
    }
    if term_program == "apple_terminal" {
        return ImageSupport::Unsupported;
    }

    // ── Known-good: terminal-specific env vars (survive inside tmux) ─────
    // Inside tmux, TERM_PROGRAM is overwritten to "tmux". These vars are
    // set by the outer terminal and inherited through tmux.

    if lookup("KITTY_WINDOW_ID").is_some() {
        return ImageSupport::Supported(DisplayTool::KittenIcat);
    }
    if lookup("WEZTERM_EXECUTABLE").is_some() || lookup("WEZTERM_PANE").is_some() {
        return ImageSupport::Supported(DisplayTool::KittenIcat);
    }
    if lookup("GHOSTTY_RESOURCES_DIR").is_some() {
        return ImageSupport::Supported(DisplayTool::KittenIcat);
    }

    // ── Known-good: TERM_PROGRAM (works outside tmux) ────────────────────
    match term_program.as_str() {
        "kitty" | "ghostty" | "wezterm" | "konsole" | "contour" | "rio" => {
            return ImageSupport::Supported(DisplayTool::KittenIcat);
        }
        _ => {}
    }
    if term.contains("kitty") {
        return ImageSupport::Supported(DisplayTool::KittenIcat);
    }

    // ── Unknown ──────────────────────────────────────────────────────────
    ImageSupport::Unknown
}

/// Select the best available display tool by checking PATH.
pub fn best_available_tool() -> Option<DisplayTool> {
    if tool_on_path("kitten") {
        return Some(DisplayTool::KittenIcat);
    }
    if tool_on_path("wezterm") {
        return Some(DisplayTool::WeztermImgcat);
    }
    None
}

/// Return true if the current process is running inside a tmux session.
pub fn inside_tmux() -> bool {
    env::var("TMUX").is_ok()
}

/// Check if a binary name exists on PATH.
fn tool_on_path(name: &str) -> bool {
    env::var_os("PATH")
        .map(|path| {
            env::split_paths(&path).any(|dir| {
                let candidate = dir.join(name);
                candidate.is_file()
            })
        })
        .unwrap_or(false)
}

// Convenience for resolve_display_intent callers: resolve the tool when
// detection says Supported but we still need to verify the binary exists.
fn verify_tool(tool: DisplayTool) -> Option<DisplayTool> {
    match tool {
        DisplayTool::KittenIcat => {
            if tool_on_path("kitten") {
                Some(DisplayTool::KittenIcat)
            } else if tool_on_path("wezterm") {
                // Terminal supports Kitty protocol but kitten isn't installed.
                // Fall back to wezterm imgcat if available.
                Some(DisplayTool::WeztermImgcat)
            } else {
                None
            }
        }
        DisplayTool::WeztermImgcat => {
            if tool_on_path("wezterm") {
                Some(DisplayTool::WeztermImgcat)
            } else {
                None
            }
        }
    }
}

/// Resolve whether to attempt image display and which tool to use.
///
/// `display_override` is the user's config value: "auto", "always", or "never".
pub fn resolve_display_intent(display_override: &str) -> (bool, Option<DisplayTool>) {
    match display_override {
        "never" => (false, None),
        "always" => {
            let tool = best_available_tool();
            (tool.is_some(), tool)
        }
        _ => {
            // "auto" — use detection
            match detect_image_support() {
                ImageSupport::Supported(tool) => {
                    let verified = verify_tool(tool);
                    (verified.is_some(), verified)
                }
                ImageSupport::Unsupported => (false, None),
                ImageSupport::Unknown => {
                    // Unknown terminal — try anyway. Image display failure is cosmetic.
                    let tool = best_available_tool();
                    (tool.is_some(), tool)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn env_from<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        let map: HashMap<&str, &str> = pairs.iter().copied().collect();
        move |key: &str| map.get(key).map(|v| (*v).to_string())
    }

    #[test]
    fn dumb_terminal_unsupported() {
        let lookup = env_from(&[("TERM", "dumb")]);
        assert_eq!(detect_from_env(lookup), ImageSupport::Unsupported);
    }

    #[test]
    fn linux_console_unsupported() {
        let lookup = env_from(&[("TERM", "linux")]);
        assert_eq!(detect_from_env(lookup), ImageSupport::Unsupported);
    }

    #[test]
    fn apple_terminal_unsupported() {
        let lookup = env_from(&[("TERM_PROGRAM", "Apple_Terminal")]);
        assert_eq!(detect_from_env(lookup), ImageSupport::Unsupported);
    }

    #[test]
    fn kitty_via_window_id() {
        let lookup = env_from(&[("KITTY_WINDOW_ID", "1")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn wezterm_via_executable() {
        let lookup = env_from(&[("WEZTERM_EXECUTABLE", "/usr/bin/wezterm-gui")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn wezterm_via_pane() {
        let lookup = env_from(&[("WEZTERM_PANE", "0")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn ghostty_via_resources_dir() {
        let lookup = env_from(&[("GHOSTTY_RESOURCES_DIR", "/usr/share/ghostty")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn kitty_via_term_program() {
        let lookup = env_from(&[("TERM_PROGRAM", "kitty")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn ghostty_via_term_program() {
        let lookup = env_from(&[("TERM_PROGRAM", "ghostty")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn wezterm_via_term_program() {
        let lookup = env_from(&[("TERM_PROGRAM", "WezTerm")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn konsole_via_term_program() {
        let lookup = env_from(&[("TERM_PROGRAM", "konsole")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn xterm_kitty_term() {
        let lookup = env_from(&[("TERM", "xterm-kitty")]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn unknown_terminal() {
        let lookup = env_from(&[("TERM", "xterm-256color")]);
        assert_eq!(detect_from_env(lookup), ImageSupport::Unknown);
    }

    #[test]
    fn empty_env_is_unknown() {
        let lookup = env_from(&[]);
        assert_eq!(detect_from_env(lookup), ImageSupport::Unknown);
    }

    #[test]
    fn inside_tmux_wezterm_detected() {
        // Inside tmux, TERM_PROGRAM=tmux, but WEZTERM_EXECUTABLE survives
        let lookup = env_from(&[
            ("TERM_PROGRAM", "tmux"),
            ("TERM", "tmux-256color"),
            ("WEZTERM_EXECUTABLE", "/opt/homebrew/bin/wezterm-gui"),
        ]);
        assert_eq!(
            detect_from_env(lookup),
            ImageSupport::Supported(DisplayTool::KittenIcat)
        );
    }

    #[test]
    fn known_bad_takes_priority_over_term_containing_kitty() {
        // Pathological: TERM=dumb should win even if it somehow contained "kitty"
        let lookup = env_from(&[("TERM", "dumb")]);
        assert_eq!(detect_from_env(lookup), ImageSupport::Unsupported);
    }

    #[test]
    fn resolve_never_skips_everything() {
        let (should_try, tool) = resolve_display_intent("never");
        assert!(!should_try);
        assert!(tool.is_none());
    }
}

//! Layout computation for the TUI.
//!
//! ```text
//! ┌─────────────────────────────┬──────────┐
//! │ CONVERSATION VIEWPORT       │ PORTRAIT │
//! │ (scrollable)                │ (upper   │
//! │                             │  right)  │
//! ├─────────────────────────────┴──────────┤
//! │ > INPUT AREA                            │
//! ├─────────────────────────────────────────┤
//! │ STATUS BAR                              │
//! └─────────────────────────────────────────┘
//! ```

use ratatui::layout::{Constraint, Direction, Layout, Rect};

use super::app::PortraitSize;

/// Minimum terminal width to show portrait column.
const MIN_WIDTH_FOR_PORTRAIT: u16 = 60;

/// Input area height in rows.
const INPUT_HEIGHT: u16 = 3;

/// Status bar height in rows.
const STATUS_HEIGHT: u16 = 1;

/// Computed layout areas for a single frame.
pub struct TuiLayout {
    /// Conversation viewport (scrollable text).
    pub conversation: Rect,
    /// Portrait area (upper-right, may be zero-size).
    pub portrait: Rect,
    /// User input area.
    pub input: Rect,
    /// Status bar.
    pub status: Rect,
}

/// Compute layout for the given terminal size and portrait configuration.
pub fn compute_layout(area: Rect, portrait_size: PortraitSize, has_portrait: bool) -> TuiLayout {
    // Vertical split: main content | input | status
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),                // main content area
            Constraint::Length(INPUT_HEIGHT),  // input
            Constraint::Length(STATUS_HEIGHT), // status
        ])
        .split(area);

    let main_area = vertical[0];
    let input = vertical[1];
    let status = vertical[2];

    // Horizontal split of main area: conversation | portrait
    let show_portrait = has_portrait && area.width >= MIN_WIDTH_FOR_PORTRAIT;

    if show_portrait {
        let portrait_width = portrait_column_width(portrait_size, area.width);
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),                 // conversation
                Constraint::Length(portrait_width), // portrait
            ])
            .split(main_area);

        TuiLayout {
            conversation: horizontal[0],
            portrait: horizontal[1],
            input,
            status,
        }
    } else {
        TuiLayout {
            conversation: main_area,
            portrait: Rect::default(), // zero-size
            input,
            status,
        }
    }
}

/// Portrait column width for a given size setting.
fn portrait_column_width(size: PortraitSize, terminal_width: u16) -> u16 {
    match size {
        PortraitSize::Small => 20,
        PortraitSize::Medium => 32,
        PortraitSize::Large => 48,
        PortraitSize::Original => (terminal_width / 3).min(64),
    }
}

//! Portrait image overlay widget for ratatui.
//!
//! Wraps `ratatui-image` to render persona portraits in the TUI.
//! Falls back gracefully when the terminal doesn't support inline images.

use std::path::{Path, PathBuf};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};

use super::app::PortraitSize;
use crate::portrait::PortraitPaths;

/// Portrait widget state.
///
/// Manages the image picker, current protocol state, and loaded image path.
/// Returns `None` from `new()` if the terminal doesn't support images.
pub struct PortraitWidget {
    picker: Picker,
    image_state: Option<StatefulProtocol>,
    current_path: Option<PathBuf>,
}

impl PortraitWidget {
    /// Create a new portrait widget.
    ///
    /// Must be called AFTER `crossterm::terminal::enable_raw_mode()` and
    /// BEFORE `Terminal::new()` — the picker queries terminal capabilities
    /// via stdio.
    ///
    /// Returns `None` if the terminal doesn't support inline images.
    pub fn new() -> Option<Self> {
        let picker = Picker::from_query_stdio().ok()?;
        Some(Self {
            picker,
            image_state: None,
            current_path: None,
        })
    }

    /// Set the portrait size, loading the appropriate image.
    ///
    /// Only reloads if the resolved path changed.
    pub fn set_size(&mut self, size: PortraitSize, paths: &PortraitPaths) {
        let size_str = match size {
            PortraitSize::Small => "small",
            PortraitSize::Medium => "medium",
            PortraitSize::Large => "large",
            PortraitSize::Original => "original",
        };
        let target_path = paths.best_for_size(size_str).map(Path::to_path_buf);

        // Skip reload if same path
        if self.current_path == target_path {
            return;
        }

        self.current_path = target_path.clone();

        if let Some(path) = target_path {
            match image::open(&path) {
                Ok(img) => {
                    let protocol = self.picker.new_resize_protocol(img);
                    self.image_state = Some(protocol);
                }
                Err(_) => {
                    self.image_state = None;
                }
            }
        } else {
            self.image_state = None;
        }
    }

    /// Whether a portrait image is loaded and ready to render.
    pub fn has_image(&self) -> bool {
        self.image_state.is_some()
    }

    /// Render the portrait in the given area.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        if let Some(state) = &mut self.image_state {
            let image = StatefulImage::default().resize(Resize::Fit(None));
            frame.render_stateful_widget(image, area, state);
        }
    }
}

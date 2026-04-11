//! Scroll state for the conversation viewport.

/// Tracks scroll position with auto-scroll behavior.
///
/// Auto-scroll follows new content as it arrives. Manual scrolling
/// (arrow keys, page up) disables auto-scroll. End key re-enables it.
#[derive(Debug)]
pub struct ScrollState {
    /// Current scroll offset (lines from top).
    pub offset: u16,
    /// Total content height in lines.
    pub content_height: u16,
    /// Visible viewport height in lines.
    pub viewport_height: u16,
    /// Whether to automatically follow new content.
    pub auto_scroll: bool,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self {
            offset: 0,
            content_height: 0,
            viewport_height: 0,
            auto_scroll: true,
        }
    }
}

impl ScrollState {
    /// Maximum valid scroll offset.
    fn max_offset(&self) -> u16 {
        self.content_height.saturating_sub(self.viewport_height)
    }

    /// Update content height and auto-scroll to bottom if enabled.
    pub fn set_content_height(&mut self, height: u16) {
        self.content_height = height;
        if self.auto_scroll {
            self.offset = self.max_offset();
        }
    }

    /// Update viewport height.
    pub fn set_viewport_height(&mut self, height: u16) {
        self.viewport_height = height;
        if self.auto_scroll {
            self.offset = self.max_offset();
        }
    }

    /// Scroll up by one line. Disables auto-scroll.
    pub fn scroll_up(&mut self) {
        self.auto_scroll = false;
        self.offset = self.offset.saturating_sub(1);
    }

    /// Scroll down by one line.
    pub fn scroll_down(&mut self) {
        self.auto_scroll = false;
        let max = self.max_offset();
        self.offset = (self.offset + 1).min(max);
        // Re-enable auto-scroll if we've scrolled to bottom
        if self.offset >= max {
            self.auto_scroll = true;
        }
    }

    /// Scroll up by a page.
    pub fn page_up(&mut self) {
        self.auto_scroll = false;
        self.offset = self.offset.saturating_sub(self.viewport_height);
    }

    /// Scroll down by a page.
    pub fn page_down(&mut self) {
        self.auto_scroll = false;
        let max = self.max_offset();
        self.offset = (self.offset + self.viewport_height).min(max);
        if self.offset >= max {
            self.auto_scroll = true;
        }
    }

    /// Jump to bottom and re-enable auto-scroll.
    pub fn scroll_to_bottom(&mut self) {
        self.auto_scroll = true;
        self.offset = self.max_offset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_scroll_follows_content() {
        let mut s = ScrollState::default();
        s.set_viewport_height(10);
        s.set_content_height(20);
        assert_eq!(s.offset, 10);

        s.set_content_height(30);
        assert_eq!(s.offset, 20);
    }

    #[test]
    fn manual_scroll_disables_auto() {
        let mut s = ScrollState::default();
        s.set_viewport_height(10);
        s.set_content_height(30);
        assert!(s.auto_scroll);

        s.scroll_up();
        assert!(!s.auto_scroll);
        assert_eq!(s.offset, 19);
    }

    #[test]
    fn scroll_to_bottom_re_enables_auto() {
        let mut s = ScrollState::default();
        s.set_viewport_height(10);
        s.set_content_height(30);
        s.scroll_up();
        assert!(!s.auto_scroll);

        s.scroll_to_bottom();
        assert!(s.auto_scroll);
        assert_eq!(s.offset, 20);
    }

    #[test]
    fn scroll_down_at_bottom_re_enables_auto() {
        let mut s = ScrollState::default();
        s.set_viewport_height(10);
        s.set_content_height(30);
        s.offset = 19;
        s.auto_scroll = false;

        s.scroll_down();
        assert!(s.auto_scroll);
        assert_eq!(s.offset, 20);
    }
}

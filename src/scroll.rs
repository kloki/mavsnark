pub(crate) struct ScrollState {
    pub(crate) offset: usize,
    pub(crate) selected: usize,
    pub(crate) auto_scroll: bool,
}

impl ScrollState {
    pub(crate) fn new() -> Self {
        Self {
            offset: 0,
            selected: 0,
            auto_scroll: true,
        }
    }

    pub(crate) fn select_up(&mut self, amount: usize) {
        self.auto_scroll = false;
        self.selected = self.selected.saturating_sub(amount);
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    pub(crate) fn select_down(&mut self, amount: usize, total: usize, visible: usize) {
        if total == 0 {
            return;
        }
        self.selected = self.selected.saturating_add(amount).min(total - 1);
        if self.selected >= self.offset + visible {
            self.offset = self.selected.saturating_sub(visible - 1);
        }
        self.auto_scroll = self.selected >= total.saturating_sub(1);
    }

    pub(crate) fn select_top(&mut self) {
        self.auto_scroll = false;
        self.selected = 0;
        self.offset = 0;
    }

    pub(crate) fn select_bottom(&mut self, total: usize, visible: usize) {
        if total == 0 {
            return;
        }
        self.auto_scroll = true;
        self.selected = total - 1;
        self.offset = total.saturating_sub(visible);
    }

    pub(crate) fn clamp(&mut self, total: usize, visible: usize) {
        if total == 0 {
            self.selected = 0;
            self.offset = 0;
        } else {
            self.selected = self.selected.min(total - 1);
            self.offset = self.offset.min(total.saturating_sub(visible));
        }
    }

    pub(crate) fn auto_follow(&mut self, total: usize, visible: usize) {
        if self.auto_scroll && total > 0 {
            self.selected = total - 1;
            self.offset = total.saturating_sub(visible);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_up_from_zero() {
        let mut s = ScrollState::new();
        s.select_up(1);
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
    }

    #[test]
    fn scroll_down_clamps_to_total() {
        let mut s = ScrollState::new();
        s.select_down(100, 5, 10);
        assert_eq!(s.selected, 4);
    }

    #[test]
    fn scroll_down_adjusts_offset() {
        let mut s = ScrollState::new();
        for _ in 0..5 {
            s.select_down(1, 10, 3);
        }
        assert_eq!(s.selected, 5);
        assert!(s.offset + 3 > s.selected);
    }

    #[test]
    fn scroll_up_adjusts_offset() {
        let mut s = ScrollState::new();
        s.select_bottom(10, 3);
        s.select_up(5);
        assert_eq!(s.selected, 4);
        assert!(s.offset <= s.selected);
    }

    #[test]
    fn select_top_resets() {
        let mut s = ScrollState::new();
        s.select_down(5, 10, 3);
        s.select_top();
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
    }

    #[test]
    fn select_bottom_jumps_to_end() {
        let mut s = ScrollState::new();
        s.select_bottom(10, 3);
        assert_eq!(s.selected, 9);
        assert_eq!(s.offset, 7);
    }

    #[test]
    fn auto_follow_when_enabled() {
        let mut s = ScrollState::new();
        s.auto_scroll = true;
        s.auto_follow(10, 5);
        assert_eq!(s.selected, 9);
        assert_eq!(s.offset, 5);
    }

    #[test]
    fn auto_follow_noop_when_disabled() {
        let mut s = ScrollState::new();
        s.auto_scroll = false;
        s.auto_follow(10, 5);
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
    }
}

use super::dotfiles::Dotfiles;

impl Dotfiles {
    pub(crate) fn scroll_script(&mut self, amount: i16) {
        if self.script_lines.is_empty() {
            return;
        }
        if self.script_scroll == self.script_lines.len().saturating_sub(self.view_height) as u16
            && amount > 0
        {
            return;
        }
        if self.script_scroll == 0 && amount < 0 {
            return;
        }
        self.script_scroll = if amount < 0 {
            self.script_scroll.saturating_sub(amount.unsigned_abs())
        } else {
            self.script_scroll.saturating_add(amount.unsigned_abs())
        };
        let max_scroll = self.script_lines.len().saturating_sub(self.view_height) as u16;
        self.script_scroll = self.script_scroll.min(max_scroll);
    }

    pub(crate) fn scroll_script_to_top(&mut self) {
        self.script_scroll = 0;
    }

    pub(crate) fn scroll_script_to_bottom(&mut self) {
        self.script_scroll = self.script_lines.len().saturating_sub(self.view_height) as u16;
    }

    pub(crate) fn reset_script_view(&mut self) {
        self.script_scroll = 0;
        self.script_lines.clear();
    }
}

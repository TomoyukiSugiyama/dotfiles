use super::dotfiles::Dotfiles;
use crate::tools::Tools;
use ratatui::widgets::ListState;

impl Dotfiles {
    pub(crate) fn select_next_tool(&mut self) {
        let previous = self.preferences.tools_settings.state.selected();
        self.preferences.tools_settings.state.select_next();
        if self.preferences.tools_settings.state.selected() != previous {
            self.reset_script_view();
        }
    }

    pub(crate) fn select_previous_tool(&mut self) {
        let previous = self.preferences.tools_settings.state.selected();
        self.preferences.tools_settings.state.select_previous();
        if self.preferences.tools_settings.state.selected() != previous {
            self.reset_script_view();
        }
    }
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

    pub(crate) fn apply_tools(&mut self, tools: Tools) {
        let previous_id = self
            .preferences
            .tools_settings
            .state
            .selected()
            .and_then(|index| {
                self.preferences
                    .tools_settings
                    .tools
                    .get_by_index(index)
                    .map(|tool| tool.id.clone())
            });

        let selected_index = previous_id
            .as_deref()
            .and_then(|target_id| tools.index_of(target_id))
            .or_else(|| tools.get_by_index(0).map(|_| 0));

        let mut state = ListState::default();
        if let Some(index) = selected_index {
            state.select(Some(index));
        }

        self.reload_error = None;
        self.reload_warning = None;
        self.preferences.tools_settings.tools = tools;
        self.preferences.tools_settings.state = state;
        self.reset_script_view();
    }

    pub(crate) fn show_reload_error(&mut self, message: String) {
        self.script_scroll = 0;
        self.script_lines.clear();
        self.reload_error = Some(message);
    }

    pub(crate) fn show_reload_warning(&mut self, message: String) {
        self.reload_error = None;
        self.reload_warning = Some(message);
    }

    pub(crate) fn clear_reload_warning(&mut self) {
        self.reload_warning = None;
    }
}

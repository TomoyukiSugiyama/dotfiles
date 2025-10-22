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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_script() {
        let mut dotfiles = Dotfiles::new();
        
        // Add some script lines
        for i in 0..20 {
            dotfiles.script_lines.push_back(format!("Line {}\n", i));
        }
        dotfiles.view_height = 10;
        
        // Test scroll down
        dotfiles.scroll_script(5);
        assert_eq!(dotfiles.script_scroll, 5);
        
        // Test scroll up
        dotfiles.scroll_script(-2);
        assert_eq!(dotfiles.script_scroll, 3);
        
        // Test scroll to bottom
        dotfiles.scroll_script_to_bottom();
        assert_eq!(dotfiles.script_scroll, 10); // 20 - 10
        
        // Test scroll to top
        dotfiles.scroll_script_to_top();
        assert_eq!(dotfiles.script_scroll, 0);
    }

    #[test]
    fn test_reset_script_view() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.script_lines.push_back("Line 1\n".to_string());
        dotfiles.script_scroll = 5;
        
        dotfiles.reset_script_view();
        
        assert_eq!(dotfiles.script_scroll, 0);
        assert!(dotfiles.script_lines.is_empty());
    }

    #[test]
    fn test_show_reload_error() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.script_lines.push_back("Line 1\n".to_string());
        dotfiles.script_scroll = 5;
        
        dotfiles.show_reload_error("Test error".to_string());
        
        assert_eq!(dotfiles.reload_error, Some("Test error".to_string()));
        assert_eq!(dotfiles.script_scroll, 0);
        assert!(dotfiles.script_lines.is_empty());
    }

    #[test]
    fn test_show_reload_warning() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.reload_error = Some("Error".to_string());
        
        dotfiles.show_reload_warning("Warning".to_string());
        
        assert!(dotfiles.reload_error.is_none());
        assert_eq!(dotfiles.reload_warning, Some("Warning".to_string()));
    }

    #[test]
    fn test_clear_reload_warning() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.reload_warning = Some("Warning".to_string());
        
        dotfiles.clear_reload_warning();
        
        assert!(dotfiles.reload_warning.is_none());
    }
}

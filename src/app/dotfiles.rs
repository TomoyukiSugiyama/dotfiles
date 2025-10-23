use crate::tools::Tools;
use ratatui::widgets::ListState;
use std::collections::VecDeque;

pub(crate) struct Dotfiles {
    pub preferences: Preferences,
    pub view: ViewTab,
    pub script_lines: VecDeque<String>,
    pub script_scroll: u16,
    pub view_height: usize,
    pub reload_error: Option<String>,
    pub reload_warning: Option<String>,
}

pub(crate) struct Preferences {
    pub tools_settings: ToolsSettings,
}

pub(crate) struct ToolsSettings {
    pub state: ListState,
    pub tools: Tools,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTab {
    Menu,
    Script,
}

impl ViewTab {
    pub fn next(self) -> Self {
        match self {
            ViewTab::Menu => ViewTab::Script,
            ViewTab::Script => ViewTab::Menu,
        }
    }
}

impl Dotfiles {
    pub(crate) fn new() -> Self {
        let mut tools_settings = ToolsSettings {
            state: ListState::default(),
            tools: Tools::new().unwrap_or_else(|error| {
                panic!("Failed to build tools: {:?}", error);
            }),
        };
        tools_settings.state.select_first();

        let preferences = Preferences { tools_settings };
        Self {
            preferences,
            view: ViewTab::Menu,
            script_lines: VecDeque::new(),
            script_scroll: 0,
            view_height: 0,
            reload_error: None,
            reload_warning: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test() -> Self {
        let mut tools_settings = ToolsSettings {
            state: ListState::default(),
            tools: Tools::new_empty(),
        };
        tools_settings.state.select_first();

        let preferences = Preferences { tools_settings };
        Self {
            preferences,
            view: ViewTab::Menu,
            script_lines: VecDeque::new(),
            script_scroll: 0,
            view_height: 0,
            reload_error: None,
            reload_warning: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_tab_next() {
        assert_eq!(ViewTab::Menu.next(), ViewTab::Script);
        assert_eq!(ViewTab::Script.next(), ViewTab::Menu);
    }

    #[test]
    fn test_dotfiles_new() {
        let dotfiles = Dotfiles::new();
        assert_eq!(dotfiles.view, ViewTab::Menu);
        assert_eq!(dotfiles.script_scroll, 0);
        assert_eq!(dotfiles.view_height, 0);
        assert!(dotfiles.reload_error.is_none());
        assert!(dotfiles.reload_warning.is_none());
        assert!(dotfiles.preferences.tools_settings.state.selected().is_some());
    }
}

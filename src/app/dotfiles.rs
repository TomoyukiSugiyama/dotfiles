use crate::tools::Tools;
use ratatui::widgets::ListState;

pub(crate) struct Dotfiles {
    pub preferences: Preferences,
    pub view: ViewTab,
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
    View,
}

impl ViewTab {
    pub fn next(self) -> Self {
        match self {
            ViewTab::Menu => ViewTab::View,
            ViewTab::View => ViewTab::Menu,
        }
    }
}

impl Dotfiles {
    pub(crate) fn new() -> Self {
        let tools_settings = ToolsSettings {
            state: ListState::default(),
            tools: Tools::new(),
        };
        let preferences = Preferences {
            tools_settings: tools_settings,
        };
        Self {
            preferences,
            view: ViewTab::Menu,
        }
    }
}

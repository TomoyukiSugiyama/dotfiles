use crate::tools::Tools;
use ratatui::widgets::ListState;

pub(crate) struct Dotfiles {
    pub preferences: Preferences,
}

pub(crate) struct Preferences {
    pub tools_settings: ToolsSettings,
}

pub(crate) struct ToolsSettings {
    pub state: ListState,
    pub tools: Tools,
}

impl Dotfiles {
    pub fn new() -> Self {
        let tools_settings = ToolsSettings {
            state: ListState::default(),
            tools: Tools::new(),
        };
        let preferences = Preferences {
            tools_settings: tools_settings,
        };
        Self { preferences }
    }
}

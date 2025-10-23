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
        let (tools, load_error) = match Tools::new() {
            Ok(tools) => (tools, None),
            Err(error) => (Tools::default(), Some(error.to_string())),
        };

        let mut tools_settings = ToolsSettings {
            state: ListState::default(),
            tools,
        };
        tools_settings.state.select_first();

        let preferences = Preferences { tools_settings };
        Self {
            preferences,
            view: ViewTab::Menu,
            script_lines: VecDeque::new(),
            script_scroll: 0,
            view_height: 0,
            reload_error: load_error,
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

    #[cfg(test)]
    pub(crate) fn new_with_test_tools() -> Self {
        let mut tools_settings = ToolsSettings {
            state: ListState::default(),
            tools: Tools::new_with_test_data(),
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
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    struct HomeEnvGuard {
        original: Option<String>,
    }

    impl HomeEnvGuard {
        fn set(path: &Path) -> Self {
            let original = std::env::var("HOME").ok();
            unsafe {
                std::env::set_var("HOME", path);
            }
            Self { original }
        }
    }

    impl Drop for HomeEnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(ref value) = self.original {
                    std::env::set_var("HOME", value);
                } else {
                    std::env::remove_var("HOME");
                }
            }
        }
    }

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
        // On startup we should not surface a reload error (default config is created automatically)
        assert!(dotfiles.reload_error.is_none());
        assert!(dotfiles.reload_warning.is_none());
        assert!(
            dotfiles
                .preferences
                .tools_settings
                .state
                .selected()
                .is_some()
        );
    }

    #[test]
    fn test_dotfiles_new_with_invalid_config_sets_reload_error() {
        let dir = tempdir().unwrap();
        let dotfiles_dir = dir.path().join(".dotfiles");
        fs::create_dir_all(&dotfiles_dir).unwrap();
        fs::write(dotfiles_dir.join("config.yaml"), "invalid: [").unwrap();
        let _home_guard = HomeEnvGuard::set(dir.path());

        let dotfiles = Dotfiles::new();

        let error = dotfiles
            .reload_error
            .expect("expected reload_error to contain failure message");
        assert!(error.contains("Failed to load config"));
        assert_eq!(dotfiles.preferences.tools_settings.tools.iter().count(), 0);
    }
}

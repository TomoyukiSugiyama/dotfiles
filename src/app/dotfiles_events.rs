use super::dotfiles::Dotfiles;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use super::dotfiles::ViewTab;

impl Dotfiles {
    pub(crate) fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => self.view = self.view.next(),
            (_, KeyCode::Up) => if self.view == ViewTab::Menu {
                let previous = self
                    .preferences
                    .tools_settings
                    .state
                    .selected();
                self.preferences.tools_settings.state.select_previous();
                if self
                    .preferences
                    .tools_settings
                    .state
                    .selected()
                    != previous
                {
                    self.reset_script_view();
                }
            } else {
                self.scroll_script(-1)
            },
            (_, KeyCode::Down) => if self.view == ViewTab::Menu {
                let previous = self
                    .preferences
                    .tools_settings
                    .state
                    .selected();
                self.preferences.tools_settings.state.select_next();
                if self
                    .preferences
                    .tools_settings
                    .state
                    .selected()
                    != previous
                {
                    self.reset_script_view();
                }
            } else {
                self.scroll_script(1)
            },
            (_, KeyCode::Home) => if self.view == ViewTab::Script {
                self.scroll_script_to_top()
            },
            (_, KeyCode::End) => if self.view == ViewTab::Script {
                self.scroll_script_to_bottom()
            },
            _ => {}
        }
    }
}

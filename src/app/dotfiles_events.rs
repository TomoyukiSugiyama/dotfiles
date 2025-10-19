use super::dotfiles::Dotfiles;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

impl Dotfiles {
    pub(crate) fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => self.view = self.view.next(),
            (_, KeyCode::Up) => self.preferences.tools_settings.state.select_previous(),
            (_, KeyCode::Down) => self.preferences.tools_settings.state.select_next(),
            _ => {}
        }
    }
}

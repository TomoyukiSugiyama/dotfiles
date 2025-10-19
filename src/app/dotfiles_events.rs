use super::dotfiles::Dotfiles;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

impl Dotfiles {
    pub(crate) fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Up) => self.todo(),
            (_, KeyCode::Down) => self.todo(),
            (_, KeyCode::Enter) => self.todo(),
            (_, KeyCode::Home) => self.todo(),
            (_, KeyCode::End) => self.todo(),
            _ => {}
        }
    }
}

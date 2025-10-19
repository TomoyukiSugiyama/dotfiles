use super::dotfiles::Dotfiles;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

impl Dotfiles {
    pub(crate) fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => self.view = self.view.next(),
            _ => {}
        }
    }
}

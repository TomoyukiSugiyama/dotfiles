use super::dotfiles::Dotfiles;
use super::dotfiles::ViewTab;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

impl Dotfiles {
    pub(crate) fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => self.view = self.view.next(),
            (_, KeyCode::Up) => {
                if self.view == ViewTab::Menu {
                    self.select_previous_tool();
                } else {
                    self.scroll_script(-1)
                }
            }
            (_, KeyCode::Down) => {
                if self.view == ViewTab::Menu {
                    self.select_next_tool();
                } else {
                    self.scroll_script(1)
                }
            }
            (_, KeyCode::Home) => {
                if self.view == ViewTab::Script {
                    self.scroll_script_to_top()
                }
            }
            (_, KeyCode::End) => {
                if self.view == ViewTab::Script {
                    self.scroll_script_to_bottom()
                }
            }
            _ => {}
        }
    }
}

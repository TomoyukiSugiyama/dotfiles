use super::execute::Execute;
use super::execute::ViewTab;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

impl Execute {
    /// Handles the key events and updates the state of [`App`].
    pub fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Add other key handlers here.
            (_, KeyCode::Home) => {
                if self.view == ViewTab::Menu {
                    self.menu.select_first()
                } else {
                    self.scroll_log_to_top()
                }
            }
            (_, KeyCode::End) => {
                if self.view == ViewTab::Menu {
                    self.menu.select_last()
                } else {
                    self.scroll_log_to_bottom()
                }
            }
            (_, KeyCode::Up) => {
                if self.view == ViewTab::Menu {
                    self.menu.select_previous()
                } else {
                    self.scroll_log(-1)
                }
            }
            (_, KeyCode::Down) => {
                if self.view == ViewTab::Menu {
                    self.menu.select_next()
                } else {
                    self.scroll_log(1)
                }
            }
            (_, KeyCode::Enter) => {
                if self.view == ViewTab::Menu {
                    self.execute_selected()
                }
            }
            (_, KeyCode::Tab) => self.view = self.view.next(),
            _ => {}
        }
    }
}

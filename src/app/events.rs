use super::App;
use super::tabs::SelectedTab;
use super::ui::ViewTab;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

impl App {
    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    pub(crate) fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Left) => self.select_previous_tab(),
            (_, KeyCode::Right) => self.select_next_tab(),
            _ => {
                if self.selected_tab == SelectedTab::Dotfiles {
                    self.on_key_event_dotfiles(key)
                } else {
                    self.on_key_event_execute(key)
                }
            }
        }
    }

    fn on_key_event_dotfiles(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Up) => self.todo(),
            (_, KeyCode::Down) => self.todo(),
            (_, KeyCode::Enter) => self.todo(),
            (_, KeyCode::Home) => self.todo(),
            (_, KeyCode::End) => self.todo(),
            _ => {}
        }
    }
    /// Handles the key events and updates the state of [`App`].
    fn on_key_event_execute(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            // Add other key handlers here.
            (_, KeyCode::Home) => {
                if self.view == ViewTab::Menu {
                    self.select_first()
                } else {
                    self.scroll_log_to_top()
                }
            }
            (_, KeyCode::End) => {
                if self.view == ViewTab::Menu {
                    self.select_last()
                } else {
                    self.scroll_log_to_bottom()
                }
            }
            (_, KeyCode::Up) => {
                if self.view == ViewTab::Menu {
                    self.select_previous()
                } else {
                    self.scroll_log(-1)
                }
            }
            (_, KeyCode::Down) => {
                if self.view == ViewTab::Menu {
                    self.select_next()
                } else {
                    self.scroll_log(1)
                }
            }
            (_, KeyCode::Enter) => {
                if self.view == ViewTab::Menu {
                    self.execute_selected()
                }
            }
            (_, KeyCode::Tab) => self.select_next_view(),
            _ => {}
        }
    }

    fn select_previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn select_next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }
    fn select_first(&mut self) {
        self.menu.state.select_first();
    }

    fn select_last(&mut self) {
        self.menu.state.select_last();
    }

    fn select_previous(&mut self) {
        self.menu.state.select_previous();
    }

    fn select_next(&mut self) {
        self.menu.state.select_next();
    }

    fn select_next_view(&mut self) {
        self.view = self.view.next();
    }

    fn todo(&mut self) {
        todo!()
    }
}

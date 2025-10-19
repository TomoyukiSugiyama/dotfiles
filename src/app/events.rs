use super::App;
use super::tabs::SelectedTab;
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
                    self.execute.on_key_event(key)
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

    fn select_previous_tab(&mut self) {
        self.selected_tab = self.selected_tab.previous();
    }

    fn select_next_tab(&mut self) {
        self.selected_tab = self.selected_tab.next();
    }

    fn todo(&mut self) {
        todo!()
    }
}

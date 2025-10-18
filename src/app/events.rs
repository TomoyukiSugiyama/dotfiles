use super::App;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;
use color_eyre::Result;

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

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Home) => self.select_first(),
            (_, KeyCode::End) => self.select_last(),
            (_, KeyCode::Up) => self.select_previous(),
            (_, KeyCode::Down) => self.select_next(),
            (_, KeyCode::Enter | KeyCode::Right) => self.execute_selected(),
            (_, KeyCode::Left) => self.unselect(),
            _ => {}
        }
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

    fn unselect(&mut self) {
        self.menu.state.select(None);
    }
}
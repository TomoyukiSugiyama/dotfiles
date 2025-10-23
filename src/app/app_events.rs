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
            (_, KeyCode::Left) => self.selected_tab.select_previous_tab(),
            (_, KeyCode::Right) => self.selected_tab.select_next_tab(),
            (_, KeyCode::Char('r' | 'R')) => {
                if let Err(message) = self.reload_config() {
                    self.dotfiles.show_reload_error(message.clone());
                    self.workflow.show_reload_error(message);
                }
            }
            _ => {
                if self.selected_tab == SelectedTab::Dotfiles {
                    self.dotfiles.on_key_event(key)
                } else {
                    self.workflow.on_key_event(key)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_key_event_quit() {
        let mut app = App::new();
        assert!(app.running);

        // Test 'q' key
        app.on_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(!app.running);

        // Reset and test Esc key
        app.running = true;
        app.on_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(!app.running);

        // Reset and test Ctrl+C
        app.running = true;
        app.on_key_event(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!app.running);
    }

    #[test]
    fn test_on_key_event_tab_navigation() {
        let mut app = App::new();
        let initial_tab = app.selected_tab;

        // Test Right key
        app.on_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_ne!(app.selected_tab, initial_tab);

        // Test Left key
        app.on_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(app.selected_tab, initial_tab);
    }

    #[test]
    fn test_on_key_event_ctrl_c() {
        let mut app = App::new();
        assert!(app.running);

        // Test Ctrl+C (capital C)
        app.on_key_event(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL));
        assert!(!app.running);
    }

    #[test]
    fn test_on_key_event_reload() {
        let mut app = App::new();

        // Test 'r' key for reload (should not crash)
        app.on_key_event(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));

        // Test 'R' key for reload (should not crash)
        app.on_key_event(KeyEvent::new(KeyCode::Char('R'), KeyModifiers::NONE));
    }
}

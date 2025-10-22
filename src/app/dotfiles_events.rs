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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_dotfiles_on_key_event_tab() {
        let mut dotfiles = Dotfiles::new();
        assert_eq!(dotfiles.view, ViewTab::Menu);
        
        dotfiles.on_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(dotfiles.view, ViewTab::Script);
        
        dotfiles.on_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(dotfiles.view, ViewTab::Menu);
    }

    #[test]
    fn test_dotfiles_on_key_event_menu_navigation() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.view = ViewTab::Menu;
        let initial_selection = dotfiles.preferences.tools_settings.state.selected();
        
        // Test Down key
        dotfiles.on_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        // Selection might change depending on available tools
        
        // Test Up key
        dotfiles.on_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(dotfiles.preferences.tools_settings.state.selected(), initial_selection);
    }

    #[test]
    fn test_dotfiles_on_key_event_script_scroll() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.view = ViewTab::Script;
        
        // Add some script lines
        for i in 0..20 {
            dotfiles.script_lines.push_back(format!("Line {}\n", i));
        }
        dotfiles.view_height = 10;
        dotfiles.script_scroll = 5;
        
        // Test Down key
        dotfiles.on_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(dotfiles.script_scroll, 6);
        
        // Test Up key
        dotfiles.on_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(dotfiles.script_scroll, 5);
        
        // Test Home key
        dotfiles.on_key_event(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(dotfiles.script_scroll, 0);
        
        // Test End key
        dotfiles.on_key_event(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(dotfiles.script_scroll, 10); // 20 - 10
    }
}

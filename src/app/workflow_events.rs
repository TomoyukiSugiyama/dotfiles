use super::workflow::ViewTab;
use super::workflow::Workflow;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;

impl Workflow {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_workflow_on_key_event_tab() {
        let mut workflow = Workflow::new_for_test();
        assert_eq!(workflow.view, ViewTab::Menu);
        
        workflow.on_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(workflow.view, ViewTab::Log);
        
        workflow.on_key_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(workflow.view, ViewTab::Menu);
    }

    #[test]
    fn test_workflow_on_key_event_menu_navigation() {
        let mut workflow = Workflow::new_for_test();
        workflow.view = ViewTab::Menu;
        
        // Test Home key
        workflow.on_key_event(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(workflow.menu.state.selected(), Some(0));
        
        // Test Down key
        workflow.on_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        // Selection should change
        
        // Test Up key
        workflow.on_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(workflow.menu.state.selected(), Some(0));
    }

    #[test]
    fn test_workflow_on_key_event_log_scroll() {
        let mut workflow = Workflow::new_for_test();
        workflow.view = ViewTab::Log;
        
        // Add some log lines
        for i in 0..20 {
            workflow.log_lines.push_back(format!("Line {}\n", i));
        }
        workflow.view_height = 10;
        workflow.log_scroll = 5;
        
        // Test Down key
        workflow.on_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(workflow.log_scroll, 6);
        
        // Test Up key
        workflow.on_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(workflow.log_scroll, 5);
        
        // Test Home key
        workflow.on_key_event(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(workflow.log_scroll, 0);
        
        // Test End key
        workflow.on_key_event(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(workflow.log_scroll, 10); // 20 - 10
    }
}

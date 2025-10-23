use super::App;
use super::tabs::SelectedTab;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};

impl App {
    fn render_title(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Dotfiles Manager")
            .centered()
            .render(area, buffer);
    }

    fn render_inner(&mut self, area: Rect, buffer: &mut Buffer) {
        match self.selected_tab {
            SelectedTab::Dotfiles => self.dotfiles.render(area, buffer),
            SelectedTab::Workflow => self.workflow.render(area, buffer),
        }
    }

    fn render_footer(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(
            "Use ←/→ to switch tabs, ↓/↑ to move, Tab to change pane, Enter to run, R to reload config, Home/End to jump, q/Esc to quit.",
        )
            .centered()
            .render(area, buffer);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, inner_area, footer_area] = Layout::vertical([
            Constraint::Percentage(5),
            Constraint::Percentage(90),
            Constraint::Percentage(5),
        ])
        .areas(area);

        let [tabs_area, title_area] =
            Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                .areas(header_area);

        self.render_title(title_area, buffer);
        self.selected_tab.render(tabs_area, buffer);
        self.render_inner(inner_area, buffer);
        self.render_footer(footer_area, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn buffer_to_string(backend: &TestBackend) -> String {
        let buffer = backend.buffer();
        let area = buffer.area();
        let mut result = String::new();

        for y in 0..area.height {
            for x in 0..area.width {
                let cell = buffer.cell((x, y)).expect("valid cell position");
                result.push_str(cell.symbol());
            }
            if y < area.height - 1 {
                result.push('\n');
            }
        }

        result
    }

    #[test]
    fn test_render_app_workflow_tab() {
        let mut app = App::new();
        app.selected_tab = SelectedTab::Workflow;
        
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let result = terminal.draw(|frame| frame.render_widget(&mut app, frame.area()));
        
        // Just verify rendering doesn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_app_dotfiles_tab() {
        let mut app = App::new();
        app.selected_tab = SelectedTab::Dotfiles;
        
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let result = terminal.draw(|frame| frame.render_widget(&mut app, frame.area()));
        
        // Just verify rendering doesn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_app_workflow_tab_with_tools() {
        let mut app = App::new_with_test_tools();
        app.selected_tab = SelectedTab::Workflow;

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_app_dotfiles_tab_with_tools() {
        let mut app = App::new_with_test_tools();
        app.selected_tab = SelectedTab::Dotfiles;

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }
}

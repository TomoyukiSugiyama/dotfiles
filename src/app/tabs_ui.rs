use super::tabs::SelectedTab;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Tabs;
use ratatui::widgets::Widget;
use strum::IntoEnumIterator;

impl SelectedTab {
    pub(crate) fn title(self) -> Line<'static> {
        let title = match self {
            SelectedTab::Dotfiles => "Dotfiles",
            SelectedTab::Workflow => "Workflow",
        };
        Line::from(format!("  {title}  "))
    }

    pub(crate) fn render_tabs(self, area: Rect, buffer: &mut Buffer) {
        let titles = SelectedTab::iter()
            .map(|tab| tab.title())
            .collect::<Vec<Line>>();
        Tabs::new(titles)
            .style(Style::new().fg(Color::White))
            .highlight_style(Style::new().fg(Color::Black).bg(Color::White))
            .select(self as usize)
            .render(area, buffer);
    }
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.render_tabs(area, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_render_tabs_workflow_selected() {
        let tab = SelectedTab::Workflow;
        let mut terminal = Terminal::new(TestBackend::new(50, 3)).unwrap();
        let result = terminal.draw(|frame| frame.render_widget(tab, frame.area()));
        
        // Just verify rendering doesn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_tabs_dotfiles_selected() {
        let tab = SelectedTab::Dotfiles;
        let mut terminal = Terminal::new(TestBackend::new(50, 3)).unwrap();
        let result = terminal.draw(|frame| frame.render_widget(tab, frame.area()));
        
        // Just verify rendering doesn't panic
        assert!(result.is_ok());
    }
}

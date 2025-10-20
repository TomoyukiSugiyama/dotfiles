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

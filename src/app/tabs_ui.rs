use super::tabs::SelectedTab;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;

impl SelectedTab {
    pub(crate) fn title(self) -> Line<'static> {
        let title = match self {
            SelectedTab::Dotfiles => "Dotfiles",
            SelectedTab::Execute => "Execute",
        };
        let line = Line::from(format!("  {title}  "));
        line.style(Style::new().fg(Color::White).bg(Color::Black))
    }

    pub(crate) fn next(self) -> Self {
        let tab = match self {
            SelectedTab::Dotfiles => SelectedTab::Execute,
            SelectedTab::Execute => SelectedTab::Dotfiles,
        };
        tab
    }
    pub(crate) fn previous(self) -> Self {
        let tab = match self {
            SelectedTab::Dotfiles => SelectedTab::Execute,
            SelectedTab::Execute => SelectedTab::Dotfiles,
        };
        tab
    }
}

// impl Widget for SelectedTab {
//     fn render(self, area: Rect, buffer: &mut Buffer) {
//         // if self.tab == Tab::Dotfiles {
//         //     self.execute.render(area, buffer)
//         // } else {
//         //     self.execute.render(area, buffer)
//         // }
//     }
// }

use ratatui::buffer::Buffer;
use ratatui::{layout::Rect, style::Color, style::Style, text::Line, widgets::Widget};
use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, EnumIter)]
pub(crate) enum SelectedTab {
    #[default]
    #[strum(to_string = "Dotfiles")]
    Dotfiles,
    #[strum(to_string = "Execute")]
    Execute,
}

impl SelectedTab {
    pub(crate) fn title(self) -> Line<'static> {
        let line = Line::from(format!("  {self}  "));
        line.style(Style::new().fg(Color::White).bg(Color::Black))
    }

    pub(crate) fn next(self) -> Self {
        match self {
            SelectedTab::Dotfiles => SelectedTab::Execute,
            SelectedTab::Execute => SelectedTab::Dotfiles,
        }
    }
    pub(crate) fn previous(self) -> Self {
        match self {
            SelectedTab::Dotfiles => SelectedTab::Execute,
            SelectedTab::Execute => SelectedTab::Dotfiles,
        }
    }
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        match self {
            SelectedTab::Dotfiles => "Dotfiles".render(area, buffer),
            SelectedTab::Execute => "Execute".render(area, buffer),
        }
    }
}

use super::App;
use ratatui::{
    buffer::Buffer,
    layout::{Rect, Constraint, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem, HighlightSpacing, StatefulWidget, Widget},
    text::Line,
    style::Style,
    symbols,
    style::Modifier,
    style::Color,
    style::palette::tailwind::SLATE,
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl App {
    fn render_header(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Dotfiles Manager")
            .centered()
            .render(area, buffer);
    }
    fn render_footer(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to select, Home/End to go top/bottom.")
            .centered()
            .render(area, buffer);
    }

    fn render_log(&mut self, area: Rect, buffer: &mut Buffer) {
        let block = Block::new()
            .title(Line::from("Log"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::Black));
        self.view_height = area.height as usize;
        let text: String = self
            .log_lines
            .iter()
            .skip(self.log_scroll as usize)
            .take(self.view_height)
            .cloned()
            .collect();
        Paragraph::new(text).block(block).render(area, buffer);
    }


    fn render_menu(&mut self, area: Rect, buffer: &mut Buffer) {

        let block = Block::new()
            .title(Line::from("Menu"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::Black));
        let items = self
            .menu
            .items
            .iter()
            .map(|item| ListItem::new(item.title.clone()))
            .collect::<Vec<ListItem>>();
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(list, area, buffer, &mut self.menu.state);
    }


}

impl Widget for &mut App {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, footer_area, menu_area, log_area] = Layout::vertical([
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(30),
            Constraint::Percentage(60),
        ])
        .areas(area);

        self.render_header(header_area, buffer);
        self.render_footer(footer_area, buffer);
        self.render_log(log_area, buffer);
        self.render_menu(menu_area, buffer);
    }
}
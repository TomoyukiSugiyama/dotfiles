use super::execute::Execute;
use super::execute::ViewTab;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget,
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl Execute {
    fn render_log(&mut self, area: Rect, buffer: &mut Buffer, focused: bool) {
        let mut block = Block::new()
            .title(Line::from("Log"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));
        if focused {
            block = block.border_style(Style::new().fg(Color::Yellow));
        }

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

    fn render_menu(&mut self, area: Rect, buffer: &mut Buffer, focused: bool) {
        let mut block = Block::new()
            .title(Line::from("Menu"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));
        if focused {
            block = block.border_style(Style::new().fg(Color::Yellow));
        }

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

impl Widget for &mut Execute {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [menu_area, log_area] =
            Layout::vertical([Constraint::Percentage(10), Constraint::Percentage(90)]).areas(area);

        self.render_log(log_area, buffer, self.view == ViewTab::Log);
        self.render_menu(menu_area, buffer, self.view == ViewTab::Menu);
    }
}

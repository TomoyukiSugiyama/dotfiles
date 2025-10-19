use super::dotfiles::Dotfiles;
use super::dotfiles::ViewTab;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Modifier;
use ratatui::style::palette::tailwind::SLATE;
use ratatui::style::{Color, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::Widget;
use ratatui::widgets::{
    Block, Borders, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget,
};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl Dotfiles {
    fn render_menu(&mut self, area: Rect, buffer: &mut Buffer) {
        let mut block = Block::new()
            .title(Line::from("Preferences"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));

        if self.view == ViewTab::Menu {
            block = block.border_style(Style::new().fg(Color::Yellow));
        }

        let items = self
            .preferences
            .tools_settings
            .tools
            .items
            .iter()
            .map(|item| ListItem::new(item.name.clone()))
            .collect::<Vec<ListItem>>();
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(
            list,
            area,
            buffer,
            &mut self.preferences.tools_settings.state,
        );
    }

    fn render_view(&mut self, area: Rect, buffer: &mut Buffer) {
        let mut block = Block::new()
            .title(Line::from("View"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));

        if self.view == ViewTab::View {
            block = block.border_style(Style::new().fg(Color::Yellow));
        }

        let text = self
            .preferences
            .tools_settings
            .state
            .selected()
            .and_then(|index| self.preferences.tools_settings.tools.items.get(index))
            .map(|item| {
                let script = self
                    .preferences
                    .tools_settings
                    .tools
                    .raw_script(item)
                    .unwrap_or_else(|| "(Failed to read script)".to_string());

                let formatted_script = if script.trim().is_empty() {
                    "  (File is empty)".to_string()
                } else {
                    script
                        .lines()
                        .map(|line| format!("  {line}"))
                        .collect::<Vec<String>>()
                        .join("\n")
                };

                format!(
                    "Tool: {}\nPath: {}\n\nScript:\n{}",
                    item.name,
                    self.preferences.tools_settings.tools.file_path(item),
                    formatted_script
                )
            })
            .unwrap_or_else(|| "Select a tool to view its details.".to_string());

        Paragraph::new(text).block(block).render(area, buffer);
    }
}

impl Widget for &mut Dotfiles {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [menu_area, view_area] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(area);
        self.render_menu(menu_area, buffer);
        self.render_view(view_area, buffer);
    }
}

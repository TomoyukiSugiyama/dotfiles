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

        let inner_block = block.clone();
        block.render(area, buffer);
        let inner = inner_block.inner(area);

        let [label_area, list_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(inner);

        Paragraph::new("Tools Settings")
            .style(Style::new().fg(SLATE.c200))
            .render(label_area, buffer);

        let items = self
            .preferences
            .tools_settings
            .tools
            .iter()
            .map(|item| ListItem::new(item.display_name()))
            .collect::<Vec<ListItem>>();
        let list = List::new(items)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(
            list,
            list_area,
            buffer,
            &mut self.preferences.tools_settings.state,
        );
    }

    fn render_view(&mut self, area: Rect, buffer: &mut Buffer) {
        let block = Block::new()
            .title(Line::from("Tool Details"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));

        let inner_block = block.clone();
        block.render(area, buffer);
        let inner = inner_block.inner(area);

        let Some(selected_tool) = self
            .preferences
            .tools_settings
            .state
            .selected()
            .and_then(|index| self.preferences.tools_settings.tools.get_by_index(index))
        else {
            Paragraph::new("Select a tool to view its details.").render(inner, buffer);
            return;
        };

        let tools = &self.preferences.tools_settings.tools;
        let stage = tools.execution_stage_index(&selected_tool.id);
        let dependency_map = tools.dependency_map_lines(Some(&selected_tool.id));
        let script = tools
            .raw_script(selected_tool)
            .unwrap_or_else(|| "(Failed to read script)".to_string());

        let stage_text = stage.map_or("(unknown)".to_string(), |index| {
            format!("Stage {}", index + 1)
        });

        let info_text = format!(
            "Tool: {}\nID: {}\nPath: {}\nOrder: {}",
            selected_tool.name,
            selected_tool.id,
            tools.file_path(selected_tool),
            stage_text,
        );

        let dependency_map_text = dependency_map.join("\n");
        let chunks = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length((dependency_map_text.lines().count() + 2) as u16),
            Constraint::Min(3),
        ])
        .split(inner);
        Paragraph::new(info_text).render(chunks[0], buffer);

        let map_block = Block::new()
            .title(Line::from("Dependency Map (* current tool)"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));
        Paragraph::new(dependency_map_text)
            .block(map_block)
            .render(chunks[1], buffer);

        let mut script_block = Block::new()
            .title(Line::from("Script"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));
        if self.view == ViewTab::Script {
            script_block = script_block.border_style(Style::new().fg(Color::Yellow));
        }
        self.view_height = script_block.inner(chunks[2]).height as usize;
        self.script_lines = script.lines().map(|line| format!("  {line}")).collect();
        let text = self
            .script_lines
            .iter()
            .skip(self.script_scroll as usize)
            .take(self.view_height)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        Paragraph::new(text)
            .block(script_block)
            .render(chunks[2], buffer);
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

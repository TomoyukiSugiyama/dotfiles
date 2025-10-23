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

struct ToolViewData {
    info_text: String,
    dependency_map_text: String,
    dependency_map_height: u16,
    script: String,
}

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

        self.render_label(label_area, buffer);

        self.render_list(list_area, buffer);
    }

    fn render_label(&mut self, area: Rect, buffer: &mut Buffer) {
        let label = "Tools Settings";
        Paragraph::new(label)
            .style(Style::new().fg(SLATE.c200))
            .render(area, buffer);
    }
    fn render_list(&mut self, area: Rect, buffer: &mut Buffer) {
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
            area,
            buffer,
            &mut self.preferences.tools_settings.state,
        );
    }
    fn render_view(&mut self, area: Rect, buffer: &mut Buffer) {
        let inner = Self::render_tool_details_container(area, buffer);

        if self.render_reload_error(inner, buffer) {
            return;
        }

        let Some(data) = self.build_tool_view_data() else {
            self.render_tool_selection_prompt(inner, buffer);
            return;
        };

        let chunks = self.tool_details_layout(inner, &data);
        let mut chunk_index = 0;

        if let Some(message) = self.reload_warning.as_ref() {
            Paragraph::new(message.as_str())
                .style(Style::new().fg(Color::Yellow))
                .render(chunks[chunk_index], buffer);
            chunk_index += 1;
        }

        self.render_info_section(chunks[chunk_index], buffer, &data);
        chunk_index += 1;

        self.render_dependency_map_section(chunks[chunk_index], buffer, &data);
        chunk_index += 1;

        self.render_script_section(chunks[chunk_index], buffer, data.script.as_str());
    }

    fn render_tool_details_container(area: Rect, buffer: &mut Buffer) -> Rect {
        let block = Block::new()
            .title(Line::from("Tool Details"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));

        let inner_block = block.clone();
        block.render(area, buffer);
        inner_block.inner(area)
    }

    fn render_reload_error(&self, area: Rect, buffer: &mut Buffer) -> bool {
        if let Some(message) = self.reload_error.as_ref() {
            Paragraph::new(message.as_str())
                .style(Style::new().fg(Color::Red))
                .render(area, buffer);
            true
        } else {
            false
        }
    }

    fn render_tool_selection_prompt(&self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Select a tool to view its details.").render(area, buffer);
    }

    fn build_tool_view_data(&self) -> Option<ToolViewData> {
        let tools_settings = &self.preferences.tools_settings;
        let tools_state = &tools_settings.state;
        let tools = &tools_settings.tools;

        let selected_index = tools_state.selected()?;
        let selected_tool = tools.get_by_index(selected_index)?;

        let stage_text = tools
            .execution_stage_index(&selected_tool.id)
            .map_or("(unknown)".to_string(), |index| {
                format!("Stage {}", index + 1)
            });

        let info_text = format!(
            "Tool: {}\nID: {}\nPath: {}\nOrder: {}",
            selected_tool.name,
            selected_tool.id,
            tools.file_path(selected_tool),
            stage_text,
        );

        let dependency_map_text = tools
            .dependency_map_lines(Some(&selected_tool.id))
            .join("\n");

        let dependency_map_height = (dependency_map_text.lines().count() + 2) as u16;

        let script = tools
            .raw_script(selected_tool)
            .unwrap_or_else(|| "(Failed to read script)".to_string());

        Some(ToolViewData {
            info_text,
            dependency_map_text,
            dependency_map_height,
            script,
        })
    }

    fn tool_details_layout(&self, area: Rect, data: &ToolViewData) -> Vec<Rect> {
        let mut constraints = Vec::new();
        if let Some(message) = self.reload_warning.as_ref() {
            let lines = message.lines().count().max(1) as u16 + 1;
            constraints.push(Constraint::Length(lines));
        }
        const INFO_SECTION_HEIGHT: u16 = 5;
        const SCRIPT_SECTION_MIN_HEIGHT: u16 = 3;
        constraints.extend([
            Constraint::Length(INFO_SECTION_HEIGHT),
            Constraint::Length(data.dependency_map_height),
            Constraint::Min(SCRIPT_SECTION_MIN_HEIGHT),
        ]);

        Layout::vertical(constraints)
            .split(area)
            .iter()
            .copied()
            .collect()
    }

    fn render_info_section(&self, area: Rect, buffer: &mut Buffer, data: &ToolViewData) {
        Paragraph::new(data.info_text.as_str()).render(area, buffer);
    }

    fn render_dependency_map_section(&self, area: Rect, buffer: &mut Buffer, data: &ToolViewData) {
        let map_block = Block::new()
            .title(Line::from("Dependency Map (* current tool)"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));

        Paragraph::new(data.dependency_map_text.as_str())
            .block(map_block)
            .render(area, buffer);
    }

    fn render_script_section(&mut self, area: Rect, buffer: &mut Buffer, script: &str) {
        let mut script_block = Block::new()
            .title(Line::from("Script"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::White));

        if self.view == ViewTab::Script {
            script_block = script_block.border_style(Style::new().fg(Color::Yellow));
        }

        self.view_height = script_block.inner(area).height as usize;
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
            .render(area, buffer);
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

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
    fn test_render_dotfiles_menu_view() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.view = ViewTab::Menu;

        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let result = terminal.draw(|frame| frame.render_widget(&mut dotfiles, frame.area()));

        // Just verify rendering doesn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_render_dotfiles_script_view() {
        let mut dotfiles = Dotfiles::new();
        dotfiles.view = ViewTab::Script;

        // Add some script lines for testing
        dotfiles.script_lines.push_back("#!/bin/zsh\n".to_string());
        dotfiles
            .script_lines
            .push_back("echo 'Hello World'\n".to_string());

        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let result = terminal.draw(|frame| frame.render_widget(&mut dotfiles, frame.area()));

        // Just verify rendering doesn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_dotfiles_empty_menu() {
        let mut dotfiles = Dotfiles::new_for_test();
        dotfiles.view = ViewTab::Menu;

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_dotfiles_empty_script() {
        let mut dotfiles = Dotfiles::new_for_test();
        dotfiles.view = ViewTab::Script;

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_dotfiles_with_error() {
        let mut dotfiles = Dotfiles::new_for_test();
        dotfiles.view = ViewTab::Menu;
        dotfiles.show_reload_error("Failed to load configuration".to_string());

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_dotfiles_menu_focused() {
        let mut dotfiles = Dotfiles::new_with_test_tools();
        dotfiles.view = ViewTab::Menu; // Menu is focused (yellow border)

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_dotfiles_script_focused() {
        let mut dotfiles = Dotfiles::new_with_test_tools();
        dotfiles.view = ViewTab::Script; // Script is focused (yellow border)

        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    // Tests with actual tool data (deterministic)
    #[test]
    fn test_snapshot_dotfiles_with_tools_menu() {
        let mut dotfiles = Dotfiles::new_with_test_tools();
        dotfiles.view = ViewTab::Menu;
        // First tool (Brew) is selected by default

        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_dotfiles_with_tools_script_view() {
        let mut dotfiles = Dotfiles::new_with_test_tools();
        dotfiles.view = ViewTab::Script;
        // First tool (Brew) is selected by default

        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_dotfiles_with_last_tool_selected() {
        let mut dotfiles = Dotfiles::new_with_test_tools();
        dotfiles.view = ViewTab::Menu;

        // Select last tool (Zsh - has multiple dependencies)
        dotfiles.preferences.tools_settings.state.select(Some(5));

        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn test_snapshot_dotfiles_with_warning_and_tools() {
        let mut dotfiles = Dotfiles::new_with_test_tools();
        dotfiles.view = ViewTab::Menu;
        dotfiles.show_reload_warning("Configuration has been reloaded with warnings".to_string());

        let backend = TestBackend::new(120, 35);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| frame.render_widget(&mut dotfiles, frame.area()))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend());
        insta::assert_snapshot!(rendered);
    }
}

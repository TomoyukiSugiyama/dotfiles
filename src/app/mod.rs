mod actions;
mod events;
mod execute;
mod execute_ui;
mod menu;
mod tabs;
mod tabs_ui;
mod ui;

use color_eyre::Result;
use execute::Execute;
use ratatui::DefaultTerminal;
use tabs::SelectedTab;

pub(crate) struct App {
    /// Is the application running?
    running: bool,
    pub execute: Execute,
    pub selected_tab: SelectedTab,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            running: true,
            execute: Execute::new(),
            selected_tab: SelectedTab::new(),
        }
    }

    /// Run the application's main loop.
    pub(crate) fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            self.drain_log_messages();
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }
}

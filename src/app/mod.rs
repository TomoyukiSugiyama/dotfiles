mod app_actions;
mod app_ui;
mod dotfiles;
mod dotfiles_ui;
mod app_events;
mod execute;
mod execute_actions;
mod execute_events;
mod execute_log;
mod execute_menu;
mod execute_ui;
mod tabs;
mod tabs_ui;

use color_eyre::Result;
use dotfiles::Dotfiles;
use execute::Execute;
use ratatui::DefaultTerminal;
use tabs::SelectedTab;

pub(crate) struct App {
    /// Is the application running?
    running: bool,
    pub execute: Execute,
    pub dotfiles: Dotfiles,
    pub selected_tab: SelectedTab,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            running: true,
            execute: Execute::new(),
            dotfiles: Dotfiles::new(),
            selected_tab: SelectedTab::new(),
        }
    }

    /// Run the application's main loop.
    pub(crate) fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            self.execute.drain_log_messages();
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }
}

mod app_actions;
mod app_events;
mod app_ui;
mod dotfiles;
mod dotfiles_actions;
mod dotfiles_events;
mod dotfiles_ui;
mod workflow;
mod workflow_actions;
mod workflow_events;
mod workflow_log;
mod workflow_menu;
mod workflow_ui;
mod tabs;
mod tabs_ui;

use color_eyre::Result;
use workflow::Workflow;
use ratatui::DefaultTerminal;
use tabs::SelectedTab;

pub(crate) use dotfiles::Dotfiles;

pub(crate) struct App {
    /// Is the application running?
    running: bool,
    pub workflow: Workflow,
    pub dotfiles: Dotfiles,
    pub selected_tab: SelectedTab,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            running: true,
            workflow: Workflow::new(),
            dotfiles: Dotfiles::new(),
            selected_tab: SelectedTab::new(),
        }
    }

    /// Run the application's main loop.
    pub(crate) fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            self.workflow.drain_log_messages();
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }
}

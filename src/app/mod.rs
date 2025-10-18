mod menu;
mod ui;
mod events;
mod actions;

use std::collections::VecDeque;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use menu::{Menu, MenuItemAction};
use ratatui::DefaultTerminal;
use color_eyre::Result;

pub(crate) struct App {
    /// Is the application running?
    running: bool,
    menu: Menu,
    runtime: Runtime,
    log_sender: mpsc::UnboundedSender<String>,
    pub log_receiver: mpsc::UnboundedReceiver<String>,
    tools: super::tools::Tools,
    pub log_lines: VecDeque<String>,
    pub log_scroll: u16,
    pub view_height: usize,
}

impl App {
    pub(crate) fn new() -> Self {
        let tools = super::tools::Tools::new();
        let runtime = Runtime::new().expect("failed to start tokio runtime");
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        Self {
            running: true,
            menu: Menu::from_iter([
                (
                    "Update Dotfiles".to_string(),
                    Some(MenuItemAction::UpdateDotfiles),
                ),
                ("Quit".to_string(), Some(MenuItemAction::Quit)),
            ]),
            runtime,
            log_sender,
            log_receiver,
            tools,
            log_lines: VecDeque::new(),
            log_scroll: 0,
            view_height: 0,
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
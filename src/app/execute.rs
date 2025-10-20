use super::execute_menu::Menu;
use crate::tools::Tools;

use super::execute_menu::MenuItemAction;
use std::collections::VecDeque;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTab {
    Menu,
    Log,
}

impl ViewTab {
    pub fn next(self) -> Self {
        match self {
            ViewTab::Menu => ViewTab::Log,
            ViewTab::Log => ViewTab::Menu,
        }
    }
}

pub(crate) struct Execute {
    pub menu: Menu,
    pub runtime: Runtime,
    pub log_sender: mpsc::UnboundedSender<String>,
    pub log_receiver: mpsc::UnboundedReceiver<String>,
    pub log_lines: VecDeque<String>,
    pub log_scroll: u16,
    pub view_height: usize,
    pub pending_scroll_to_bottom: bool,
    pub view: ViewTab,
    pub tools: Tools,
}

impl Execute {
    pub fn new() -> Self {
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        let mut menu = Menu::from_iter([(
            "Update Dotfiles".to_string(),
            Some(MenuItemAction::UpdateDotfiles),
        )]);
        menu.state.select_first();
        Self {
            menu,
            runtime: Runtime::new().expect("failed to start tokio runtime"),
            log_sender,
            log_receiver,
            log_lines: VecDeque::new(),
            log_scroll: 0,
            view_height: 0,
            pending_scroll_to_bottom: false,
            view: ViewTab::Menu,
            tools: Tools::new().unwrap_or_else(|error| {
                panic!("Failed to build tools: {:?}", error);
            }),
        }
    }
}

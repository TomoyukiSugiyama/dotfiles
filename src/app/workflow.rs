use super::workflow_menu::Menu;
use crate::tools::Tools;

use super::workflow_menu::MenuItemAction;
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

pub(crate) struct Workflow {
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
    pub reload_warning: Option<String>,
}

impl Workflow {
    pub fn new() -> Self {
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        let mut menu = Menu::from_iter([("Run Tools".to_string(), Some(MenuItemAction::RunTools))]);
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
            reload_warning: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test() -> Self {
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        let mut menu = Menu::from_iter([("Run Tools".to_string(), Some(MenuItemAction::RunTools))]);
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
            tools: Tools::new_empty(),
            reload_warning: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_tab_next() {
        assert_eq!(ViewTab::Menu.next(), ViewTab::Log);
        assert_eq!(ViewTab::Log.next(), ViewTab::Menu);
    }

    #[test]
    fn test_workflow_new() {
        let workflow = Workflow::new_for_test();
        assert_eq!(workflow.view, ViewTab::Menu);
        assert_eq!(workflow.log_scroll, 0);
        assert_eq!(workflow.view_height, 0);
        assert!(!workflow.pending_scroll_to_bottom);
        assert!(workflow.reload_warning.is_none());
        assert!(workflow.menu.state.selected().is_some());
    }
}

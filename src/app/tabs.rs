use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, EnumIter)]
pub(crate) enum SelectedTab {
    #[strum(to_string = "Dotfiles")]
    Dotfiles,
    #[default]
    #[strum(to_string = "Workflow")]
    Workflow,
}

impl SelectedTab {
    pub fn new() -> Self {
        Self::Workflow
    }

    fn next(self) -> Self {
        match self {
            SelectedTab::Dotfiles => SelectedTab::Workflow,
            SelectedTab::Workflow => SelectedTab::Dotfiles,
        }
    }
    fn previous(self) -> Self {
        match self {
            SelectedTab::Dotfiles => SelectedTab::Workflow,
            SelectedTab::Workflow => SelectedTab::Dotfiles,
        }
    }

    pub fn select_next_tab(&mut self) {
        *self = self.next();
    }

    pub fn select_previous_tab(&mut self) {
        *self = self.previous();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selected_tab_new() {
        let tab = SelectedTab::new();
        assert_eq!(tab, SelectedTab::Workflow);
    }

    #[test]
    fn test_selected_tab_navigation() {
        let mut tab = SelectedTab::Workflow;
        
        tab.select_next_tab();
        assert_eq!(tab, SelectedTab::Dotfiles);
        
        tab.select_next_tab();
        assert_eq!(tab, SelectedTab::Workflow);
        
        tab.select_previous_tab();
        assert_eq!(tab, SelectedTab::Dotfiles);
        
        tab.select_previous_tab();
        assert_eq!(tab, SelectedTab::Workflow);
    }
}

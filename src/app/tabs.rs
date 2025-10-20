use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, EnumIter)]
pub(crate) enum SelectedTab {
    #[default]
    #[strum(to_string = "Dotfiles")]
    Dotfiles,
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

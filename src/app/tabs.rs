use strum::{Display, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, EnumIter)]
pub(crate) enum SelectedTab {
    #[default]
    #[strum(to_string = "Dotfiles")]
    Dotfiles,
    #[strum(to_string = "Execute")]
    Execute,
}

impl SelectedTab {
    pub fn new() -> Self {
        Self::Execute
    }

    fn next(self) -> Self {
        match self {
            SelectedTab::Dotfiles => SelectedTab::Execute,
            SelectedTab::Execute => SelectedTab::Dotfiles,
        }
    }
    fn previous(self) -> Self {
        match self {
            SelectedTab::Dotfiles => SelectedTab::Execute,
            SelectedTab::Execute => SelectedTab::Dotfiles,
        }
    }

    pub fn select_next_tab(&mut self) {
        *self = self.next();
    }

    pub fn select_previous_tab(&mut self) {
        *self = self.previous();
    }
}

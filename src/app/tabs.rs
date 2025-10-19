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
}

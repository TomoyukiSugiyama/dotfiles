use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub(crate) struct Menu {
    pub(crate) state: ListState,
    pub(crate) items: Vec<MenuItem>,
}

#[derive(Debug, Default)]
pub(crate) struct MenuItem {
    pub(crate) title: String,
    pub(crate) action: Option<MenuItemAction>,
}

#[derive(Debug)]
pub(crate) enum MenuItemAction {
    UpdateDotfiles,
}

impl FromIterator<(String, Option<MenuItemAction>)> for Menu {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, Option<MenuItemAction>)>,
    {
        let items = iter
            .into_iter()
            .map(|(title, action)| MenuItem {
                title: title,
                action: action,
            })
            .collect();
        Self {
            items: items,
            state: ListState::default(),
        }
    }
}

impl Menu {
    pub(crate) fn select_first(&mut self) {
        self.state.select_first();
    }

    pub(crate) fn select_last(&mut self) {
        self.state.select_last();
    }

    pub(crate) fn select_previous(&mut self) {
        self.state.select_previous();
    }

    pub(crate) fn select_next(&mut self) {
        self.state.select_next();
    }

}

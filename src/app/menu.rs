use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub(crate) struct Menu {
    pub state: ListState,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Default)]
pub(crate) struct MenuItem {
    pub title: String,
    pub action: Option<MenuItemAction>,
}

#[derive(Debug)]
pub(crate) enum MenuItemAction {
    UpdateDotfiles,
    Quit,
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
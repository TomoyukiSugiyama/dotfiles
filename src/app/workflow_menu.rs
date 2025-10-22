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
    RunTools,
}

impl FromIterator<(String, Option<MenuItemAction>)> for Menu {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, Option<MenuItemAction>)>,
    {
        let items = iter
            .into_iter()
            .map(|(title, action)| MenuItem { title, action })
            .collect();
        Self {
            items,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_menu() -> Menu {
        Menu::from_iter([
            ("Item 1".to_string(), None),
            ("Item 2".to_string(), None),
            ("Item 3".to_string(), None),
        ])
    }

    #[test]
    fn test_menu_selection() {
        let mut menu = create_test_menu();

        // Test select_first
        menu.select_first();
        assert_eq!(menu.state.selected(), Some(0));

        // Test select_next
        menu.select_next();
        assert_eq!(menu.state.selected(), Some(1));
        menu.select_next();
        assert_eq!(menu.state.selected(), Some(2));

        // Test select_previous
        menu.select_previous();
        assert_eq!(menu.state.selected(), Some(1));
        menu.select_previous();
        assert_eq!(menu.state.selected(), Some(0));
    }
}

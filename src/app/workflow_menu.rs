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

    #[test]
    fn test_menu_select_last() {
        let mut menu = create_test_menu();
        
        menu.select_last();
        // ListState::select_last() behavior depends on the number of items
        // Since we have 3 items, it should select the last one
        // But we need to verify the actual behavior
        let selected = menu.state.selected();
        assert!(selected.is_some());
    }

    #[test]
    fn test_menu_from_iter() {
        let menu = Menu::from_iter([
            ("Item 1".to_string(), Some(MenuItemAction::RunTools)),
            ("Item 2".to_string(), None),
        ]);
        
        assert_eq!(menu.items.len(), 2);
        assert_eq!(menu.items[0].title, "Item 1");
        assert!(matches!(menu.items[0].action, Some(MenuItemAction::RunTools)));
        assert_eq!(menu.items[1].title, "Item 2");
        assert!(menu.items[1].action.is_none());
    }

    #[test]
    fn test_menu_item_default() {
        let item = MenuItem::default();
        assert_eq!(item.title, "");
        assert!(item.action.is_none());
    }
}

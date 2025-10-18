pub(crate) struct Tools {
    pub root: String,
    pub items: Vec<ToolItem>,
}

pub(crate) struct ToolItem {
    pub name: String,
    pub root: String,
    pub file: String,
}

impl Tools {
    pub(crate) fn new() -> Self {
        Self {
            root: "~/.dotfiles".to_string(),
            items: vec![ToolItem {
                name: "Brew".to_string(),
                root: "brew".to_string(),
                file: "brew-settings.zsh".to_string(),
            }],
        }
    }

    pub(crate) fn file_path(&self, tool: &ToolItem) -> String {
        format!("{}/{}/{}", self.root, tool.root, tool.file)
    }
}

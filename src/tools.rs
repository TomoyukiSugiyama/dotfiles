use std::fs;
use std::path::PathBuf;
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
            items: vec![
                ToolItem {
                    name: "Brew".to_string(),
                    root: "brew".to_string(),
                    file: "brew-settings.zsh".to_string(),
                },
                ToolItem {
                    name: "Gcloud".to_string(),
                    root: "gcloud".to_string(),
                    file: "gcloud-settings.zsh".to_string(),
                },
            ],
        }
    }

    pub(crate) fn file_path(&self, tool: &ToolItem) -> String {
        self.tool_path(tool).to_string_lossy().into_owned()
    }

    pub(crate) fn raw_script(&self, tool: &ToolItem) -> Option<String> {
        fs::read_to_string(self.tool_path(tool)).ok()
    }

    fn tool_path(&self, tool: &ToolItem) -> PathBuf {
        let mut root = self.expand_home_path(&self.root);
        root.push(&tool.root);
        root.push(&tool.file);
        root
    }

    fn expand_home_path(&self, path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(stripped);
            }
        }
        PathBuf::from(path)
    }
}

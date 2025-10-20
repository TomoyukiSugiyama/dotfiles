use crate::config::Config;
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
        let config = Config::new().expect("Failed to load config");
        Self {
            root: config.root().to_string(),
            items: config
                .tools()
                .iter()
                .map(|tool| ToolItem {
                    name: tool.name(),
                    root: tool.root_name(),
                    file: tool.file_name(),
                })
                .collect(),
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
        if let Some(stripped) = path.strip_prefix("~/")
            && let Ok(home) = std::env::var("HOME")
        {
            PathBuf::from(home).join(stripped)
        } else {
            PathBuf::from(path)
        }
    }
}

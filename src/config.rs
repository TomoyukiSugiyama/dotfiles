use color_eyre::Result;
use serde::Deserialize;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(rename = "SystemPreferences")]
    system_preferences: SystemPreferences,
    #[serde(rename = "Preferences")]
    preferences: Preferences,
}

#[derive(Debug, Deserialize)]
struct SystemPreferences {
    #[serde(rename = "Root")]
    root: String,
}

#[derive(Debug, Deserialize)]
struct Preferences {
    #[serde(rename = "ToolsSettings")]
    tools_settings: Vec<Tool>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Tool {
    #[serde(rename = "Id", default)]
    pub id: Option<String>,
    #[serde(rename = "Name", default)]
    pub name: Option<String>,
    #[serde(rename = "Root", default)]
    pub root: Option<String>,
    #[serde(rename = "File", default)]
    pub file: Option<String>,
    #[serde(rename = "Dependencies", default)]
    pub dependencies: Vec<String>,
}

pub(crate) const DEFAULT_CONFIG_PATH: &str = "~/.dotfiles/config.yaml";

impl Config {
    pub(crate) fn new() -> Result<Self> {
        Self::create_config_dir()?;
        let config = Self::load()?;
        config.create_tools_dir()?;
        Ok(config)
    }

    pub(crate) fn load() -> Result<Self> {
        Self::load_from_file(DEFAULT_CONFIG_PATH)
    }

    pub(crate) fn load_from_file(path: &str) -> Result<Self> {
        let path = expand_home_path(path);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        Ok(serde_yaml::from_reader(reader)?)
    }

    pub(crate) fn root(&self) -> &str {
        &self.system_preferences.root
    }

    pub(crate) fn tools(&self) -> &[Tool] {
        &self.preferences.tools_settings
    }
    fn create_config_dir() -> Result<()> {
        let home = env::var("HOME").expect("HOME environment variable is not set");
        let config_dir = PathBuf::from(home).join(".dotfiles");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).expect("Failed to create config directory");
        }
        let config_file = config_dir.join("config.yaml");
        if !config_file.exists() {
            const DEFAULT_CONFIG: &str = concat!(
                "# Dotfiles Manager configuration\n",
                "# SystemPreferences.Root: absolute path that stores all managed tool directories\n",
                "SystemPreferences:\n",
                "  Root: ~/.dotfiles\n",
                "# Preferences.ToolsSettings: list of tools to manage\n",
                "#   Id: Optional unique identifier used to reference dependencies\n",
                "#       (if omitted, an identifier is generated automatically)\n",
                "#       Auto-generated Ids are derived from Name (lowercase, hyphenated, with numeric suffixes on conflict).\n",
                "#       The exact format is equivalent to: `name.to_lowercase().replace(' ', '-')` with `-N` suffixes for duplicates.\n",
                "#   Name: Optional display label (defaults to Root or 'unknown')\n",
                "#   Root: Optional directory segment; defaults to lowercase Name\n",
                "#   File: Optional script filename; defaults to '<name>-settings.zsh'\n",
                "#   Dependencies: List other tool Ids this tool requires (never include its own Id)\n",
                "Preferences:\n",
                "  ToolsSettings:\n",
                "    # - Name: Brew            # Label shown in the UI\n",
                "    #   Root: brew            # Directory (under SystemPreferences.Root) that contains the tool's scripts\n",
                "    #   File: brew-settings.zsh # Script executed when running the tool\n",
                "    # - Name: Gcloud\n",
                "    #   Dependencies:\n",
                "    #     - brew              # Reference another tool Id defined above (e.g., Brew)\n",
            );
            fs::write(&config_file, DEFAULT_CONFIG)?;
        }
        Ok(())
    }
    fn create_tools_dir(&self) -> Result<()> {
        let root = expand_home_path(&self.system_preferences.root);

        for tool in self.tools() {
            let tool_dir = root.join(tool.root_name());
            if !tool_dir.exists() {
                fs::create_dir_all(&tool_dir)?;
            }
            let tool_file = tool_dir.join(tool.file_name());
            if !tool_file.exists() {
                let content = format!(
                    "# Settings for tool: {name}\n# Located under: {root}\n# Populate this file with the commands or scripts needed to configure {name}.\n",
                    name = tool.name(),
                    root = tool.root_name()
                );
                #[cfg(unix)]
                {
                    OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .mode(0o700)
                        .open(&tool_file)?
                        .write_all(content.as_bytes())?;
                }
                #[cfg(not(unix))]
                {
                    fs::write(&tool_file, content)?;
                }
            }
        }

        Ok(())
    }
}

impl Tool {
    pub fn identifier(&self) -> Option<String> {
        self.id
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
    }
    pub fn name(&self) -> String {
        self.name.clone().unwrap_or_else(|| "unknown".to_string())
    }
    pub fn root_name(&self) -> String {
        self.root
            .clone()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.name().to_lowercase())
    }
    pub fn file_name(&self) -> String {
        self.file
            .clone()
            .unwrap_or_else(|| format!("{}-settings.zsh", self.name().to_lowercase()))
    }
    pub fn dependencies(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .map(|dependency| dependency.trim())
            .filter(|dependency| !dependency.is_empty())
            .map(|dependency| dependency.to_string())
            .collect()
    }
}
pub(crate) fn expand_home_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Ok(home) = env::var("HOME")
    {
        PathBuf::from(home).join(stripped)
    } else {
        PathBuf::from(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_from_file() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("config.yaml");
        fs::write(
            &config_file,
            r#"
SystemPreferences:
  Root: /test/root
Preferences:
  ToolsSettings:
    - Name: TestTool
      Root: testtool
      File: test.zsh
"#,
        )
        .unwrap();

        let config = Config::load_from_file(config_file.to_str().unwrap()).unwrap();
        assert_eq!(config.root(), "/test/root");
        assert_eq!(config.tools().len(), 1);
        assert_eq!(config.tools()[0].name(), "TestTool");
    }

    #[test]
    fn test_tool_defaults() {
        let tool = Tool {
            id: None,
            name: Some("MyTool".to_string()),
            root: None,
            file: None,
            dependencies: vec![],
        };

        assert_eq!(tool.name(), "MyTool");
        assert_eq!(tool.root_name(), "mytool");
        assert_eq!(tool.file_name(), "mytool-settings.zsh");
        assert!(tool.identifier().is_none());
    }

    #[test]
    fn test_expand_home_path() {
        let original_home = std::env::var("HOME");
        unsafe {
            std::env::set_var("HOME", "/test/home");
        }

        let expanded = expand_home_path("~/test/path");
        assert_eq!(expanded, PathBuf::from("/test/home/test/path"));

        let not_expanded = expand_home_path("/absolute/path");
        assert_eq!(not_expanded, PathBuf::from("/absolute/path"));

        unsafe {
            if let Ok(home) = original_home {
                std::env::set_var("HOME", home);
            }
        }
    }
}

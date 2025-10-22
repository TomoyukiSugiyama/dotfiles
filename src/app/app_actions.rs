use super::App;
use crate::tools::Tools;

impl App {
    pub(crate) fn reload_config(&mut self) -> Result<Option<String>, String> {
        match Tools::new_relaxed() {
            Ok((tools, warnings)) => {
                self.dotfiles.apply_tools(tools.clone());
                self.workflow.apply_tools(tools);
                self.workflow.clear_reload_warning();

                if warnings.is_empty() {
                    self.dotfiles.clear_reload_warning();
                    Ok(None)
                } else {
                    let message = warnings.join("\n");
                    self.dotfiles.show_reload_warning(message.clone());
                    self.workflow.show_reload_warning(message.clone());
                    Ok(Some(message))
                }
            }
            Err(error) => {
                let message = error.to_string();
                self.dotfiles.show_reload_error(message.clone());
                self.workflow.show_reload_error(message.clone());
                Err(message)
            }
        }
    }

    /// Set running to false to quit the application.
    pub(crate) fn quit(&mut self) {
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_reload_config_success() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join(".dotfiles");
        fs::create_dir(&config_path).unwrap();
        let config_file = config_path.join("config.yaml");
        fs::write(
            &config_file,
            r#"
SystemPreferences:
  Root: ~/.dotfiles
Preferences:
  ToolsSettings: []
"#,
        )
        .unwrap();

        let original_home = std::env::var("HOME");
        unsafe {
            std::env::set_var("HOME", dir.path().to_str().unwrap());
        }

        let mut app = App::new();
        let result = app.reload_config();

        unsafe {
            if let Ok(home) = original_home {
                std::env::set_var("HOME", home);
            }
        }

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert!(app.dotfiles.reload_error.is_none());
        assert!(app.dotfiles.reload_warning.is_none());
    }

    #[test]
    fn test_quit() {
        let mut app = App::new();
        assert!(app.running);
        app.quit();
        assert!(!app.running);
    }
}

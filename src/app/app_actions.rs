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

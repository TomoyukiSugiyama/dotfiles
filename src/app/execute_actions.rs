use super::execute::Execute;
use super::execute_log::forward_stream;
use super::execute_menu::MenuItemAction;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

impl Execute {
    pub(crate) fn execute_selected(&mut self) {
        if let Some(selected_index) = self.menu.state.selected() {
            let item = &self.menu.items[selected_index];
            match item.action {
                Some(MenuItemAction::UpdateDotfiles) => self.update_dotfiles(),
                None => {}
            };
        }
    }
    pub(crate) fn scroll_log(&mut self, amount: i16) {
        if self.log_lines.is_empty() {
            return;
        }
        if self.log_scroll == self.log_lines.len().saturating_sub(self.view_height) as u16
            && amount > 0
        {
            return;
        }
        if self.log_scroll == 0 && amount < 0 {
            return;
        }

        self.log_scroll = if amount < 0 {
            self.log_scroll.saturating_sub(amount.unsigned_abs())
        } else {
            self.log_scroll.saturating_add(amount.unsigned_abs())
        };

        let max_scroll = self.log_lines.len().saturating_sub(self.view_height) as u16;
        self.log_scroll = self.log_scroll.min(max_scroll);
    }

    pub(crate) fn scroll_log_to_bottom(&mut self) {
        self.log_scroll = self.log_lines.len().saturating_sub(self.view_height) as u16;
    }
    pub(crate) fn scroll_log_to_top(&mut self) {
        self.log_scroll = 0;
    }

    fn update_dotfiles(&self) {
        self.log_message("Updating dotfiles...\n");

        for tool in &self.tools.items {
            let file = self.tools.file_path(tool);
            self.log_tool_start(&tool.name, &file);
            self.spawn_tool_update_task(&tool.name, file);
        }
        self.log_message("All tools updated successfully.\n");
    }

    fn log_message<S: Into<String>>(&self, message: S) {
        let _ = self.log_sender.send(message.into());
    }

    fn log_tool_start(&self, tool_name: &str, file: &str) {
        self.log_message(format!("{tool_name} | Updating...\n"));
        self.log_message(format!("{tool_name} | Running {file}\n"));
    }

    fn spawn_tool_update_task(&self, tool_name: &str, file: String) {
        let sender = self.log_sender.clone();
        let tool_name = tool_name.to_string();
        self.runtime.spawn(async move {
            Self::run_tool_script(tool_name, file, sender).await;
        });
    }

    async fn run_tool_script(
        tool_name: String,
        file: String,
        sender: mpsc::UnboundedSender<String>,
    ) {
        let command = TokioCommand::new("zsh")
            .arg("-c")
            .arg(file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match command {
            Ok(child) => child,
            Err(e) => {
                let hint = match e.kind() {
                    std::io::ErrorKind::NotFound => "Script not found or zsh missing?",
                    std::io::ErrorKind::PermissionDenied => "Try chmod +x or run with sudo",
                    std::io::ErrorKind::Other => "unknown error",
                    _ => "unknown error",
                };
                let _ = sender.send(format!("Failed to spawn command: {e}\n{hint}"));
                return;
            }
        };

        let stdout_prefix = format!("{tool_name} | ");
        let stderr_prefix = format!("{tool_name} | error: ");
        let stdout_task = child.stdout.take().map(|stdout| {
            let sender = sender.clone();
            let prefix = stdout_prefix.clone();
            tokio::spawn(async move {
                forward_stream(stdout, sender, "stdout", prefix).await;
            })
        });

        let stderr_task = child.stderr.take().map(|stderr| {
            let sender = sender.clone();
            let prefix = stderr_prefix.clone();
            tokio::spawn(async move {
                forward_stream(stderr, sender, "stderr", prefix).await;
            })
        });

        let status = child.wait().await;

        if let Some(task) = stdout_task {
            let _ = task.await;
        }
        if let Some(task) = stderr_task {
            let _ = task.await;
        }

        match status {
            Ok(status) => {
                let _ = sender.send(format!(
                    "{tool_name} | Command exited with status: {status}\n"
                ));
            }
            Err(e) => {
                let _ = sender.send(format!("{tool_name} | Command failed with error: {e}\n"));
            }
        }
    }
}

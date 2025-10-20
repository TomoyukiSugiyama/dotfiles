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

        let jobs = self
            .tools
            .iter()
            .map(|tool| (tool.name.clone(), self.tools.file_path(tool)))
            .collect::<Vec<(String, String)>>();

        let sender = self.log_sender.clone();

        self.runtime.spawn(async move {
            for (tool_name, file) in jobs {
                let _ = sender.send(format!("{tool_name} | Updating...\n"));
                let _ = sender.send(format!("{tool_name} | Running {file}\n"));

                Execute::run_tool_script(tool_name.clone(), file, sender.clone()).await;
            }

            let _ = sender.send("All tools updated successfully.\n".to_string());
        });
    }

    fn log_message<S: Into<String>>(&self, message: S) {
        let _ = self.log_sender.send(message.into());
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

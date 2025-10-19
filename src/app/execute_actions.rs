use super::execute::Execute;
use super::execute_log::forward_stream;
use super::execute_menu::MenuItemAction;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

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

    fn update_dotfiles(&mut self) {
        let _ = self.log_sender.send("Updating dotfiles...\n".to_string());

        for tool in self.tools.items.iter() {
            let sender = self.log_sender.clone();
            let file = self.tools.file_path(tool);
            let tool_name = tool.name.clone();
            let _ = sender.send(format!("{} | Updating...\n", tool_name));
            let _ = sender.send(format!("{} | Running {}\n", tool_name, file));
            let task_sender = sender.clone();
            self.runtime.spawn({
                let file = file.clone();
                let tool_name = tool_name.clone();
                async move {
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
                                std::io::ErrorKind::PermissionDenied => {
                                    "Try chmod +x or run with sudo"
                                }
                                std::io::ErrorKind::Other => "unknown error",
                                _ => "unknown error",
                            };
                            let _ =
                                task_sender.send(format!("Failed to spawn command: {e}\n{hint}"));
                            return;
                        }
                    };
                    let stdout_prefix = format!("{} | ", tool_name);
                    let stderr_prefix = format!("{} | error: ", tool_name);
                    let stdout_task = child.stdout.take().map(|stdout| {
                        let sender = task_sender.clone();
                        let prefix = stdout_prefix.clone();
                        tokio::spawn(async move {
                            forward_stream(stdout, sender, "stdout", prefix).await;
                        })
                    });

                    let stderr_task = child.stderr.take().map(|stderr| {
                        let sender = task_sender.clone();
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
                            let _ = task_sender.send(format!(
                                "{} | Command exited with status: {}\n",
                                tool_name, status
                            ));
                        }
                        Err(e) => {
                            let _ = task_sender.send(format!(
                                "{} | Command failed with error: {}\n",
                                tool_name, e
                            ));
                        }
                    }
                }
            });
        }
    }
}

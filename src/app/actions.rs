use super::App;
use super::menu::MenuItemAction;
use crate::logging::forward_stream;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;

impl App {
    pub(crate) fn execute_selected(&mut self) {
        if let Some(selected_index) = self.menu.state.selected() {
            let item = &self.menu.items[selected_index];
            match item.action {
                Some(MenuItemAction::UpdateDotfiles) => self.update_dotfiles(),
                Some(MenuItemAction::Quit) => self.quit(),
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
            self.log_scroll.saturating_sub(amount.abs() as u16)
        } else {
            self.log_scroll.saturating_add(amount.abs() as u16)
        };
    }

    pub(crate) fn scroll_log_to_bottom(&mut self) {
        self.log_scroll = self.log_lines.len().saturating_sub(self.view_height) as u16;
    }
    pub(crate) fn scroll_log_to_top(&mut self) {
        self.log_scroll = 0;
    }

    /// Set running to false to quit the application.
    pub(crate) fn quit(&mut self) {
        self.running = false;
    }

    fn update_dotfiles(&mut self) {
        let _ = self.log_sender.send("Updating dotfiles...\n".to_string());

        for tool in self.tools.items.iter() {
            let sender = self.log_sender.clone();
            let file = self.tools.file_path(tool);
            let _ = sender.send(format!("Updating {}\n", tool.name));
            let _ = sender.send(format!("Running {}\n", file));
            self.runtime.spawn(async move {
                let mut child = TokioCommand::new("zsh")
                    .arg("-c")
                    .arg(file)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("failed to spawn command");

                let stdout_task = child.stdout.take().map(|stdout| {
                    let sender = sender.clone();
                    tokio::spawn(async move {
                        forward_stream(stdout, sender, "stdout", "").await;
                    })
                });

                let stderr_task = child.stderr.take().map(|stderr| {
                    let sender = sender.clone();
                    tokio::spawn(async move {
                        forward_stream(stderr, sender, "stderr", "stderr: ").await;
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
                        let _ = sender.send(format!("Command exited with status: {}\n", status));
                    }
                    Err(e) => {
                        let _ = sender.send(format!("Command failed with error: {}\n", e));
                    }
                }
            });
        }
    }
}

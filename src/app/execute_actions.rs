use super::execute::{Execute, ViewTab};
use super::execute_log::forward_stream;
use super::execute_menu::MenuItemAction;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;

#[derive(Debug)]
struct ToolRunResult {
    name: String,
    status: ToolRunStatus,
}

#[derive(Debug)]
enum ToolRunStatus {
    Success,
    Failed { reason: String },
}

impl ToolRunResult {
    fn success(name: String) -> Self {
        Self {
            name,
            status: ToolRunStatus::Success,
        }
    }

    fn failed(name: String, reason: String) -> Self {
        Self {
            name,
            status: ToolRunStatus::Failed { reason },
        }
    }

    fn is_success(&self) -> bool {
        matches!(self.status, ToolRunStatus::Success)
    }

    fn failure_reason(&self) -> Option<&str> {
        match &self.status {
            ToolRunStatus::Failed { reason } => Some(reason.as_str()),
            ToolRunStatus::Success => None,
        }
    }
}

impl Execute {
    pub(crate) fn execute_selected(&mut self) {
        if let Some(selected_index) = self.menu.state.selected() {
            let item = &self.menu.items[selected_index];
            match item.action {
                Some(MenuItemAction::UpdateDotfiles) => {
                    self.view = ViewTab::Log;
                    self.pending_scroll_to_bottom = true;
                    self.update_dotfiles();
                }
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

        let tool_groups = self
            .tools
            .execution_stages()
            .into_iter()
            .map(|stage| {
                stage
                    .into_iter()
                    .map(|tool| (tool.name.clone(), self.tools.file_path(&tool)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let sender = self.log_sender.clone();
        self.runtime.spawn(async move {
            let mut all_results = Vec::new();
            for stage in tool_groups {
                let mut handles = Vec::new();
                for (tool_name, file) in stage {
                    let sender = sender.clone();
                    handles.push(tokio::spawn(async move {
                        let _ = sender.send(format!("{tool_name} | Updating...\n"));
                        let _ = sender.send(format!("{tool_name} | Running {file}\n"));
                        Execute::run_tool_script(tool_name, file, sender).await
                    }));
                }

                for handle in handles {
                    match handle.await {
                        Ok(result) => all_results.push(result),
                        Err(join_error) => {
                            let reason = format!("background task join error: {}", join_error);
                            let _ =
                                sender.send(format!("Worker join failure detected: {reason}\n"));
                            all_results
                                .push(ToolRunResult::failed("<unknown>".to_string(), reason));
                        }
                    }
                }
            }

            if all_results.is_empty() {
                let _ = sender.send("No tools were scheduled for update.\n".to_string());
                return;
            }

            let successes = all_results
                .iter()
                .filter(|result| result.is_success())
                .map(|result| result.name.as_str())
                .collect::<Vec<_>>();
            let failures = all_results
                .iter()
                .filter(|result| !result.is_success())
                .collect::<Vec<_>>();
            let has_failures = !failures.is_empty();

            let _ = sender.send("\n----- Update Summary -----\n".to_string());
            let _ = sender.send(format!(
                "Status: {}\n",
                if has_failures { "FAILED" } else { "SUCCESS" }
            ));
            let _ = sender.send(format!(
                "Succeeded: {}/{}\n",
                successes.len(),
                all_results.len()
            ));

            if has_failures {
                let _ = sender.send("Failed tools:\n".to_string());
                for failure in &failures {
                    let failure = *failure;
                    let reason = failure
                        .failure_reason()
                        .map(|text| text.trim())
                        .filter(|text| !text.is_empty())
                        .unwrap_or("no additional details");
                    let _ = sender.send(format!("  - {} ({})\n", failure.name, reason));
                }
            } else {
                let _ = sender.send("Failed: none\n".to_string());
            }

            if !successes.is_empty() {
                let _ = sender.send("Successful tools:\n".to_string());
                for name in successes {
                    let _ = sender.send(format!("  - {}\n", name));
                }
            }

            let final_status = if has_failures {
                "Update finished with errors. See summary above.\n"
            } else {
                "Update completed successfully.\n"
            };
            let _ = sender.send(final_status.to_string());
        });
    }

    fn log_message<S: Into<String>>(&self, message: S) {
        let _ = self.log_sender.send(message.into());
    }

    async fn run_tool_script(
        tool_name: String,
        file: String,
        sender: mpsc::UnboundedSender<String>,
    ) -> ToolRunResult {
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
                let message = format!("Failed to spawn command: {e}\n{hint}\n");
                let _ = sender.send(message.clone());
                return ToolRunResult::failed(tool_name, format!("failed to spawn command: {e}"));
            }
        };

        let stdout_prefix = format!("{tool_name} | ");
        let stderr_prefix = format!("{tool_name} | ");
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
                if status.success() {
                    ToolRunResult::success(tool_name)
                } else {
                    ToolRunResult::failed(tool_name, format!("command exited with status {status}"))
                }
            }
            Err(e) => {
                let _ = sender.send(format!("{tool_name} | Command failed with error: {e}\n"));
                ToolRunResult::failed(tool_name, format!("command failed with error: {e}"))
            }
        }
    }
}

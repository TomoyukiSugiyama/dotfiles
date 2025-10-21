use super::workflow::{ViewTab, Workflow};
use super::workflow_log::forward_stream;
use super::workflow_menu::MenuItemAction;
use crate::tools::Tools;
use std::process::Stdio;
use tokio::io::AsyncRead;
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

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

#[derive(Debug, Clone)]
struct PreparedTool {
    name: String,
    script_path: String,
}

impl Workflow {
    pub(crate) fn apply_tools(&mut self, tools: Tools) {
        self.tools = tools;
        self.reload_warning = None;
    }

    pub(crate) fn show_reload_warning(&mut self, message: String) {
        self.reload_warning = Some(message);
    }

    pub(crate) fn clear_reload_warning(&mut self) {
        self.reload_warning = None;
    }

    pub(crate) fn show_reload_error(&mut self, message: String) {
        self.log_lines.push_back(format!("{message}\n"));
        self.pending_scroll_to_bottom = true;
    }

    pub(crate) fn execute_selected(&mut self) {
        if let Some(selected_index) = self.menu.state.selected() {
            let item = &self.menu.items[selected_index];
            match item.action {
                Some(MenuItemAction::RunTools) => {
                    self.view = ViewTab::Log;
                    self.pending_scroll_to_bottom = true;
                    self.run_tools();
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

    fn run_tools(&self) {
        self.log_message("Running tools...\n");

        let tool_groups = self.prepare_tool_groups();
        let sender = self.log_sender.clone();

        self.runtime
            .spawn(async move { Workflow::execute_tool_groups(tool_groups, sender).await });
    }

    fn log_message<S: Into<String>>(&self, message: S) {
        let _ = self.log_sender.send(message.into());
    }

    async fn run_tool_script(
        tool_name: String,
        file: String,
        sender: mpsc::UnboundedSender<String>,
    ) -> ToolRunResult {
        let mut child = match Self::spawn_tool_child(&file) {
            Ok(child) => child,
            Err(error) => return Self::handle_command_spawn_error(tool_name, error, &sender),
        };

        let stdout_task = Self::spawn_output_forwarder(
            child.stdout.take(),
            &sender,
            "stdout",
            format!("{tool_name} | "),
        );
        let stderr_task = Self::spawn_output_forwarder(
            child.stderr.take(),
            &sender,
            "stderr",
            format!("{tool_name} | "),
        );

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
            Err(error) => {
                let _ = sender.send(format!(
                    "{tool_name} | Command failed with error: {error}\n"
                ));
                ToolRunResult::failed(tool_name, format!("command failed with error: {error}"))
            }
        }
    }
}

impl Workflow {
    fn prepare_tool_groups(&self) -> Vec<Vec<PreparedTool>> {
        self.tools
            .execution_stages()
            .into_iter()
            .map(|stage| {
                stage
                    .into_iter()
                    .map(|tool| PreparedTool {
                        name: tool.name.clone(),
                        script_path: self.tools.file_path(&tool),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    async fn execute_tool_groups(
        tool_groups: Vec<Vec<PreparedTool>>,
        sender: mpsc::UnboundedSender<String>,
    ) {
        let mut all_results = Vec::new();

        for stage in tool_groups {
            if stage.is_empty() {
                continue;
            }

            let mut handles = Vec::with_capacity(stage.len());
            for tool in stage {
                let sender = sender.clone();
                handles.push(tokio::spawn(async move {
                    Workflow::run_prepared_tool(tool, sender).await
                }));
            }

            for handle in handles {
                match handle.await {
                    Ok(result) => all_results.push(result),
                    Err(join_error) => {
                        let reason = format!("background task join error: {}", join_error);
                        let _ = sender.send(format!("Worker join failure detected: {reason}\n"));
                        all_results.push(ToolRunResult::failed("<unknown>".to_string(), reason));
                    }
                }
            }
        }

        if all_results.is_empty() {
            let _ = sender.send("No tools were scheduled to run.\n".to_string());
            return;
        }

        Self::report_tool_run_summary(&all_results, &sender);
    }

    async fn run_prepared_tool(
        tool: PreparedTool,
        sender: mpsc::UnboundedSender<String>,
    ) -> ToolRunResult {
        let PreparedTool { name, script_path } = tool;
        let _ = sender.send(format!("{name} | Starting...\n"));
        let _ = sender.send(format!("{name} | Running {script_path}\n"));
        Self::run_tool_script(name, script_path, sender).await
    }

    fn report_tool_run_summary(results: &[ToolRunResult], sender: &mpsc::UnboundedSender<String>) {
        let successes = results
            .iter()
            .filter(|result| result.is_success())
            .map(|result| result.name.as_str())
            .collect::<Vec<_>>();
        let failures = results
            .iter()
            .filter(|result| !result.is_success())
            .collect::<Vec<_>>();
        let has_failures = !failures.is_empty();

        let _ = sender.send("\n----- Tool Run Summary -----\n".to_string());
        let _ = sender.send(format!(
            "Status: {}\n",
            if has_failures { "FAILED" } else { "SUCCESS" }
        ));
        let _ = sender.send(format!(
            "Succeeded: {}/{}\n",
            successes.len(),
            results.len()
        ));

        if has_failures {
            let _ = sender.send("Failed tools:\n".to_string());
            for failure in failures {
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
            "Tool run finished with errors. See summary above.\n"
        } else {
            "Tool run completed successfully.\n"
        };
        let _ = sender.send(final_status.to_string());
    }

    fn spawn_output_forwarder<R>(
        stream: Option<R>,
        sender: &mpsc::UnboundedSender<String>,
        stream_label: &'static str,
        prefix: String,
    ) -> Option<JoinHandle<()>>
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        stream.map(|stream| {
            let sender = sender.clone();
            tokio::spawn(async move {
                forward_stream(stream, sender, stream_label, prefix).await;
            })
        })
    }

    fn handle_command_spawn_error(
        tool_name: String,
        error: std::io::Error,
        sender: &mpsc::UnboundedSender<String>,
    ) -> ToolRunResult {
        let hint = match error.kind() {
            std::io::ErrorKind::NotFound => "Script not found or zsh missing?",
            std::io::ErrorKind::PermissionDenied => "Try chmod +x or run with sudo",
            std::io::ErrorKind::Other => "unknown error",
            _ => "unknown error",
        };
        let message = format!("Failed to spawn command: {error}\n{hint}\n");
        let _ = sender.send(message);
        ToolRunResult::failed(tool_name, format!("failed to spawn command: {error}"))
    }

    fn spawn_tool_child(file: &str) -> std::io::Result<tokio::process::Child> {
        TokioCommand::new("zsh")
            .arg("-c")
            .arg(file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

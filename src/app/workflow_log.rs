use super::workflow::Workflow;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc;

pub(crate) async fn forward_stream<R>(
    reader: R,
    sender: mpsc::UnboundedSender<String>,
    label: &'static str,
    prefix: String,
) where
    R: AsyncRead + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                if !line.is_empty() {
                    let _ = sender.send(format!("{}{}", prefix.as_str(), line));
                }
                break;
            }
            Ok(_) => {
                let _ = sender.send(format!("{}{}", prefix.as_str(), line));
            }
            Err(e) => {
                let _ = sender.send(format!("{} read error: {}\n", label, e));
                break;
            }
        }
    }
}

const MAX_LOG_LINES: usize = 1000;

impl Workflow {
    pub(crate) fn drain_log_messages(&mut self) {
        while let Ok(message) = self.log_receiver.try_recv() {
            if self.log_lines.len() >= MAX_LOG_LINES {
                self.log_lines.pop_front();
            }
            self.log_lines.push_back(message);
            if self.view_height == 0 {
                self.pending_scroll_to_bottom = true;
            } else {
                self.scroll_log_to_bottom();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drain_log_messages() {
        let mut workflow = Workflow::new_for_test();

        // Send some messages
        workflow.log_sender.send("Line 1\n".to_string()).unwrap();
        workflow.log_sender.send("Line 2\n".to_string()).unwrap();
        workflow.log_sender.send("Line 3\n".to_string()).unwrap();

        // Drain messages
        workflow.drain_log_messages();

        assert_eq!(workflow.log_lines.len(), 3);
        assert_eq!(workflow.log_lines[0], "Line 1\n");
        assert_eq!(workflow.log_lines[1], "Line 2\n");
        assert_eq!(workflow.log_lines[2], "Line 3\n");
    }

    #[test]
    fn test_drain_log_messages_max_limit() {
        let mut workflow = Workflow::new_for_test();

        // Send more than MAX_LOG_LINES messages
        for i in 0..(MAX_LOG_LINES + 10) {
            workflow.log_sender.send(format!("Line {}\n", i)).unwrap();
        }

        workflow.drain_log_messages();

        // Should not exceed MAX_LOG_LINES
        assert_eq!(workflow.log_lines.len(), MAX_LOG_LINES);
        // First line should be line 10 (0-9 were removed)
        assert_eq!(workflow.log_lines[0], "Line 10\n");
    }
}

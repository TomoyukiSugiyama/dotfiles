use super::execute::Execute;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc;

pub(crate) async fn forward_stream<R>(
    reader: R,
    sender: mpsc::UnboundedSender<String>,
    label: &'static str,
    prefix: &str,
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
                    let _ = sender.send(format!("{}{}", prefix, line));
                }
                break;
            }
            Ok(_) => {
                let _ = sender.send(format!("{}{}", prefix, line));
            }
            Err(e) => {
                let _ = sender.send(format!("{} read error: {}\n", label, e));
                break;
            }
        }
    }
}

const MAX_LOG_LINES: usize = 1000;

impl Execute {
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

use tokio::sync::mpsc;

use super::app::App;

use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

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

impl App {
    pub(crate) fn drain_log_messages(&mut self) {
        while let Ok(message) = self.log_receiver.try_recv() {
            if self.log_lines.len() >= MAX_LOG_LINES {
                self.log_lines.pop_front();
            }
            self.log_lines.push_back(message);
            self.log_scroll = self.log_lines.len().saturating_sub(self.view_height) as u16;
        }
    }
}

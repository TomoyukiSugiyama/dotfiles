async fn forward_stream<R>(
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
mod tools;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::Rect,
    layout::{Constraint, Layout},
    style::palette::tailwind::SLATE,
    style::{Color, Modifier, Style},
    symbols::{self},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget,
        Widget,
    },
};
use std::{process::Stdio, time::Duration};
use tokio::process::Command as TokioCommand;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    runtime::Runtime,
    sync::mpsc,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
struct App {
    /// Is the application running?
    running: bool,
    menu: Menu,
    log: String,
    runtime: Runtime,
    log_sender: mpsc::UnboundedSender<String>,
    log_receiver: mpsc::UnboundedReceiver<String>,
    tools: tools::Tools,
}

#[derive(Debug, Default)]
struct Menu {
    state: ListState,
    items: Vec<MenuItem>,
}

#[derive(Debug, Default)]
struct MenuItem {
    title: String,
    action: Option<MenuItemAction>,
}

#[derive(Debug)]
enum MenuItemAction {
    UpdateDotfiles,
    Quit,
}

impl FromIterator<(String, Option<MenuItemAction>)> for Menu {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (String, Option<MenuItemAction>)>,
    {
        let items = iter
            .into_iter()
            .map(|(title, action)| MenuItem {
                title: title,
                action: action,
            })
            .collect();
        Self {
            items: items,
            state: ListState::default(),
        }
    }
}

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl Widget for &mut App {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let [header_area, footer_area, menu_area, log_area] = Layout::vertical([
            Constraint::Percentage(5),
            Constraint::Percentage(5),
            Constraint::Percentage(30),
            Constraint::Percentage(60),
        ])
        .areas(area);

        self.render_header(header_area, buffer);
        self.render_footer(footer_area, buffer);
        self.render_log(log_area, buffer);
        self.render_menu(menu_area, buffer);
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let tools = tools::Tools::new();
        let runtime = Runtime::new().expect("failed to start tokio runtime");
        let (log_sender, log_receiver) = mpsc::unbounded_channel();
        Self {
            running: true,
            menu: Menu::from_iter([
                (
                    "Update Dotfiles".to_string(),
                    Some(MenuItemAction::UpdateDotfiles),
                ),
                ("Quit".to_string(), Some(MenuItemAction::Quit)),
            ]),
            log: String::new(),
            runtime,
            log_sender,
            log_receiver,
            tools,
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            self.drain_log_messages();
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    fn render_header(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Dotfiles Manager")
            .centered()
            .render(area, buffer);
    }
    fn render_footer(&mut self, area: Rect, buffer: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, → to select, Home/End to go top/bottom.")
            .centered()
            .render(area, buffer);
    }

    fn render_log(&mut self, area: Rect, buffer: &mut Buffer) {
        let block = Block::new()
            .title(Line::from("Log"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::Black));
        Paragraph::new(self.log.clone())
            .block(block)
            .render(area, buffer);
    }
    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render_menu(&mut self, area: Rect, buffer: &mut Buffer) {
        // let title = Line::from("Dotfiles Manager").bold().blue().centered();
        // let text = "Hello, Dotfiles!\n\n\
        //     Created using https://github.com/tomoyukisugiyama/dotfiles\n\
        //     Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
        //     Press `u` to update dotfiles.\n\n";

        let block = Block::new()
            .title(Line::from("Menu"))
            .borders(Borders::ALL)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(Color::Black));
        let items = self
            .menu
            .items
            .iter()
            .map(|item| ListItem::new(item.title.clone()))
            .collect::<Vec<ListItem>>();
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(list, area, buffer, &mut self.menu.state);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Home) => self.select_first(),
            (_, KeyCode::End) => self.select_last(),
            (_, KeyCode::Up) => self.select_previous(),
            (_, KeyCode::Down) => self.select_next(),
            (_, KeyCode::Enter | KeyCode::Right) => self.execute_selected(),
            (_, KeyCode::Left) => self.unselect(),
            _ => {}
        }
    }

    fn select_first(&mut self) {
        self.menu.state.select_first();
    }

    fn select_last(&mut self) {
        self.menu.state.select_last();
    }

    fn select_previous(&mut self) {
        self.menu.state.select_previous();
    }

    fn select_next(&mut self) {
        self.menu.state.select_next();
    }

    fn unselect(&mut self) {
        self.menu.state.select(None);
    }

    fn execute_selected(&mut self) {
        if let Some(selected_index) = self.menu.state.selected() {
            let item = &self.menu.items[selected_index];
            match item.action {
                Some(MenuItemAction::UpdateDotfiles) => self.update_dotfiles(),
                Some(MenuItemAction::Quit) => self.quit(),
                None => {}
            };
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn update_dotfiles(&mut self) {
        self.log.push_str("Updating dotfiles...\n");

        for tool in self.tools.items.iter() {
            let sender = self.log_sender.clone();
            let file = format!("{}/{}/{}", self.tools.root, tool.root, tool.file);
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

    fn drain_log_messages(&mut self) {
        while let Ok(message) = self.log_receiver.try_recv() {
            self.log.push_str(&message);
        }
    }
}

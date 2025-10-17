use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    layout::Rect,
    widgets::{Block, List, ListItem, ListState, StatefulWidget, HighlightSpacing, Widget, Borders},
    symbols,
    style::{Style, Modifier},
    buffer::Buffer,
    style::{palette::tailwind::{SLATE, BLUE},},
};
use std::process::Command;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
struct App {
    /// Is the application running?
    running: bool,
    menu: Menu,
}

#[derive(Debug, Default)]
struct Menu {
    state: ListState,
    items: Vec<MenuItem>,
}

#[derive(Debug, Default)]
struct MenuItem {
    title: String,
}

impl FromIterator<String> for Menu {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = String> {
        let items = iter.into_iter().map(|title| MenuItem { title: title }).collect();
        Self { items: items, state: ListState::default() }
    }
}

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl Widget for &mut App {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.render(area, buffer);
    }
}
impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            menu: Menu::from_iter([
                "Update Dotfiles".to_string(),
                "Quit".to_string(),
            ]),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, area: Rect, buffer: &mut Buffer) {
        // let title = Line::from("Dotfiles Manager").bold().blue().centered();
        // let text = "Hello, Dotfiles!\n\n\
        //     Created using https://github.com/tomoyukisugiyama/dotfiles\n\
        //     Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
        //     Press `u` to update dotfiles.\n\n";

        let block = Block::new()
        .title("Dotfiles Manager")
        .borders(Borders::TOP)
        .border_set(symbols::border::EMPTY)
        .border_style(Style::new().fg(SLATE.c100).bg(BLUE.c800));
        let items = self.menu.items.iter().map(|item| ListItem::new(item.title.clone())).collect::<Vec<ListItem>>();
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
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Char('u')) => self.update_dotfiles(),
            (_, KeyCode::Home) => self.select_first(),
            (_, KeyCode::End) => self.select_last(),
            (_, KeyCode::Up) => self.select_previous(),
            (_, KeyCode::Down) => self.select_next(),
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

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }

    fn update_dotfiles(&mut self) {
        let output = Command::new("git")
            .arg("pull")
            .arg("-r")
            .arg("--autostash")
            .output()
            .expect("Failed to update dotfiles");
        println!("output: {:?}", output);
    }
}

use common::Message;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::{error::Error, time::Duration};

use std::sync::mpsc;
use std::thread;
enum InputMode {
    Normal,
    Editing,
}

enum AppMode {
    Settings(usize),
    Chat,
}
const MAX_MESSAGES: usize = 50;
const COLORS: &[&str] = &[
    "Red",
    "Blue",
    "Green",
    "Yellow",
    "Magenta",
    "Dark Gray",
    "Light Red",
    "Light Green",
    "Light Yellow",
    "Light Blue",
    "Light Magenta",
    "Light Cyan",
];

/// App holds the state of the application
struct App<'a> {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    /// Current input mode
    input_mode: InputMode,
    app_mode: AppMode,
    /// History of recorded messages
    messages: VecDeque<Message<'a>>,

    stream: TcpStream,
}

impl<'a> App<'a> {
    fn new() -> Self {
        Self {
            app_mode: AppMode::Settings(0),
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: VecDeque::with_capacity(MAX_MESSAGES),
            cursor_position: 0,
            stream: TcpStream::connect("127.0.0.1:80").unwrap(),
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
    fn push_message(&mut self, msg: Message<'a>) {
        self.messages.push_back(msg);
        if self.messages.len() > MAX_MESSAGES {
            self.messages.pop_front();
        }
    }

    fn submit_message(&mut self) {
        if !self.input.is_empty() {
            self.input.push('\n');
            if let Err(_) = self.stream.write_all(self.input.as_bytes()) {
                self.push_message(Message::error("Disconnected from server"))
            }
            self.input.clear();
            self.reset_cursor();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let (send, recv) = mpsc::channel();

    let mut reader = BufReader::new(app.stream.try_clone()?);
    thread::spawn(move || loop {
        let mut buf = Vec::new();
        if let Err(err) = reader.read_until(b'\n', &mut buf) {
            send.send(Message::error(&err.to_string())).unwrap();
            break;
        }

        send.send(
            serde_json::from_slice::<Message>(&buf)
                .unwrap_or(Message::error("invalid data from server")),
        )
        .unwrap();
    });
    loop {
        if let Ok(message) = recv.try_recv() {
            app.push_message(message);
        }
        terminal.draw(|f| ui(f, &app))?;

        if let Some(Event::Key(key)) = event::poll(Duration::from_millis(100))?
            .then(event::read)
            .transpose()?
        {
            if key.kind == KeyEventKind::Press {
                if let AppMode::Settings(ref mut color_index) = app.app_mode {
                    match key.code {
                        KeyCode::Up => *color_index = color_index.saturating_sub(1),
                        KeyCode::Down => *color_index = (COLORS.len() - 1).min(*color_index + 1),
                        _ => {}
                    }
                }
            }

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }

                    KeyCode::Char(' ') if key.kind == KeyEventKind::Press => {
                        app.app_mode = match app.app_mode {
                            AppMode::Settings(_) => AppMode::Chat,
                            AppMode::Chat => AppMode::Settings(0),
                        }
                    }
                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => app.submit_message(),
                    KeyCode::Char(to_insert) => {
                        app.enter_char(to_insert);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                "q".bold(),
                " to exit, ".into(),
                "e".bold(),
                " to start editing.".into(),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                "Press ".into(),
                "Esc".bold(),
                " to stop editing, ".into(),
                "Enter".bold(),
                " to record the message".into(),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Line::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_str())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal => {}

        InputMode::Editing => f.set_cursor(
            chunks[1].x + app.cursor_position as u16 + 1,
            chunks[1].y + 1,
        ),
    }
    match app.app_mode {
        AppMode::Settings(color_index) => {
            let colors: Vec<ListItem> = COLORS
                .iter()
                .map(|c| {
                    let text =
                        Span::styled(format!("{}", c), Style::default().fg(c.parse().unwrap()));
                    ListItem::new(Line::from(text))
                })
                .collect();
            let colors = List::new(colors)
                .block(Block::default().borders(Borders::ALL).title("Colors"))
                .highlight_style(Style::default().bold())
                .highlight_symbol(">> ");
            let mut state = ListState::default().with_selected(Some(color_index));
            f.render_stateful_widget(colors, chunks[2], &mut state);
        }
        AppMode::Chat => {
            let messages: Vec<ListItem> = app
                .messages
                .iter()
                .rev()
                .map(|m| {
                    let user_name =
                        Span::styled(format!("{}: ", m.user_name), Style::default().fg(m.color));
                    let text = Span::raw(format!("{}", m.content));
                    ListItem::new(Line::from(vec![user_name, text]))
                })
                .collect();
            let messages = List::new(messages)
                .block(Block::default().borders(Borders::ALL).title("Messages"))
                .start_corner(Corner::BottomLeft);

            f.render_widget(messages, chunks[2]);
        }
    }
}

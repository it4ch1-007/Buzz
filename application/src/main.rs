use std::io;
use std::sync::Arc;
use futures::{stream, SinkExt, StreamExt};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_util::codec::{FramedWrite,FramedRead, LinesCodec};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::block_in_place;
use tokio::time::{timeout, Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Debug, PartialEq)]
enum Key {
    Char(char),
    Enter,
    Backspace,
    Left,
    Right,
    Esc,
    Ctrl(char),
    Alt(char),
    Null,
}

struct Input {
    key: Key,
    ctrl: bool,
    alt: bool,
}

impl From<KeyEvent> for Input {
    fn from(event: KeyEvent) -> Self {
        let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
        let alt = event.modifiers.contains(KeyModifiers::ALT);
        let key = match event.code {
            KeyCode::Char(c) if ctrl => Key::Ctrl(c),
            KeyCode::Char(c) if alt => Key::Alt(c),
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Enter => Key::Enter,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Esc => Key::Esc,
            _ => Key::Null,
        };

        Input { key, ctrl, alt }
    }
}

// Define the ChatApp struct
struct ChatApp {
    input: String,
    messages: Arc<Mutex<Vec<String>>>,
}

impl ChatApp {
    // Create a new instance of ChatApp and connect to the server
    async fn new(addr: &str) -> io::Result<(ChatApp, TcpStream)> {
        let stream = TcpStream::connect(addr).await?;
        Ok((
            ChatApp {
            input: String::new(),
            messages: Arc::new(Mutex::new(vec!["Welcome to the chat!".to_string()])),
            
        },
    stream,
))
    }
    // Run the chat application
    async fn run(&mut self, mut stream: TcpStream) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let messages_clone = Arc::clone(&self.messages);
        let (mut read_half, mut write_half) = stream.into_split();
        let mut reader = FramedRead::new(read_half,LinesCodec::new());



        // Spawn a task to read messages from the server
        let mut writer = FramedWrite::new(write_half,LinesCodec::new());
        tokio::spawn(async move {
            loop {
                let mut msg = reader.next().await;
                let user_msg = match msg {
                    Some(msg) => msg.unwrap(),
                    None => break,
                };
                messages_clone.lock().await.push(user_msg);
            }
        });
        

        //Loop for sending the messages to the server
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                self.input.push(c);
                            }
                            KeyCode::Enter => {
                                
                                if !self.input.is_empty() {
                                    let message_to_send = self.input.clone();
                                    self.input.clear();

                                    writer.send(message_to_send).await.unwrap();
                                }
                            }
                            KeyCode::Backspace => {
                                self.input.pop();
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
        }

            // Render the UI
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Min(1),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                // Render messages
                let messages = {
                    let messages_guard = block_in_place(|| self.messages.blocking_lock());

                    let items: Vec<ListItem> = messages_guard.iter()
                        .map(|m| ListItem::new(vec![Spans::from(Span::raw(m.clone()))]))
                        .collect();

                    List::new(items)
                        .block(Block::default().borders(Borders::ALL).title("Messages"))
                };

                f.render_widget(messages, chunks[0]);

                // Render input box
                let input = Paragraph::new(self.input.as_ref())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Input"));
                f.render_widget(input, chunks[1]);
            })?;
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (mut app,mut stream) = ChatApp::new("127.0.0.1:8080").await?;
    app.run(stream).await
}

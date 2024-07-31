use std::io;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::task::block_in_place;
use tokio::sync::{Mutex, MutexGuard};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

struct ChatApp {
    input: String,
    messages: Arc<Mutex<Vec<String>>>,
    stream: Arc<Mutex<TcpStream>>,
}

impl ChatApp {
    async fn new(addr: &str) -> io::Result<ChatApp> {
        let stream = TcpStream::connect(addr).await?;
        Ok(ChatApp {
            input: String::new(),
            messages: Arc::new(Mutex::new(vec!["Welcome to the chat!".to_string()])),
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    async fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let messages_clone = Arc::clone(&self.messages);
        let stream_clone = Arc::clone(&self.stream);

        // Clone the stream for the reader task
        let stream_for_reader = Arc::clone(&stream_clone);

        // Spawn a task to read messages from the server
        tokio::spawn(async move {
            
            
            loop {
                let mut line = String::new();
                let mut stream = stream_for_reader.lock().await;
                let mut reader = BufReader::new(&mut *stream);
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        messages_clone.lock().await.push(format!("Server: {}", line.trim()));
                    }
                    Err(_) => break,
                }
            }
        });

        loop {
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

            // Properly scope the lock and conversion
            let messages = {
                let messages_guard = block_in_place(|| {
                    self.messages.blocking_lock()
                });
                let items: Vec<ListItem> = messages_guard.iter()
                    .map(|m| ListItem::new(vec![Spans::from(Span::raw(m.clone()))]))
                    .collect();
                
                List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Messages"))
            };
            
            f.render_widget(messages, chunks[0]);

                let input = Paragraph::new(self.input.as_ref())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title("Input"));
                f.render_widget(input, chunks[1]);
            })?;

            // Poll for events
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Enter => {
                            if !self.input.is_empty() {
                                let mut stream = self.stream.lock().await;
                                stream.write_all(self.input.as_bytes()).await?;
                                stream.write_all(b"\n").await?;
                                self.messages.lock().await.push(format!("You: {}", self.input));
                                self.input.clear();
                            }
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
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
    let mut app = ChatApp::new("127.0.0.1:8080").await?;
    app.run().await
}

// use std::io;
// use tui::{
//     backend::CrosstermBackend,
//     layout::{Constraint, Direction, Layout},
//     style::{Color, Modifier, Style},
//     text::{Span, Spans},
//     widgets::{Block, Borders, List, ListItem, Paragraph},
//     Terminal,
// };
// use crossterm::{
//     event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
//     execute,
//     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
// };

// struct ChatApp {
//     input: String,
//     messages: Vec<String>,
// }

// impl ChatApp {
//     fn new() -> ChatApp {
//         ChatApp {
//             input: String::new(),
//             messages: vec!["Welcome to the chat!".to_string()],
//         }
//     }

//     fn run(&mut self) -> io::Result<()> {
//         enable_raw_mode()?;
//         let mut stdout = io::stdout();
//         execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
//         let backend = CrosstermBackend::new(stdout);
//         let mut terminal = Terminal::new(backend)?;

//         loop {
//             terminal.draw(|f| {
//                 let chunks = Layout::default()
//                     .direction(Direction::Vertical)
//                     .margin(2)
//                     .constraints(
//                         [
//                             Constraint::Min(1),
//                             Constraint::Length(3),
//                         ]
//                         .as_ref(),
//                     )
//                     .split(f.size());

//                 let messages: Vec<ListItem> = self
//                     .messages
//                     .iter()
//                     .map(|m| {
//                         ListItem::new(vec![Spans::from(Span::raw(m))])
//                     })
//                     .collect();

//                 let messages = List::new(messages)
//                     .block(Block::default().borders(Borders::ALL).title("Messages"));

//                 f.render_widget(messages, chunks[0]);

//                 let input = Paragraph::new(self.input.as_ref())
//                     .style(Style::default().fg(Color::Yellow))
//                     .block(Block::default().borders(Borders::ALL).title("Input"));
//                 f.render_widget(input, chunks[1]);
//             })?;

//             if let Event::Key(key) = event::read()? {
//                 match key.code {
//                     KeyCode::Enter => {
//                         self.messages.push(format!("You: {}", self.input));
//                         self.input.clear();
//                     }
//                     KeyCode::Char(c) => {
//                         self.input.push(c);
//                     }
//                     KeyCode::Backspace => {
//                         self.input.pop();
//                     }
//                     KeyCode::Esc => {
//                         break;
//                     }
//                     _ => {}
//                 }
//             }
//         }

//         disable_raw_mode()?;
//         execute!(
//             terminal.backend_mut(),
//             LeaveAlternateScreen,
//             DisableMouseCapture
//         )?;
//         terminal.show_cursor()?;

//         Ok(())
//     }
// }

// fn main() -> io::Result<()> {
//     let mut app = ChatApp::new();
//     app.run()
// }
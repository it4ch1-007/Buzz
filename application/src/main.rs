mod input;
mod ui;

use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    stdout.execute(crossterm::terminal::EnterAlternateScreen).unwrap();
    stdout.execute(crossterm::cursor::Hide).unwrap();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = ui::App::new();

    tokio::spawn(async move {
        input::connect(&mut app).await;
    });

    loop{
        app.draw(&mut terminal);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    disable_raw_mode().unwrap();
    stdout.execute(crossterm::terminal::LeaveAlternateScreen).unwrap();
    stdout.execute(crossterm::cursor::Show).unwrap();
}
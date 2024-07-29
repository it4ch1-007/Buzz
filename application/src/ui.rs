//This file makes a ui for the application and each ui needs the messages and input

use ratatui::backend::CrosstermBackend; //To enable the cross terminal backend for the application
use ratatui::layout::{Constraint,Direction,Layout};
use ratatui::style::{Color,Style};
use ratatui::widgets::{Block,Borders,List,ListItem,Paragraph};
use ratatui::Terminal;
use std::io::Stdout;
use std::borrow::Cow;


//This is the basic struct that will get all the resources we need for the application to function its ui
pub struct App{
    pub messages: Vec<String>,
    pub input: String,
}

impl App{
    pub fn new() -> App{
        App{
            messages: Vec::new(),
            input: String::new(),
        }
    }

    //ratatui standard function to draw an application in the terminal
    pub fn draw(&self,terminal: &mut Terminal<CrosstermBackend<Stdout>>){
        let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        //Gets us the reference to all the chunks organized vertically
        .split(terminal.size().unwrap());

    //Making the messages show pane window
    let messages: Vec<ListItem> =self
        .messages
        .iter()
        .map(|m| ListItem::new(m.clone()))
        .collect();

    //Making the messages list pane window
    let message_list  = List::new(messages)
    .block(Block::default().borders(Borders::ALL).title("Messages"));

    //Making the input bar window pane to enter the input
    let input = Paragraph::new(Cow::Borrowed(self.input.as_ref()))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    terminal.draw(|f| {
        f.render_widget(message_list,chunks[0]);
        f.render_widget(input,chunks[1]);
        }).unwrap();

    //We can actually say that the terminal is having the chunks of Input pane and the messages pane inside it 
    }
}
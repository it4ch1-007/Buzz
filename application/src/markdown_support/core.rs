use std::thread::current;
use pulldown_cmark::{Parser, Event, Tag};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};

fn parse_markdown(input: &str) {
    let parser = Parser::new(input);
    let mut spans = vec![];
    let mut current_line = vec![];
    
    for event in parser{
        match event{
            // Event::Start(tag) => {
            //     match tag{
            //         Tag::CodeBlock(_) => {
            //             //TODO: WHEN THE MARKDOWN HAS CODE WRITTEN INSIDE 
            //         },
            //         _ => {}
            //     }
            // }
            // Event::End(tag) => {
            //     match tag{
            //         Tag::CodeBlock(_) => {
            //             //TODO: When it is the end of the code written portion
            //         },
            //         _ => {}
            //     }
            // }
            Event::Text(text ) => {
                current_line.push(Span::raw(text.to_string())); //Simply puishing the string if not markdown
            }
            Event::Code(code_text) => {
                current_line.push(
                    Span::styled(
                        code_text.to_string(),
                        Style::default().fg(Color::Green).bg(Color::Black),
                    )
                );
            }
            Event::SoftBreak | Event::HardBreak => {
                spans.push(Spans::from(current_line));
                current_line = vec![];
            }
            _ => {}
        };
        
    }
}
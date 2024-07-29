//Every activity generally has a different rust file and functions to be able to handle it properly apart from the frontend on the terminal
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead,FramedWrite,LinesCodec};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crossterm::event::{self, Event, KeyCode};
use crate::ui::App;
use futures::SinkExt;


pub async fn handle_input<T>(app: &mut App, mut writer: FramedWrite<T,LinesCodec>) 
where
    T: tokio::io::AsyncWrite + Unpin,
{
    loop {
        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        let msg = app.input.drain(..).collect::<String>();
                        app.messages.push(msg.clone());
                        writer.send(msg.as_bytes()).await.unwrap();
                        writer.send(b"\n").await.unwrap();
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
pub async fn handle_server(app: &mut App,mut reader: FramedRead<T,LinesCodec>){
    let mut line = String::new();
    loop{

        let msg = reader.next();
        match  msg{
            Ok(0) => break, // Server closed connection
            Ok(_) => {
                app.messages.push(msg.clone());
                msg.clear();
            }
            Err(_) => break,
        }
    }
}

pub async fn connect(app: &mut App) {
    let mut tcp = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (mut tx,mut rx) = tcp.split();
    let stream = FramedRead::new(tx,LinesCodec::new());
    let sink = FramedWrite::new(rx,LinesCodec::new());

    //this decides and takes action on the basis of which fn amnong handle_input and handle_server give out the result

    tokio::select!{
        _ = handle_server(app,stream) => {},
        _ = handle_input(app,sink) => {}
    }
}
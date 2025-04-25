mod file_uploading;
mod helper_fns;
mod room;
mod names;
mod clients;


use futures::{SinkExt, StreamExt};
use lazy_static;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Sender};
use room::core::Room;
use names::core::Names;
use clients::handle::handle_clients;
#[derive(Clone)]
struct Rooms(Arc<RwLock<HashMap<String, Room>>>);
//Here a read write lock is implemented because for a read-write lock multiple clients can have read access to the messages at the same time apart from the mutex lock that would have given both the write as well as the read lock to just one client.

impl Rooms {
    fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }
    //Making a new entry to the room

    //Joining an existing room
    fn join_room(&self, room_name: &str) -> Sender<String> {
        let read_guard = self.0.read().unwrap();
        if let Some(room) = read_guard.get(room_name) {
            return room.tx.clone();
            //Making a one more transmitter for the same room for the client that is going to join the room
        }
        drop(read_guard);
        //Unlocking as soon as the transmitter is made
        let mut write_guard = self.0.write().unwrap();
        let room = write_guard
            .entry(room_name.to_owned())
            .or_insert(Room::new());
        //If the room is not there then this makes a new room with that name
        room.tx.clone()
        //Making a transmitter for the entire group
    }

    fn list_rooms(&self) -> Vec<(String, usize)> {
        let mut list: Vec<_> = self
            .0
            .read()
            .unwrap()
            .iter()
            .map(|(name, room)| (name.to_owned(), room.tx.receiver_count()))
            .collect();
        //Getting the| receivers of a transmitter that is initialized inside a room will give us the number of clients inside the room.
        list.sort_by(|a, b| {
            use std::cmp::Ordering::*;
            //This is sorted in order so that the names and the participants number sync with each other.
            match b.1.cmp(&a.1) {
                Equal => a.0.cmp(&b.0),
                ordering => ordering,
            }
        });
        list
    }
}


#[tokio::main]
async fn main() {
    let server = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    //Making a main server for the whole application

    let (tx, _) = broadcast::channel::<String>(40);
    //Making a broadcast connection between server and all the clients.

    let mut rooms = Rooms::new();
    let mut names = Names::new();
    loop {
        let (mut tcp, _) = server.accept().await.unwrap();
        tokio::spawn(handle_clients(
            tcp,
            tx.clone(),
            names.clone(),
            rooms.clone(),
        ));
        //For every different client a new task will be spawned and thus it is ensured that the two clients tasks donot clash with each other.
    }
}


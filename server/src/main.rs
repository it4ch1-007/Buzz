mod file_uploading;
mod helper_fns;

use futures::{SinkExt, StreamExt};
use lazy_static;
use rpassword::read_password;
use server::{get_name,set_password,get_password, GetName};
use std::collections::{HashMap, HashSet};
use std::fs::{self, read, File};
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Sender};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

use file_uploading::core::upload;
use helper_fns::notify::show_notification;

const HELP_MSG: &str = "This is help for each of the client";
const MAIN: &str = "main";
pub struct Room {
    tx: Sender<String>,
    //a type of a broadcast sender that can help us to establish connection between two clients
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    //this will be the vector hashmap that will store the files uploaded.
    password: Arc<RwLock<String>>,
}

impl Room {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(40);
        //to make a new broadcast for every room
        Self {
            tx,
            files: Arc::new(RwLock::new(HashMap::new())),
            password: Arc::new(RwLock::new(String::new())),
        }
    }
    //functions to handle the features of the files inside the room's unique files Hashmap
    fn upload_file(&self) {
        let (file_name, content) = upload().unwrap();
        self.files
            .write()
            .unwrap()
            .insert(file_name.split("\\").last().unwrap().to_string(), content);
    }

    fn list_all_files(&self) -> Vec<String> {
        //making a list vector of all the filenames only.
        self.files.read().unwrap().keys().cloned().collect()
    }
    fn download_file(&self, file_name: &String) {
        //download into the local system
        let mut read_guard = self.files.read().unwrap();
        let mut bytes = read_guard.get(file_name).unwrap();
        let mut new_file = File::create(&file_name).unwrap();
        new_file.write_all(bytes);

        //remove it from the hashmap entry
        self.files.write().unwrap().remove(file_name);
    }
    fn set_password(&mut self, pass: String) {
        let mut write_guard = self.password.write().unwrap();
        *write_guard = pass;
    }
    fn get_password(&mut self) -> String {
        self.password.read().unwrap().to_string()
    }
}
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

//Names of the clients
#[derive(Clone)]
struct Names(Arc<Mutex<HashSet<String>>>);
//Mutex lock for the names coz the names must be assigned and locked to a single client only.

impl Names {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }
    fn insert(&self, name: String) -> bool {
        self.0.lock().unwrap().insert(name)
    }
    fn remove(&self, name: &str) -> bool {
        self.0.lock().unwrap().remove(name)
    }
    fn get_unique(&self) -> String {
        let mut name = get_name();
        let mut guard = self.0.lock().unwrap();
        while !guard.insert(name.clone()) {
            //Until there is not inserted a unique name for the locked resource client the name we get from the function will be the name of the client.
            name = get_name()
        }
        name
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

async fn handle_clients(mut tcp: TcpStream, tx: Sender<String>, names: Names, rooms: Rooms) {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    // let mut rx = tx.subscribe();
    //whichever client uses this receiver will be connected to the broadcast channel.
    let mut name = names.get_unique(); //The name for the new client
                                       //Get a unique default name
    sink.send(format!("You are {}", name)).await.unwrap();
    //Gets the context of the whole main function environment
    let mut room_name = MAIN.to_owned();
    let mut room_tx = rooms.join_room(&room_name);
    let mut room_rx = room_tx.subscribe();
    room_tx.send(format!("{name} joined {room_name}")).unwrap();
    //This is sent only inside the room for the room clients
    loop {
        tokio::select! {
            msg = stream.next() => {
            //This is the way for the server to get commands from all the clients using the application
                let user_msg = match msg {
                    Some(msg) => msg.unwrap(),
                    None => break,
                };
                if user_msg.starts_with("/help"){
                    sink.send(HELP_MSG).await.unwrap();
                    continue;
                }
                //Basically stream and sink are used for the messages sending and receiving by the server and the clients
                else if user_msg.starts_with("/rooms"){
                    let rooms_list = rooms.list_rooms();
                    let rooms_list = rooms_list
                    .into_iter()
                    .map(|(name,count)| format!("{name} ({count})"))
                    .collect::<Vec<_>>()
                    .join(", ");
                //We first created a vector with the elements as tuples that have name and count as elements of a single tuple'
                //then we join it with , as the separator
                    sink.send(format!("Rooms -> {rooms_list}")).await.unwrap();
                }
                else if user_msg.starts_with("/join"){
                    let new_room = user_msg
                    .split_ascii_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_owned();
                //Get the 1th element after splitting the string whitespaces and then joining the room
                if room_name == new_room{
                    sink.send(format!("You are already inside the room {room_name}")).await.unwrap();
                    continue;
                }
                //Check for the entered password and the password of the room
                let password = read_password().unwrap();
                if get_password() == password{
                //if we want to get a mutable reference then we have to use the write lock instead of a read lock always
                
                //The room_name is now the previous one and thus the client will leave the room
                room_tx.send(format!("{name} left {room_name}")).unwrap();
                //The client gets the tx and rx of the new room
                room_tx = rooms.join_room(&new_room);
                //This statement updates the room_name
                room_rx = room_tx.subscribe();
                room_name = new_room;
                sink.send(format!("{name} joined {room_name}")).await.unwrap();
                room_tx.send(format!("{name} joined {room_name}")).unwrap();
                }
                else{
                    sink.send(format!("You entered the wrong password!!!")).await.unwrap();
                }
            }
                else if user_msg.starts_with("/set_password_room"){
                    let password = user_msg
                    .split_ascii_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_owned();
                    let mut write_guard = rooms.0.write().unwrap();
                    if let Some(room) = write_guard.get_mut(&room_name){
                        room.set_password(password);
                        //This does not take the ownership of the Room instance and thus the fn has to be amde get the mutabke reference to the instance.
                    }
                    drop(write_guard);
                }
                else if user_msg.starts_with("/set_password_me"){
                    let password = user_msg
                    .split_ascii_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_owned();
                    set_password(password); //Sets the password for the given user
                }
                else if user_msg.starts_with("/name") {
            //This is to split the whitespaces and then get the first element to get the name
                    let new_name =  user_msg
                    .split_ascii_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_owned();
               //For changing the unique name
               let changed_name = names.insert(new_name.clone());
               //This is to insert the new names inside the list of the names
               if changed_name {
                   //only the room clients will get the names update
                   room_tx.send(format!("{name} is now {new_name}")).unwrap();
                   name = new_name;
               }
                else{
                    sink.send(format!("{new_name} is already taken")).await.unwrap();
                }
                }
                else if user_msg.starts_with("/video_call"){
                    let friend_name = user_msg
                    .split_ascii_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_owned();

                }
                else if user_msg.starts_with("/upload_file"){
                    let read_guard = rooms.0.read().unwrap();
                    if let Some(room) = read_guard.get(&room_name){
                        room.upload_file();
                    }
                    drop(read_guard);
                    show_notification(format!("Uploaded specified file"));

                }
                else if user_msg.starts_with("/list_files"){
                    let read_guard = rooms.0.read().unwrap();
                    let mut files_list_string = "".to_string();
                    if let Some(room) = read_guard.get(&room_name){
                        let files_list = room.list_all_files();
                        files_list_string = files_list.join("\n");
                    }
                    drop(read_guard);
                    show_notification(format!("Files available -> \n{files_list_string}"));

                }
                else if user_msg.starts_with("/download_file"){
                    let file_name = user_msg.split_ascii_whitespace().nth(1).unwrap().to_owned();
                    show_notification(format!("Downloaded file {file_name}"));
                    let read_guard = rooms.0.read().unwrap();
                    if let Some(room) = read_guard.get(&room_name){
                        room.download_file(&file_name);
                    }
                    drop(read_guard);

                }
                else if user_msg.starts_with("/quit"){
                    break;
                }
                else{
                    room_tx.send(format!("{name}: {user_msg}")).unwrap();
                }
            },
            //WHEN WE USE THE TOKIO SELECT STATEMENT THEN THE TOKENS WE USE ARE CONVERTED INTO RESULT ENUMS AND THUS WE HAVE TO UNWRAP THEM FIRST BEFORE USING THEM IN THE FUTURES SPAWNED.
            //If there is no command sent then send the message to that particular room
            peer_msg = room_rx.recv() => {
                sink.send(peer_msg.unwrap()).await.unwrap();
            },
        //TOKIO SELECT ACTUALLY POLLS MULTIPLE FUTURES AT ONCE
        //This loop is that for getting a message we first have to send a message thus we will add the select message for choosing the event whichever happens first inside the async tasks

        //when the loop ends the client is disconnected thus he will have left the room

        }
    }

    room_tx.send(format!("{name} left the {room_name}")).unwrap();
    names.remove(&name);
    //If the loop ends then the client has ended his session and has left the chat application.
}

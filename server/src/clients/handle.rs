use crate::helper_fns::notify::show_notification;
use crate::names::core::Names;
use crate::Rooms;
use futures::{SinkExt, StreamExt};
use rpassword::read_password;
use server::{get_password, set_password};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Sender;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

const HELP_MSG: &str = "This is help for each of the client";
const MAIN: &str = "main";

pub async fn handle_clients(mut tcp: TcpStream, tx: Sender<String>, names: Names, rooms: Rooms) {
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
                
                
                
                
                else if user_msg.starts_with("/set_password_mine"){
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



                //SENDING AUDIO MESSAGES
                //Command should be like: /audio_msg <audio_msg_name> (specified by the sender)
                else if user_msg.starts_with("/audio_msg"){
                    let audio_msg_name = user_msg
                    .split_ascii_whitespace()
                    .nth(1)
                    .unwrap()
                    .to_owned();
                    room_tx.send(format!("{name} uploaded audio: {audio_msg_name}")).unwrap();
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

    room_tx
        .send(format!("{name} left the {room_name}"))
        .unwrap();
    names.remove(&name);
    //If the loop ends then the client has ended his session and has left the chat application.
}

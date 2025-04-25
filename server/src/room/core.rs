use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use crate::file_uploading::core::upload;

pub struct Room {
    pub(crate) tx: Sender<String>,
    //a type of a broadcast sender that can help us to establish connection between two clients
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    //this will be the vector hashmap that will store the files uploaded.
    // audio_msgs: Arc<RwLock<HashSet<T>>>,
    password: Arc<RwLock<String>>,
}

impl Room {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(40);
        //to make a new broadcast for every room
        Self {
            tx,
            files: Arc::new(RwLock::new(HashMap::new())),
            // audio_msgs: Arc::new(RwLock::new(HashMap::new())),
            password: Arc::new(RwLock::new(String::new())),
        }
    }
    //functions to handle the features of the files inside the room's unique files Hashmap
    pub fn upload_file(&self) {
        let (file_name, content) = upload().unwrap();
        self.files
            .write()
            .unwrap()
            .insert(file_name.split("\\").last().unwrap().to_string(), content);
    }

    pub fn list_all_files(&self) -> Vec<String> {
        //making a list vector of all the filenames only.
        self.files.read().unwrap().keys().cloned().collect()
    }
    pub fn download_file(&self, file_name: &String) {
        //download into the local system
        let mut read_guard = self.files.read().unwrap();
        let mut bytes = read_guard.get(file_name).unwrap();
        let mut new_file = File::create(&file_name).unwrap();
        new_file.write_all(bytes);

        //remove it from the hashmap entry
        self.files.write().unwrap().remove(file_name);
    }
    pub fn set_password(&mut self, pass: String) {
        let mut write_guard = self.password.write().unwrap();
        *write_guard = pass;
    }
    pub fn get_password(&mut self) -> String {
        self.password.read().unwrap().to_string()
    }
}
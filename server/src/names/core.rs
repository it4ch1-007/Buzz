use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use server::get_name;

//Names of the clients
#[derive(Clone)]
pub struct Names(Arc<Mutex<HashSet<String>>>);
//Mutex lock for the names coz the names must be assigned and locked to a single client only.

impl Names {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashSet::new())))
    }
    pub fn insert(&self, name: String) -> bool {
        self.0.lock().unwrap().insert(name)
    }
    pub fn remove(&self, name: &str) -> bool {
        self.0.lock().unwrap().remove(name)
    }
    pub fn get_unique(&self) -> String {
        let mut name = get_name();
        let mut guard = self.0.lock().unwrap();
        while !guard.insert(name.clone()) {
            //Until there is not inserted a unique name for the locked resource client the name we get from the function will be the name of the client.
            name = get_name()
        }
        name
    }
}
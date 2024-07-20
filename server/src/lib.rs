use std::sync::Mutex;
use lazy_static::lazy_static;

pub struct GetName{
    pub name: String,
    pub digit: u32,
}

//A global static variables wrapped inside a mutex so that we can use the value of the variable at global scope and retrieve its value if modified by any function

lazy_static! {
    pub static ref GLOBAL_NAME: Mutex<GetName> = Mutex::new(GetName{
        name: "Anon".to_string(),
        digit:0,
    });
}
//This is the initialization of a global variable

pub fn get_name() -> String{
    let mut name_instance = GLOBAL_NAME.lock().unwrap();
    name_instance.digit+=1;
    format!("{}{}",name_instance.name,name_instance.digit)
}
use rfd::FileDialog;
pub fn upload()->Result<((String,Vec<u8>)),std::io::Error>{
    let mut file_name = open_file();
    let mut content = std::fs::read(file_name.clone()).expect("could not read the specified file");
    Ok((file_name,content))
}
fn open_file()->String{
    if let Some(path) = FileDialog::new().pick_file(){
        path.to_string_lossy().to_string()
    }
    else{
        panic!("No file selected");
    }
}
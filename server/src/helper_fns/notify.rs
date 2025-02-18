use notify_rust::Notification;
pub fn show_notification(notification:String){
    Notification::new()
        .summary("New Message")
        .body(&notification)
        .icon("dialog-information") // Optional
        .show()
        .unwrap();
}
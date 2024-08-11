# Buzz 

Buzz is a modern peer-to-peer as well as peer-to-server chat application developed using the two most famous and useful Rust libraries `Tokio` and `RataTui`.

Among modern chat applications, where the users cannot trust the servers of the application, Buzz is a totally trustable chat application that maintains the privacy of the users to full extent giving them the feature to even choose their own names and rooms in which they can talk privately.

### Key Features
- Faster<br>
    Use of asynchronous functions through `Tokio-rs` makes Buzz faster and effective due to its ability to `spawn multiple tasks to handle each client` and `create multiple broadcast and mpsc channels`
- Rooms<br>
    Users can create their own rooms and join them to talk about a particular topic with the related people who are interested in the same topic.
- Names<br>
    For the beginning, Every user is given a temporary anonymous name starting with `Anon<user_number>`. This ensures total anonymity for the user. However, they have the freedom to choose their names using the `/name` command.
- Locks<br>
    The exclusive Mutex and ReadWrite locks provides privacy to the users and not even allows the server to have any control or access to the messages between the peers.
- CLI-GUI<br>
    The application has a terminal GUI developed using RataTui-rs with input and message boxes. This GUI works faster and efficiently more than the modern GUI applications.

### Useful Commands
- `/rooms` -> gives the list of the rooms and the clients inside the room.
- `/join <room_name>` -> allows the client to join the room with a particular room name and creates a new room if the room with the given name does not exist.
- `/name <new_name>` -> To change the name displayed while chatting
- `/quit` -> Quits the application

### How to Use
- Install Rust using: `https://www.rust-lang.org/tools/install`
- Clone the github repo of Buzz.
- Execute the following commands:

```
cd server
cargo run
```
- Now open a new `Terminal/Powershell` window and execute the commands;

```
cd <Buzz_dir_name>
cd application
cargo run
```
- The above commands runs the application in the current terminal with the GUI. Every window that runs the application.exe will result in a new user and the server allows the users to talk to each other.




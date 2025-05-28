#![allow(unused)] //FIXME: WIP

use std::io::{BufReader, Read, Write};
use std::{result, thread};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream, Shutdown};

use::common_bork::{Message, MessageType};

type Result<T> = result::Result<T, ()>;

const SERVER_PORT:u16=6556;
const SERVER_ADDRESS:&str = "0.0.0.0";
const DB_NAME:&str = "borkbork.db";
const WELCOME:&str = "
        __
     __/o \\_
     \\____  \\
         /   \\
   __   //\\   \\
__/o \\-//--\\   \\_/
\\____  ___  \\  |
     ||   \\ |\\ |
    _||   _||_||

WELCOME TO BORK BORK, A PLACE
    FOR LAKEDOGS TO BORK ABOUT
";


#[derive(Clone)]
struct User{
    displayname:    String,
    online:         bool,
}
impl User{
    fn new(displayname: String) -> User{
        User{
            displayname,
            online : true,
        }
    }
}
struct ServerState{
    user_list: Vec<User>,
}
impl ServerState{
    fn new() -> ServerState{
        ServerState{
            user_list: Vec::new(),
        }
    }
    fn add_user(&mut self, user: &mut User){
        self.user_list.push(Clone::clone(user));
    }
}

fn main() -> Result<()> {
    let address = format!("{}:{}", SERVER_ADDRESS, SERVER_PORT);
    let listener = TcpListener::bind(&address).map_err(|_err| {
        println!("[SERVER_MESSAGE]: Error: could not bind to address {address}");
    })?;
    println!("[SERVER_MESSAGE]: running on socket: {address}");

    let server_state = ServerState::new();
    let server_state = Arc::new(Mutex::new(server_state));

    let (sender, receiver) = channel();
    let receiver = Arc::new(Mutex::new(receiver));
    thread::spawn(move || handle_mspc_thread_messages(receiver));

    for stream in listener.incoming() {
        match stream{
            Ok(stream) => {
                let stream = Arc::new(stream);
                let sender = sender.clone();
                let server_state = Arc::clone(&server_state);
                println!("[SERVER_MESSAGE]: new connection, spawning thread for client {:?}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream, sender, server_state));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_mspc_thread_messages(reciever: Arc<Mutex<Receiver<Message>>>) -> Result<()> {
    println!("[SERVER_MESSAGE]: handling incomming messages from client threads");
    loop{
        let rec = reciever.lock();
        let message = rec
            .unwrap()
            .recv()
            .map_err(|err| {
                println!("[ERROR]: Couldn't receive message, got error: {}", err);
            })?;
        match message{
            Message::Version { author, message_type, major_rev, minor_rev, subminor_rev } => {
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(major_rev.to_le_bytes());
                message.extend(minor_rev.to_le_bytes());
                message.extend(subminor_rev.to_le_bytes());
                author.as_ref().write_all(&message).map_err(|err| {
                    println!("[ERROR]: couldn't send version message to client, with error: {}", err);
                })?;
            }
            Message::ChatMsg { author, message_type, sender_id, message_len, message_text } => {
                println!("received ChatMsg type");
            }
            Message::Join { author, message_type, username } => {
                println!("received Join type");
            }
            Message::Leave { author, message_type } => {
                println!("received Leave type");
            }
            Message::Welcome { author, message_type, message_len, welcome_msg } => {
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(message_len.to_le_bytes());
                message.extend(welcome_msg);
                author.as_ref().write_all(&message).map_err(|err| {
                    println!("Couldn't send welcome message to client, with error {}", err);
                })?;
            }
            _ => {
                println!("received unknown mesage type");
            }
        }
    }

    Ok(())
}

fn handle_client(
    stream: Arc<TcpStream>,
    message: Sender<Message>,
    server_state: Arc<Mutex<ServerState>>) -> Result<()> {

    if stream.peer_addr().is_err() {
        println!("[ERROR]: couldn't get client's peer address.");
        return Err(());
    }
    else {
        println!("[SERVER_MESSAGE]: New connection from {:?}", stream.peer_addr().unwrap());
    }

    let server_version = Message::Version{
        author: stream.clone(),
        message_type: MessageType::VERSION,
        major_rev: 0,
        minor_rev: 0,
        subminor_rev: 1,
    };
    message.send(server_version).map_err(|err| {
        println!("[ERROR]: Couldn't send version message to client. Err was: {}", err);
    })?;
    let welcome = Message::Welcome{
        author: stream.clone(),
        message_type: MessageType::WELCOME,
        message_len: WELCOME.len() as u16,
        welcome_msg: WELCOME.as_bytes().to_vec(),
    };
    message.send(welcome).map_err(|err|{
        println!("[ERROR]: Couldn't send welcome message to MPSC sender. Err was {}",err);
    })?;

    Ok(())
}

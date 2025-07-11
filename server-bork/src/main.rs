#![allow(unused)] //FIXME: WIP

use log::{debug, error, info, LevelFilter};
use std::collections::BTreeMap;
use std::io::{BufReader, Read, Write};
use std::{result, thread};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream, Shutdown};
use uuid::Uuid;

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
    uuid:           Uuid,
}
impl User{
    fn new(displayname: String) -> User{
        User{
            displayname,
            online : true,
            uuid : Uuid::new_v4(),
        }
    }
}
struct ServerState{
    user_map: BTreeMap<String, User>,
}
impl ServerState{
    fn new() -> ServerState{
        ServerState{
            user_map: BTreeMap::new(),
        }
    }
    fn add_user(&mut self, user: &mut User){
        self.user_map.insert(Clone::clone(&user.displayname), Clone::clone(user));
    }
}

fn main() -> Result<()> {
    let _ = simple_logging::log_to_file("./server.log", LevelFilter::Info);
    let address = format!("{}:{}", SERVER_ADDRESS, SERVER_PORT);
    let listener = TcpListener::bind(&address).map_err(|_err| {
        error!("could not bind to address {address}");
    })?;
    info!("running on socket: {address}");

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
                info!("new connection, spawning thread for client {:?}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream, sender, server_state));
            }
            Err(e) => {
                error!("error spawning thread for incomming stream: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_mspc_thread_messages(reciever: Arc<Mutex<Receiver<Message>>>) -> Result<()> {
    info!("handling incomming messages from client threads");
    loop{
        let rec = reciever.lock();
        let message = rec
            .unwrap()
            .recv()
            .map_err(|err| {
                error!("MPSC handler couldn't receive message, got error: {}", err);
            })?;
        match message{
            Message::Version { author, message_type, major_rev, minor_rev, subminor_rev } => {
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(major_rev.to_le_bytes());
                message.extend(minor_rev.to_le_bytes());
                message.extend(subminor_rev.to_le_bytes());
                author.as_ref().write_all(&message).map_err(|err| {
                    error!("MPSC handler couldn't send version message to client, with error: {}", err);
                })?;
                author.as_ref().flush();
            }
            Message::ChatMsg { author, message_type, sender_id, message_len, message_text } => {
                debug!("MPSC handler received ChatMsg type");
            }
            Message::Join { author, message_type, name_len, username } => {
                debug!("MPSC handler received Join type");
            }
            Message::Leave { author, message_type } => {
                debug!("MPSC handler received Leave type");
            }
            Message::Welcome { author, message_type, message_len, welcome_msg } => {
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(message_len.to_le_bytes());
                message.extend(welcome_msg);
                author.as_ref().write_all(&message).map_err(|err| {
                    error!("MPSC couldn't send Welcome message to client, with error {}", err);
                })?;
                author.as_ref().flush();
            }
            Message::Userjoined { author, message_type, user_id, username_len, username } => {
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(user_id.to_bytes_le());      //uuid
                message.extend(username_len.to_le_bytes()); //u16
                message.extend(username);
                author.as_ref().write_all(&message).map_err(|err| {
                    error!("MPSC couldn't send Userjoined message to client, with error {}", err);
                })?;
                author.as_ref().flush();
            }
            _ => {
                info!("MPSC handler received unknown mesage type");
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
        error!("couldn't get client's peer address.");
        return Err(());
    }
    else {
        info!("new connection from {:?}", stream.peer_addr().unwrap());
    }

    /****< Connection preamble: send sever version & welcome to each client>***/
    let server_version = Message::Version{
        author: stream.clone(),
        message_type: MessageType::VERSION,
        major_rev: 0,
        minor_rev: 2,
        subminor_rev: 0,
    };
    message.send(server_version).map_err(|err| {
        error!("couldn't send version message to client. Err was: {}", err);
    })?;
    let welcome = Message::Welcome{
        author: stream.clone(),
        message_type: MessageType::WELCOME,
        message_len: WELCOME.len() as u16,
        welcome_msg: WELCOME.as_bytes().to_vec(),
    };
    message.send(welcome).map_err(|err|{
        error!("couldn't send welcome message to MPSC sender. Err was {}",err);
    })?;
    /*********************</connection preamble>******************************/

    let mut isalive = true;
    let mut reader = BufReader::new(stream.as_ref());
    let mut message_type = [0u8];
    let mut bufr:Vec<u8> = Vec::new();
    loop{
        reader.read_exact(&mut message_type).map_err(|err| {
            error!("couldn't receive message; assuming client disconnect. Error was: {}", err);
            stream.as_ref().shutdown(Shutdown::Both);
            isalive = false;
        });
        if !isalive { break; }

        match message_type[0]{
            MessageType::JOIN => {
                let mut len = [0u8, 0u8];
                match reader.read_exact(&mut len){
                    Err(e) => error!("couldn't read username length from JOIN message. Err was: {}", e),
                    _ => (),
                }
                let len:u16 = u16::from_le_bytes(len);
                let mut uname_buf = vec![0; len as usize];
                match reader.read_exact(&mut uname_buf) {
                    Err(e) => error!("couldn't read {} bytes (expected for Username length)", len),
                    _ => (),
                }
                let uname = String::from_utf8(uname_buf.clone()).unwrap();
                match server_state.lock().unwrap().user_map.get(&uname) {
                    Some(u) => {
                        let mut u : User = User::new(Clone::clone(&uname));
                        server_state.lock().unwrap().add_user(&mut u);
                    },
                    None => info!("User with name {} already exists on the server; nothing to do", uname),
                }
                let userjoin = Message::Userjoined {
                    author: stream.clone(),
                    message_type: MessageType::USERJOINED,
                    user_id: (server_state.lock().unwrap().user_map.get(&uname).unwrap() as &User).uuid,
                    username_len: len,
                    username: uname_buf.clone(),
                };
                message.send(userjoin).map_err(|err|{
                    error!("couldn't send USERJOINED message to MPSC sender. Err was {}",err);
                })?;

            }
            _ => {
                info!(
                    "the client sent an unknown message type, with ID: {}; ignoring message contents",
                    message_type[0]
                );
            }
        }
    }

    Ok(())
}

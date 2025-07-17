#![allow(unused)] //FIXME: WIP

use common_bork::UserStatusType;
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

// TODO: should probably be in common-bork
#[derive(Clone, Debug)]
struct User{
    description:    String,
    displayname:    String,
    status:         u8,
    uuid:           Uuid,
}
impl User{
    fn new(displayname: String, uuid:Uuid) -> User{
        User{
            description: String::from("A nondescript llama"),
            displayname,
            status : UserStatusType::OFFLINE,
            uuid,
        }
    }
}
struct ServerState{
    user_map: BTreeMap<Uuid, User>,
    conns: BTreeMap<String, Arc<TcpStream> >,
}
impl ServerState{
    fn new() -> ServerState{
        ServerState{
            user_map: BTreeMap::new(),
            conns: BTreeMap::new(),
        }
    }
    fn add_user(&mut self, user: &mut User){
        self.user_map.insert(Clone::clone(&user.uuid), Clone::clone(user));
    }
}

fn main() -> Result<()> {
    let _ = simple_logging::log_to_file("./server.log", LevelFilter::Debug);
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

fn handle_broadcast_message(){
    //fixme
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
            Message::UserList { author, message_type, uuid_list } => {
                let mut message: Vec<u8> = Vec::new();
                let len:u16 = u16::try_from(uuid_list.len()).expect("Could not get u16 from uuid_list.len() (a usize downcast)");
                message.push(message_type);
                message.extend(len.to_le_bytes());
                for u in uuid_list.iter(){
                    message.extend(u.to_bytes_le());
                }
                author.as_ref().write_all(&message).map_err(|err| {
                    error!("MPSC couldn't send UserList message to client, with error {}", err);
                })?;
                author.as_ref().flush();
            }
            Message::UserStatus { author, message_type, user_id, status_type, name_len, desc_len, username, desc } => {
                info!("Sending UserStatus message");
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(user_id.to_bytes_le());
                message.extend(status_type.to_le_bytes());
                message.extend(name_len.to_le_bytes());
                message.extend(desc_len.to_le_bytes());
                message.extend(username);
                message.extend(desc);
                author.as_ref().write_all(&message).map_err(|err| {
                    error!("MPSC couldn't send UserStatus message to client, with Err: {}", err);
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
        server_state.lock().unwrap().conns.insert(
            stream.peer_addr().unwrap().to_string().clone(),
            stream.clone()
        );
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
    let mut client_uuid = Uuid::from_u128(0);
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
                info!("received JOIN message from {}", stream.peer_addr().unwrap());
                let mut len_buf = [0u8;2];
                match reader.read_exact(&mut len_buf[..]){
                    Err(e) => error!("couldn't read username length from JOIN message. Err was: {}", e),
                    _ => (),
                }
                let len:u16 = u16::from_le_bytes(len_buf);

                let mut uuid_buf = [0u8;16];
                match reader.read_exact(&mut uuid_buf[..]){
                    Err(e) => error!("Couldn't read uuid from JOIN message. Err was: {}", e),
                    _ => ()
                };
                let mut uuid = Uuid::from_bytes_le(uuid_buf);

                let mut uname_buf = vec![0; len as usize];
                match reader.read_exact(&mut uname_buf[..]) {
                    Err(e) => error!("couldn't read the username from the JOIN message. Err was: {}", e),
                    _ => ()
                }
                let uname = String::from_utf8(uname_buf.clone()).unwrap();

                let mut state_guard = server_state.lock().unwrap();
                let mut dummy_user:User = User::new(String::from(""), Uuid::from_u128(0));
                let u:&User = {
                    match state_guard.user_map.get(&uuid) {
                        Some(u) => &u,
                        None => {
                            info!("adding user with name {} to server", uname);
                            dummy_user.uuid = Uuid::new_v4();
                            dummy_user.status = UserStatusType::ONLINE;
                            dummy_user.displayname = uname;
                            state_guard.add_user(&mut dummy_user);
                            &dummy_user
                        }
                    }
                };
                client_uuid = u.uuid;
                let userstatus = Message::UserStatus {
                    author: stream.clone(),
                    message_type: MessageType::USERSTATUS,
                    user_id: u.uuid,
                    status_type: UserStatusType::ONLINE,
                    name_len: u16::try_from(u.displayname.len()).expect("displayname.len() could not be cast to u16"),
                    desc_len: u16::try_from(u.description.len()).expect("disolayname.len() could not be cast to u16"),
                    username: u.displayname.as_bytes().to_vec(),
                    desc: u.description.as_bytes().to_vec(),
                };
                message.send(userstatus).map_err(|err|{
                    error!("couldn't send USERSTATUS message to MPSC sender. Err was {}",err);
                })?;
            }
            MessageType::LEAVE => {
                info!("recieved LEAVE message from {}", stream.peer_addr().unwrap());
                // uuid = 0 means we haven't joined yet
                if client_uuid != Uuid::from_u128(0) {
                    let mut state_guard = server_state.lock().unwrap();
                    let user_map_ref = &mut state_guard.user_map;
                    let mut u:&mut User = user_map_ref.get_mut(&client_uuid).unwrap();
                    u.status = UserStatusType::OFFLINE;
                    let userstatus = Message::UserStatus {
                        author: stream.clone(),
                        message_type: MessageType::USERSTATUS,
                        user_id: u.uuid, 
                        status_type: u.status,
                        name_len: u16::try_from(u.displayname.len()).expect("displayname.len() could not be cast to u16"),
                        desc_len: u16::try_from(u.description.len()).expect("disolayname.len() could not be cast to u16"),
                        username: u.displayname.as_bytes().to_vec(),
                        desc: u.description.as_bytes().to_vec(),
                    };
                    message.send(userstatus).map_err(|err|{
                        error!("Couldn't send USERSTATUS message following LEAVE messsage. Err was {}", err);
                    })?;
                }
            }
            MessageType::GETUSERS => {
                info!("received GETUSERS message from {}", stream.peer_addr().unwrap());
                let mut uuid_list: Vec<Uuid> = Vec::new();
                for (uuid, user) in server_state.lock().unwrap().user_map.iter(){
                    if user.status != UserStatusType::OFFLINE && user.status != UserStatusType::NOSUCHUSER {
                        uuid_list.push(user.uuid);
                    }
                }
                let userlist = Message::UserList {
                    author: stream.clone(),
                    message_type: MessageType::USERLIST,
                    uuid_list: uuid_list,
                };
                message.send(userlist).map_err(|err| {
                    error!("couldn't send USERLIST message to MPSC sender. Err was: {}", err);
                })?;
            }
            MessageType::GETUSERSTATUS => {
                info!("recieved GETUSERSTATUS message from {}", stream.peer_addr().unwrap());
                let mut uuid_buf = [0u8;16];
                match reader.read_exact(&mut uuid_buf[..]) {
                    Err(e) => error!("could not read 16 bytes for UUID from GETUSERSTATUS message with Err: {}", e),
                    _ => (),
                }
                let uuid = Uuid::from_bytes_le(uuid_buf);

                // Get reference to User; read values of User to populate message
                // Dummy user fills in values for NOSUCHUSER responses
                let mut dummy_user:User = User::new(String::from(""), Uuid::from_u128(0));
                dummy_user.status = UserStatusType::NOSUCHUSER;
                let state_guard = server_state.lock().unwrap();
                let u:&User = {
                    match state_guard.user_map.get(&uuid) {
                        Some(u) => &u,
                        None => &dummy_user,
                    }
                };
                let userstatus = Message::UserStatus {
                    author: stream.clone(),
                    message_type: MessageType::USERSTATUS,
                    user_id: uuid,
                    status_type: u.status,
                    name_len: u16::try_from(u.displayname.len()).expect("displayname.len() could not be cast to u16"),
                    desc_len: u16::try_from(u.description.len()).expect("disolayname.len() could not be cast to u16"),
                    username: u.displayname.as_bytes().to_vec(),
                    desc: u.description.as_bytes().to_vec(),
                };
                message.send(userstatus).map_err(|err|{
                    error!("couldn't send USERSTATUS message to MPSC sender. Err was: {}", err);
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

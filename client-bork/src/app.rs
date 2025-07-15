use rand::Rng;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    time::Duration,
};
use crate::event::{AppEvent, Event, EventHandler};
use log::{error, info};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use socket2::{Socket, Domain, Type};
use uuid::Uuid;

use::common_bork::{MessageType, UserStatusType};


const SERVER_PORT: u16 = 6556;
//const SERVER_ADDRESS:&'static str = "164.90.146.27";
const SERVER_ADDRESS: &'static str = "0.0.0.0";


// TODO: should be part of common?
#[derive(Clone, Debug)]
pub struct User{
    pub description:String,
    pub displayname:String,
    pub status:     u8,
    pub uuid:       Uuid,
}
impl User{
    fn new(displayname: String, uuid:Uuid) -> User {
        User{
            description: String::from("A nondescript llama"),
            displayname,
            status: UserStatusType::OFFLINE,
            uuid,
        }
    }
    fn set_displayname(&mut self, displayname: String){
        self.displayname = displayname;
    }
    fn set_description(&mut self, description: String){
        self.description = description;
    }
    fn set_status(&mut self, status: u8){
        self.status = status;
        // todo: validity check?
    }
}

/// Application.
#[derive(Debug)]
pub struct App {
    // TODO: we can probably re-use the User struct from the server
    pub active_users:       BTreeMap<Uuid, User>,
    pub connected:          bool,
    pub events:             EventHandler,
    pub inbuffer:           Vec<u8>,   //TODO: should be a list of rows to use as message buffer
    pub joined:             bool,       // TODO: probably want a modal object (e.g. online,dnd, etc)
    pub running:            bool,
    pub server_port:        u16,
    pub server_address:     String,
    pub server_major_ver:   u16,
    pub server_minor_ver:   u16,
    pub server_subminor_ver:u16,
    pub tcpstream:          TcpStream,
    pub username:           String,
    pub user_uuid:          Uuid,
    pub uuid_update_pending:Vec<Uuid>, // Seems hacky
}

impl Default for App {
    fn default() -> Self {
        Self {
            active_users: BTreeMap::new(),
            connected: false,
            events: EventHandler::new(),
            inbuffer: Vec::new(),
            joined: false,
            running: true,
            server_port: 0,
            server_address: String::new(),
            server_major_ver: 0,
            server_minor_ver: 0,
            server_subminor_ver: 0,
            tcpstream: TcpStream::from(Socket::new(Domain::IPV4, Type::STREAM, None).unwrap()),
            username: String::new(),
            user_uuid: Uuid::new_v4(),
            uuid_update_pending: Vec::new(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::ConnectServer => self.connect_to_server(SERVER_ADDRESS, SERVER_PORT),
                    AppEvent::DisconnectServer => self.disconnect_server(),
                    AppEvent::GetUsers => self.get_users(),
                    AppEvent::JoinUser => self.join_user(),
                    AppEvent::Quit => self.quit(),
                    AppEvent::UpdateUsers => self.update_user_statuses(),
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Char('c' | 'C') => self.events.send(AppEvent::ConnectServer),
            KeyCode::Char('d' | 'D') => self.events.send(AppEvent::DisconnectServer),
            KeyCode::Char('j' | 'J') => self.events.send(AppEvent::JoinUser),
            KeyCode::Char('u' | 'U') => self.events.send(AppEvent::UpdateUsers), //fixme: testing
            KeyCode::Char('g' | 'G') => self.events.send(AppEvent::GetUsers),  //FIXME: just
            //testing here; this shouldn't be bound to a key
            _ => {}
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        if self.connected {
            let mut pbuf = [0u8];
            let peeklen = match self.tcpstream.peek(&mut pbuf) {
                Err(_) => {
                    0
                },
                Ok(v) => v,
            };

            if peeklen > 0 {
                self.read_incomming();
            }
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn connect_to_server(&mut self, ip: &str, port: u16) {
        self.server_address = ip.into();
        self.server_port = port;
        let address = format!("{}:{}", ip, port);
        let stream = TcpStream::connect(&address).map_err(|err|{
            error!("Couldn't connect. Error was: {}", err);
        });
        self.tcpstream = stream.unwrap();
        let one_hundred_millis = Some(Duration::from_millis(1));
        self.tcpstream.set_read_timeout(one_hundred_millis)
            .expect("set_read_timeout call failed");
        self.connected = true;

        info!("Connected to server {}:{}", ip, port);
    }

    pub fn disconnect_server(&mut self) {
        match self.tcpstream.shutdown(Shutdown::Both) {
            Err(e) => error!("failed to shutdown TCPStream, with Err: {}", e),
            _ => ()
        };
        self.server_major_ver = 0;
        self.server_minor_ver = 0;
        self.server_subminor_ver = 0;
        self.server_address = String::from("");
        self.connected = false;
        info!("disconnected");
    }

    // create a fake username with random number (Guest1234) for now
    pub fn join_user(&mut self) {
        if !self.joined{
            let mut message: Vec<u8> = Vec::new();
            let mut rng = rand::rng();
            let fakeuser = format!("Guest{}", rng.random_range(1..=10000));
            let uname_len:u16 = u16::try_from(fakeuser.chars().count()).unwrap();
            message.push(MessageType::JOIN);
            message.extend(uname_len.to_le_bytes());
            message.extend(fakeuser.as_bytes());

            self.tcpstream.write_all(&message).map_err(|err| {
                error!("Could not send Join message to server. Err: {}", err);
            }).ok();
            self.tcpstream.flush().ok();
            // TODO: Add an 'ack' type message?
            self.joined = true;
        }
    }

    pub fn get_users(&mut self){
        if self.connected {
            info!("sending GETUSERS message");
            let mut message: Vec<u8> = Vec::new();
            message.push(MessageType::GETUSERS);
            self.tcpstream.write_all(&message).map_err(|err| {
               error!("Could not send GETUSERS message to server. Err: {}", err);
            }).ok();
            self.tcpstream.flush().ok();
        }
        else {
            info!("tried to send GETUSERS message, but the client is not connected to the server");
        }
    }

    // TODO: periodically push UUID's back into uuid_update_pending
    // to poll for updates?
    pub fn update_user_statuses(&mut self){
        info!("triggerred update_user_statuses");
        for u in self.uuid_update_pending.iter() {
            let mut message: Vec<u8> = Vec::new();
            message.push(MessageType::GETUSERSTATUS);
            message.extend(u.to_bytes_le());
            self.tcpstream.write_all(&message).map_err(|err| {
                error!("could not send GETUSERSTATUS message to server. Err: {}", err);
            }).ok();
            self.tcpstream.flush().ok();
            info!("Updating for user: {}", u);
        }
    }

    pub fn read_incomming(&mut self){
        // read message type; handle one message based on that type
        // first byte is the message type per protocol -- read it, handle based on type
        let mut mtype = [0u8];
        match self.tcpstream.read_exact(&mut mtype) {
            Err(e) => error!("failed to read message type, with Err: {}", e),
            _ => ()
        }
        info!("received message type {}", mtype[0]);
        match mtype[0] {
            MessageType::VERSION => {
                let mut major = [0u8;2];
                let mut minor = [0u8;2];
                let mut subminor = [0u8;2];
                match self.tcpstream.read_exact(&mut major) {
                    Err(e) => error!("Failed to read major version with Err: {}", e),
                    _ => ()
                }
                match self.tcpstream.read_exact(&mut minor) {
                    Err(e) => error!("Failed to read minor version with Err: {}", e),
                    _ => ()
                }

                match self.tcpstream.read_exact(&mut subminor) {
                    Err(e) => error!("Failed to read subminor version with Err: {}", e),
                    _ => ()
                }

                self.server_major_ver = u16::from_le_bytes(major);
                self.server_minor_ver = u16::from_le_bytes(minor);
                self.server_subminor_ver = u16::from_le_bytes(subminor);
            }
            MessageType::WELCOME => {
                info!("Received WELCOME message");
                // this message type has variable length, so, we determine that length
                // and read that many bytes
                let mut len = [0u8;2];
                match self.tcpstream.read_exact(&mut len){
                    Err(e) => error!("failed to read Welcome message length with Err: {}", e),
                    _ => ()
                }
                let len:u16 = u16::from_le_bytes(len);
                let mut wm_buf = vec![0; len as usize];
                match self.tcpstream.read_exact(&mut wm_buf){
                    Err(e) => error!("failed to read Welcome message content with Err: {}", e),
                    _ => ()
                }
                self.inbuffer.extend_from_slice(&wm_buf[0..]);
            }
            MessageType::USERJOINED => {
                info!("received USERJOINED message");
                // read uuid
                let mut user_uuid = [0u8;16];
                match self.tcpstream.read_exact(&mut user_uuid[..]){
                    Err(e) => error!("Failed to read UUID from USERJOINED message with Err: {}", e),
                    _ => ()
                }
                let user_uuid = Uuid::from_bytes_le(user_uuid);
                // read username
                let mut namelen = [0u8;2];
                match self.tcpstream.read_exact(&mut namelen[..]) {
                    Err(e) => error!("Failed to read username length from USERJOINED message with Err: {}", e),
                    _ => ()
                }
                let mut uname_bytes = vec![0u8; u16::from_le_bytes(namelen) as usize];
                match self.tcpstream.read_exact(&mut uname_bytes[..]) {
                    Err(e) => error!("Failed to read uname_bytes bytes from USERJOINED message with Err: {}", e),
                    _ => ()
                }
                let username = String::from_utf8(uname_bytes).expect("Could not complete UTF-8 conversion from uname_bytes to String");
                self.active_users.insert(user_uuid.clone(), User::new(username.clone(), user_uuid.clone()));
                info!("current active users: {:?}", self.active_users.keys());
            }
            MessageType::USERLIST => {
                info!("Received USERLIST message");
                let mut num_users = [0u8;2];
                match self.tcpstream.read_exact(&mut num_users[..]) {
                    Err(e) => error!("Failed to read num_users from USERLIST message with Err: {}", e),
                    _ => ()
                }
                let num_users = u16::from_le_bytes(num_users);
                let mut uuid_buff = [0u8;16]; // consume uuid's in 16-byte chunks
                for i in 0..num_users {
                    match self.tcpstream.read_exact(&mut uuid_buff[..]){
                        Err(e) => error!("Could not read 16 bytes for UUID from USERLIST message, on item number {} with Err: {}", i, e),
                        _ => ()
                    };
                    self.uuid_update_pending.push(Uuid::from_bytes_le(uuid_buff));
                }
            }
            MessageType::USERSTATUS => {
                info!("Received USERSTATUS message");
                // uuid
                let mut uuid_buf = [0u8;16];
                match self.tcpstream.read_exact(&mut uuid_buf[..]) {
                    Err(e) => error!("Failed to read UUID bytes from USERSTATATUS message, with Err: {}", e),
                    _ => ()
                }
                let user_uuid = Uuid::from_bytes_le(uuid_buf);

                // status code
                let mut status_byte = [0u8;1];
                match self.tcpstream.read_exact(&mut status_byte[..]) {
                    Err(e) => error!("could not read status byte from USERSTATUS message with Err: {}", e),
                    _ => ()
                };
                let status_code = u8::from_le_bytes(status_byte);

                // display name length
                let mut namelen_buf = [0u8;2];
                match self.tcpstream.read_exact(&mut namelen_buf[..]) {
                    Err(e) => error!("Could not read name length from USERSTATUS message with Err: {}", e),
                    _ => ()
                };
                let namelen = u16::from_le_bytes(namelen_buf);

                // description length
                let mut desclen_buf = [0u8;2];
                match self.tcpstream.read_exact(&mut desclen_buf[..]) {
                    Err(e) => error!("Could not read description length from USERSTATUS message with Err: {}", e),
                    _ => ()
                };
                let desclen = u16::from_le_bytes(desclen_buf);

                // displayname
                let mut uname_bytes = vec![0u8; namelen as usize];
                match self.tcpstream.read_exact(&mut uname_bytes[..]){
                    Err(e) => error!("Could not read username bytes from USERSTATUS message with Err: {}", e),
                    _ => ()
                }
                let username = String::from_utf8(uname_bytes).expect("Could not complete UTF-8 conversion from uname_bytes to String");

                // description
                let mut description = String::new();
                if desclen > 0 {
                    let mut desc_bytes = vec![0u8; desclen as usize];
                    match self.tcpstream.read_exact(&mut desc_bytes[..]) {
                        Err(e) => error!("Could not read user description from USERSTATUS message, with Err: {}", e),
                        _ => ()
                    };
                    description = String::from_utf8(desc_bytes).expect("Could not complete UTF-8 conversion from desc_bytes to String");
                }

                // if this user already exists on the server, update it
                // otherwise, ignore this message (we still need to remove the bytes, above)
                // TOOD: Maybe it makes more sense to deprecate the USERJOINED message
                // and collapse into a USERSTATUS? IdK.
                match self.active_users.get_mut(&user_uuid){
                    Some(u) => {
                        u.set_status(status_code);
                        u.set_displayname(username);
                        if desclen > 0 { u.set_description(description); }
                    }
                    None => ()
                };
            }
            _ => {
                info!("Received unknown message type with ID: {}", mtype[0]);
            }
        }
    }
}

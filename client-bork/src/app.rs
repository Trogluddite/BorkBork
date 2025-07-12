use rand::{Rng, SeedableRng};
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

use::common_bork::MessageType;


const SERVER_PORT: u16 = 6556;
//const SERVER_ADDRESS:&'static str = "164.90.146.27";
const SERVER_ADDRESS: &'static str = "0.0.0.0";

/// Application.
#[derive(Debug)]
pub struct App {
    // TODO: we can probably re-use the User struct from the server
    pub active_users:       BTreeMap<String, Uuid>,
    pub connected:          bool,
    pub events:             EventHandler,
    pub inbuffer:           Vec<u8>,   //TODO: should be a list of rows to use as message buffer
    pub running:            bool,
    pub server_port:        u16,
    pub server_address:     String,
    pub server_major_ver:   u16,
    pub server_minor_ver:   u16,
    pub server_subminor_ver:u16,
    pub tcpstream:          TcpStream,
    pub username:           String,
    pub user_uuid:          Uuid,
}

impl Default for App {
    fn default() -> Self {
        Self {
            active_users: BTreeMap::new(),
            connected: false,
            events: EventHandler::new(),
            inbuffer: Vec::new(),
            running: true,
            server_port: 0,
            server_address: String::new(),
            server_major_ver: 0,
            server_minor_ver: 0,
            server_subminor_ver: 0,
            tcpstream: TcpStream::from(Socket::new(Domain::IPV4, Type::STREAM, None).unwrap()),
            username: String::new(),
            user_uuid: Uuid::new_v4(),
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
                    AppEvent::JoinUser => self.join_user(),
                    AppEvent::Quit => self.quit(),
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
        let mut message: Vec<u8> = Vec::new();
        let mut rng = rand::rng();
        let fakeuser = format!("Guest{}", rng.random_range(1..=1000));
        let uname_len:u16 = u16::try_from(fakeuser.chars().count()).unwrap();
        message.push(MessageType::JOIN);
        message.extend(uname_len.to_le_bytes());
        message.extend(fakeuser.as_bytes());

        self.tcpstream.write_all(&message).map_err(|err| {
            error!("Could not send Join message to server. Err: {}", err);
        }).ok();
        self.tcpstream.flush().ok();
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
                let mut user_uuid = [0u8;16];
                let mut namelen = [0u8;2];

                match self.tcpstream.read_exact(&mut user_uuid[..]){
                    Err(e) => error!("Failed to read UUID from USERJOINED message with Err: {}", e),
                    _ => ()
                }
                match self.tcpstream.read_exact(&mut namelen[..]) {
                    Err(e) => error!("Failed to read username length from USERJOINED message with Err: {}", e),
                    _ => ()
                }
                let username = String::from_utf8(vec![0u8; u16::from_le_bytes(namelen) as usize]).unwrap();
                let user_uuid = Uuid::from_u128(u128::from_le_bytes(user_uuid));
                self.active_users.insert(username, Uuid::from(user_uuid));
            }
            _ => ()
        }
    }
}

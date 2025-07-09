use std::{io::Read, net::{Shutdown, TcpStream}};

use crate::event::{AppEvent, Event, EventHandler};
use log::{error, info};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use socket2::{Socket, Domain, Type};
//use tokio::net::TcpStream;


const SERVER_PORT: u16 = 6556;
const SERVER_ADDRESS:&'static str = "164.90.146.27";
//const SERVER_ADDRESS: &'static str = "0.0.0.0";

/// Application.
#[derive(Debug)]
pub struct App {
    pub running:            bool,
    pub connected:          bool,
    pub events:             EventHandler,
    pub server_port:        u16,
    pub server_address:     String,
    pub server_major_ver:   u8,
    pub server_minor_ver:   u8,
    pub server_subminor_ver:u8,
    pub tcpstream:          TcpStream,
    pub inbuffer:           Vec<u8>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            connected: false,
            events: EventHandler::new(),
            server_port: 0,
            server_address: String::new(),
            server_major_ver: 0,
            server_minor_ver: 0,
            server_subminor_ver: 0,
            tcpstream: TcpStream::from(Socket::new(Domain::IPV4, Type::STREAM, None).unwrap()),
            inbuffer: Vec::new(),
        }
    }
}

impl App {
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
            _ => {}
        }
        Ok(())
    }

    // tick events should run once per frame; useful for things like polling
    pub fn tick(&mut self) {
        if self.connected { self.read_incomming(); }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    // set up the TcpStream
    pub fn connect_to_server(&mut self, ip: &str, port: u16) {
        self.server_address = ip.into();
        self.server_port = port;
        let address = format!("{}:{}", ip, port);
        let stream = TcpStream::connect(&address).map_err(|err|{
            error!("Couldn't connect. Error was: {}", err);
        });
        self.tcpstream = stream.unwrap();
        self.connected = true;

        info!("Connected to server {}:{}", ip, port);
    }

    // shut down the TcpStream
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

    // poll for incomming messages from the server
    pub fn read_incomming(&mut self){
        // read all currently available messages
        let mut pbuf = [0u8];
        let mut peeklen = self.tcpstream.peek(&mut pbuf).expect("peek failed");
        while peeklen > 0 {
            // first byte is the message type per protocol -- read it, handle based on type
            let mut mtype = [0u8];
            match self.tcpstream.read_exact(&mut mtype) {
                Err(e) => error!("failed to read message type, with Err: {}", e),
                _ => ()
            }
            match mtype[0] {
                3 => {
                //MessageType::VERSION => {
                    let mut major = [0u8];
                    let mut minor = [0u8];
                    let mut subminor = [0u8];
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
                    self.server_major_ver = u8::from_le_bytes(major);
                    self.server_minor_ver = u8::from_le_bytes(minor);
                    self.server_subminor_ver = u8::from_le_bytes(subminor);
                }
                4 => {
                //MessageType::WELCOME => {
                    // this message type has variable length, so, we determine that length
                    // and read that many bytes
                    let mut len = [0u8, 0u8];
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
                _ => ()
            }
            // are there any more bytes to process?
            peeklen = self.tcpstream.peek(&mut pbuf).expect("peek failed");
        }
    }
}

use log::{info, error};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, self};
use std::io::Read;
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::AtomicBool;
use socket2::{Socket, Domain, Type};

use::common_bork::MessageType;

use crate::{
    events::{Event, Events},
    tui::Tui,
};

#[allow(unused)] const SUBMINOR_VER:u8 = 1;
#[allow(unused)] const MINOR_VER:u8 = 0;
#[allow(unused)] const MAJOR_VER:u8 = 0;

pub enum CurrentScreen{
    Main,
}

pub enum Mode{
    RecieveScroll,
    Send,
    UserScroll,
    Config,
}

pub struct App{
    pub current_screen:     CurrentScreen,
    pub server_address:     String,
    pub server_port:        u16,
    pub server_connected:   bool,
    pub server_subminor_ver:u8,
    pub server_minor_ver:   u8,
    pub server_major_ver:   u8,
    pub tcpstream:          TcpStream,
    pub inbuffer:           Vec<u8>,
    pub rx: UnboundedReceiver<Action>,
    pub tx: UnboundedSender<Action>,
    pub loading_status: Arc<AtomicBoo>,
    pub mode: Mode,
    pub last_mode: Mode,
    pub last_tick_key_events: Vec<KeyEvent>,
    pub frame_count: usize,
}

#[allow(dead_code)]
impl App{
    pub fn new() -> App {
        let (tx, rx): mpsc::unbounded_channel();
        App {
            current_screen: CurrentScreen::Main,
            server_address: String::new(),
            server_port:    0,
            server_connected: false,
            server_subminor_ver: 0,
            server_minor_ver: 0,
            server_major_ver: 0,
            tcpstream: TcpStream::from(Socket::new(Domain::IPV4, Type::STREAM, None).unwrap()),
            inbuffer: Vec::new(),
            tx,
            rx,
            loading_status: Arc::new(tx.clone, loading_status.clone()),
            mode: Mode::Send,
            last_mode: Mode::Config,
            last_tick_key_events: Vec::new(),
            frame_count: 0,
        }
    }

    pub async fn run(&mut self, mut tui: Tui, mut events: Events) -> Result<()> {
        let mut tui = tui::Tui::new()?
            .tick_rate(4.0)
            .frame_rate(60.0);
        tui.enter()?;
        loop{
            tui.draw(|f| {
                self.ui(f);
            })?;

            if let Some(evt) = tui.next().await { // block until an event is received
                let mut maybe_action = self.handle_event(evt);
                while let Some(action) = maybe_action {
                    maybe_action = self.update(action);
                }
            };

            if self.should_quit() {
                break;
            }

            tui.exit()?;
            Ok(())
        }
    }

    pub fn handle_event(&mut self, e: Event) -> Result<Option<Action>> {}
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {}
    pub fn handle_action(&mut self, action: Action) -> Result<()> { Ok(()) }
    pub fn draw(&mut self, tui: &mut Tui) -> Result<()> { Ok(()) }


    pub fn connect_server(&mut self, ip: &str, port: u16){
        self.server_address = ip.into();
        self.server_port = port;
        let address = format!("{}:{}", self.server_address, self.server_port);
        let stream = TcpStream::connect(&address).map_err(|err|{
            error!("Couldn't connect, error was: {}", err);
        });
        self.tcpstream = stream.unwrap();
        self.server_connected = true;
        info!("Connected to server {}:{}", ip, port);
    }

    pub fn disconnect(&mut self){
        match self.tcpstream.shutdown(Shutdown::Both) {
            Err(e) => error!("failed to shutdown TCPStream, with Err: {}", e),
            _ => ()
        };
        self.server_major_ver = 0;
        self.server_minor_ver = 0;
        self.server_subminor_ver = 0;
        self.server_connected = false;
        info!("disconnected");
    }

    pub fn switch_screen(&mut self, target: CurrentScreen){
        self.current_screen = target;
    }

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
                MessageType::VERSION => {
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
                MessageType::WELCOME => {
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

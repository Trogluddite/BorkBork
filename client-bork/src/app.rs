use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use socket2::{Socket, Domain, Type};

use::common_bork::{Message, MessageType};

pub enum CurrentScreen{
    Main,
    Config,
}

// FIXME
#[allow(dead_code)]
pub struct App{
    pub current_screen:     CurrentScreen,
    pub server_address:     String,
    pub server_port:        u16,
    pub server_connected:   bool,
    pub tcpstream:          TcpStream,
    pub inbuffer:           Vec<u8>,
    pub outbuffer:          Vec<u8>,
}

#[allow(dead_code)]
impl App{
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            server_address: String::new(),
            server_port:    0,
            server_connected: false,
            tcpstream: TcpStream::from(Socket::new(Domain::IPV4, Type::STREAM, None).unwrap()),
            inbuffer: Vec::new(),
            outbuffer: Vec::new(),
        }
    }

    pub fn set_server(&mut self, ip: &str, port: u16){
        self.server_address = ip.into();
        self.server_port = port;
        let address = format!("{}:{}", self.server_address, self.server_port);
        let stream = TcpStream::connect(&address).map_err(|err|{
            eprintln!("Couldn't connect, error was: {}", err);
        });
        self.tcpstream = stream.unwrap();
        self.server_connected = true;
    }

    pub fn switch_screen(&mut self, target: CurrentScreen){
        self.current_screen = target;
    }

    pub fn read_incomming(&mut self){
        let mut bufr: Vec<u8> = Vec::new();

        match self.tcpstream.read_to_end(&mut bufr) {
            Err(e) => eprintln!("failed to read TcpStream into buffer, with Err: {}", e),
            _ => ()
        }
        if bufr.len() > 0 {
            // first byte is the message type per protocol
            match Vec::from_iter(bufr[0..1].iter().cloned())[0] {
                MessageType::WELCOME => {
                    self.inbuffer.extend_from_slice(&bufr[1..]);
                }
                _ => ()
            }
        }
    }
}

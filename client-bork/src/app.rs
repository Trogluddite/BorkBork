use std::net::TcpStream;

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
    pub tcpstream:          Option<TcpStream>,
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
            tcpstream: None,
            inbuffer: vec![],
            outbuffer: vec![],
        }
    }

    pub fn set_server(&mut self, ip: String, port: u16){
        self.server_address = ip;
        self.server_port = port;
        let address = format!("{}:{}", self.server_address, self.server_port);
        let stream = TcpStream::connect(&address);
        self.tcpstream = Some(stream.unwrap()); //TODO: handle failed stream setup
        self.server_connected = true;
    }

    pub fn switch_screen(&mut self, target: CurrentScreen){
        self.current_screen = target;
    }
}

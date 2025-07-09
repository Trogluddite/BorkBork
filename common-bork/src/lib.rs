use std::sync::Arc;
use std::net::TcpStream;

pub struct MessageType;
impl MessageType{
    pub const CHATMSG:      u8 = 0;
    pub const JOIN:         u8 = 1;
    pub const LEAVE:        u8 = 2;
    pub const VERSION:      u8 = 3;
    pub const WELCOME:      u8 = 4;
}

pub enum Message{
    ChatMsg{
        author:         Arc<TcpStream>,
        message_type:   u8,
        sender_id:      u128,
        message_len:    u16,
        message_text:   Vec<u8>,
    },
    Join{
        author:         Arc<TcpStream>,
        message_type:   u8,
        name_len:       u16,
        username:       Vec<u8>,
    },
    Leave{
        author:         Arc<TcpStream>,
        message_type:   u8,
    },
    Version{
        author:         Arc<TcpStream>,
        message_type:   u8,
        major_rev:      u16,
        minor_rev:      u16,
        subminor_rev:   u16,
    },
    Welcome{
        author:         Arc<TcpStream>,
        message_type:   u8,
        message_len:    u16,
        welcome_msg:    Vec<u8>,
    }
}

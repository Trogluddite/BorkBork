use std::sync::Arc;
use std::net::TcpStream;
use uuid::Uuid;

// Matches BorkBork protocol version 0.0.3
// https://github.com/Trogluddite/BorkBork/blob/main/protocol/network_protocol_specification.md
pub struct MessageType;
impl MessageType{
    pub const CHATMSG:      u8 = 0;
    pub const JOIN:         u8 = 1;
    pub const LEAVE:        u8 = 2;
    pub const VERSION:      u8 = 3;
    pub const WELCOME:      u8 = 4;
    pub const EXTENDED:     u8 = 5;
    pub const USERJOINED:   u8 = 6;
    pub const USERLEFT:     u8 = 7;
}

pub struct ExtendedMessageType;
impl ExtendedMessageType{
    pub const FUTURE: u64 = 0;
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
    },
    Extended{
        author:         Arc<TcpStream>,
        message_type:   u8,
        extended_type:  u64,
        content:        Vec<u8>, //future: we likely want type-specific controls for the extensions
    },
    Userjoined{
        author:         Arc<TcpStream>,
        message_type:   u8,
        user_id:        Uuid,
        username_len:   u16,
        username:       Vec<u8>,
    },
    Userleft{
        author:         Arc<TcpStream>,
        message_type:   u8,
        user_id:        Uuid,
    }
}

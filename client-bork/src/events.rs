use crossterm::event::{KeyEvent, MouseEvent};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Event {
    // Terminal events
    Closed,
    Error,
    FocusGained,
    FocusLost,
    Init,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Quit,
    Render,
    Resize(u16, u16),
    Tick,

    // Server events
    ServerData(Vec<u8>),
    ServerConnected,
    ServerDisconnected,
    ServerError(String),
}

#[derive(Debug)]
pub enum Action {
    Quit,
    ConnectServer,
    DisconnectServer,
    SwitchScreen,
    SendMessage(String),
    // Add more actions as needed
} 
use crate::event::{AppEvent, Event, EventHandler};
use log::info;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};


const SERVER_PORT: u16 = 6556;
//const SERVER_ADDRESS:&'static str = "164.90.146.27";
const SERVER_ADDRESS: &'static str = "0.0.0.0";

/// Application.
#[derive(Debug)]
pub struct App {
    pub running:        bool,
    pub connected:      bool,
    pub counter:        u8,
    pub events:         EventHandler,
    pub server_port:    u16,
    pub server_address: String,
    pub server_major_ver: u8,
    pub server_minor_ver: u8,
    pub server_subminor_ver: u8,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running:        true,
            connected:      false,
            counter:        0,
            events:         EventHandler::new(),
            server_port:    0,
            server_address: String::new(),
            server_major_ver: 0, //FIXME
            server_minor_ver: 0, //FIXME
            server_subminor_ver: 2 //FIXME
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
                    AppEvent::Increment => self.increment_counter(),
                    AppEvent::Decrement => self.decrement_counter(),
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
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            KeyCode::Char('c' | 'C') => self.events.send(AppEvent::ConnectServer),
            KeyCode::Char('d' | 'D') => self.events.send(AppEvent::DisconnectServer),
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }

    pub fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }

    pub fn connect_to_server(&mut self, ip: &str, port: u16) {
        self.server_address = ip.into();
        self.server_port = port;
        self.connected = true;
        info!("Connected to server");
    }

    pub fn disconnect_server(&mut self) {
        self.connected = false;
    }
}

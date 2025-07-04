/*** Derrived from guide at:
 *   https://ratatui.rs/recipes/apps/terminal-and-event-handler/
 ***/

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};
use color_eyre::eyre::Result;
use log::{info, error, LevelFilter};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use ratatui::crossterm::{
    cursor,
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Event {
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
}

pub struct Tui {
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub mouse: bool,
    pub paste: bool,
    pub task: JoinHandle<()>,
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
    pub tick_rate: f64,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let cancellation_token = CancellationToken::new();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let frame_rate = 60.0;
        let mouse = false;
        let paste = false;
        let task = tokio::spawn(async {});
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
        let tick_rate = 4.0;
    }
    /****< Setters>****/
    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }
    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self;
    }
    pub fn paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }
    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }
    /****</Setters>****/

    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);

        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancellation_token = CancellationToken::new();
        let _event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval   = tokio::time::interval(tick_delay);
            let mut reader_interval = tokio::time::interval(render_delay);
            _event_tx.send(Event::Init).unwrap();

            loop{
                let tick_delay   = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select!{
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::FocusGained => {
                                        _event_tx.send(Event::FocusGained).unwrap();
                                    },
                                    CrosstermEvent::FocusLost => {
                                        _event_tx.send(Event::FocusLost).unwrap();
                                    },
                                    CrosstermEvent::Key(key) => {
                                        if key.kind == KeyEventKind::Press {
                                            _event_tx.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    CrosstermEvent::Mouse(mouse) => {
                                        _event_tx.send(Event::Mouse(mouse)).unwrap();
                                    },
                                    CrosstermEvent::Paste(s) => {
                                        _event_tx.send(Event::Paste(s)).unwrap();
                                    },
                                    CrosstermEvent::Resize(x,y) => {
                                        _event_tx.send(Event::Resize(x,y)).unwrap();
                                    },
                                }
                            }
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap();
                            }
                            None => {},
                        }
                    },
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap();
                    },
                    _ = render_delay => {
                        _event_tx.send(Event::Render).unwrap();
                    },
                }
            }
        });
    }

    pub fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                error!("Failed to abort task after 100 milliseconds; reason unknown");
                break;
            }
        }
        Ok(())
    }

    pub fn enter(&self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse{
            crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;
        }
        if self.paste{
            crossterm::execute!(std::io::stderr(), EnableBracketedPaste)?;
        }
        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.paste {
                crossterm::execute!(std::io::stderr(), DisableBracketedPaste);
            }
            if self.mouse {
                crossterm::execute!(std::io::stderr(), DisableMouseCapture);
            }
            crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}

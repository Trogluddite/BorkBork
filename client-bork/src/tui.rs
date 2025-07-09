use std::{fmt::Result, io::{stdout, Stdout}};

use color_eyre::eyre::Result;
use crossterm::{event::*, execute, terminal::*};
use ratatui::prelude::*;


pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Tui> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    Ok(terminal);
}

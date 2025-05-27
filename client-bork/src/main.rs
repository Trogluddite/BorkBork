use std::{error::Error, result, thread, io};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app;
mod ui;

use crate::{
    app::{App, CurrentScreen},
    ui::ui,
};

fn main() -> Result<(), Box<dyn Error>> {
    /* set up the terminal */
    enable_raw_mode()?;
    let mut stderr = io::stderr();  //Since Terminal defaults stderr/stdout to the same stream
    execute!(stderr, EnterAlternateScreen);
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;
    /* end of terminal setup*/

    /* create & run app */
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    /* restore terminmal on exit */
    disable_raw_mode();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
    )?;
    terminal.show_cursor()?;
    if let Err(err) = res{
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release{
                // we're currently only interested in pressed keys
                continue;
            }
            match key.code{
                KeyCode::Char('m') => {
                    app.current_screen = CurrentScreen::Main;
                }
                KeyCode::Char('c') => {
                    app.current_screen = CurrentScreen::Config;
                }
                KeyCode::Char('q') => {
                    return Ok(true);
                }
                _ => {}
            }
        }
    }
}

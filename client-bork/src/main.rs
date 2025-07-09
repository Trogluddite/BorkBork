#[allow(unused_imports)]
use log::{info, error, LevelFilter};

mod app;
mod events;
mod tui;
mod ui;

use app::App;
use color_eyre::eyre::Result;

const SERVER_PORT: u16 = 6556;
//const SERVER_ADDRESS:&'static str = "164.90.146.27";
const SERVER_ADDRESS: &'static str = "0.0.0.0";

#[tokio::main]
async fn main() -> Result<()> {
    let _ = simple_logging::log_to_file("./client-bork.log", LevelFilter::Info);
    let tui = tui::init()?;
    let events = events::Events::new();
    App::new().run();
   
    Ok(())
}


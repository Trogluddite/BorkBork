#[allow(unused_imports)]
use crate::app::App;
#[allow(unused_imports)]
use log::{info, error, LevelFilter};

pub mod app;
pub mod event;
pub mod ui;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = simple_logging::log_to_file("./client.log", LevelFilter::Info);
    color_eyre::install()?;
    let terminal = ratatui::init();
    info!("Initialized terminal");
    let result = App::new().run(terminal).await;
    info!("Initialized app");
    ratatui::restore();
    info!("Terminal restored");
    result
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod core;
mod driver;
use tracing::info;

use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Start App");
    app::start_app()?;
    Ok(())
}

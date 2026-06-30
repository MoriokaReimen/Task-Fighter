#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tracing::info;
use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Start App");
    app::start_app()?;
    Ok(())
}

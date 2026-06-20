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


use crate::{Core, CoreOutput};
use anyhow::Result;
use domain::prelude::*;
use domain::Config;
use std::sync::Arc;
use tokio::sync::oneshot::{self};

impl ConfigRecord for Core {

    fn save_config(&self, config: &Config) -> Result<()> {
        self.runtime.block_on(async {
            let conn = self.conn.lock().await;
            driver::save_config(&conn, config.clone())
        })
    }

    fn load_config(&self) -> Result<Config> {
        self.runtime.block_on(async {
            let conn = self.conn.lock().await;
            driver::load_config(&conn)
        })
    }
}

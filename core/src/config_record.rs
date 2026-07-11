use crate::Core;
use anyhow::Result;
use domain::Config;
use domain::prelude::*;

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

use crate::config::Config;
use anyhow::Result;

pub trait ConfigRecord {
    fn save_config(&self, config: &Config) -> Result<()>;
    fn load_config(&self) -> Result<Config>;
}

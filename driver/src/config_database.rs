use crate::duckdb_config::DuckdbConfig;
use anyhow::Result;
use domain::Config;
use duckdb::{Connection, named_params};

pub fn save_config(conn: &Connection, config: Config) -> Result<()> {
    // Convert your clean domain config into the DB wrapper data layout
    let db_config = DuckdbConfig::from(config);

    // Bind parameters explicitly by name using the named_params! macro
    conn.execute(
        "INSERT OR REPLACE INTO config (id, color_scheme) VALUES (1, $color_scheme);",
        named_params! {
            "color_scheme": db_config,
        },
    )?;

    Ok(())
}

pub fn load_config(conn: &Connection) -> Result<Config> {
    let mut stmt = conn.prepare("SELECT color_scheme FROM config WHERE id = 1;")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let db_config: DuckdbConfig = row.get(0)?;
        let config = Config::try_from(db_config)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

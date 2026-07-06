use anyhow::{Context, Result, bail};
use duckdb::Connection;
use std::fs;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuckdbPath {
    InMemory,
    InDirectory(PathBuf),
}

pub fn connect(duckdb_path: &DuckdbPath) -> Result<Connection> {
    let conn = match duckdb_path {
        DuckdbPath::InMemory => {
            info!("Initializing DuckDB in-memory database.");
            Connection::open_in_memory()?
        }
        DuckdbPath::InDirectory(path) => {
            info!("Initializing File-based DuckDB database at: {:?}", path);
            if path.exists() && !path.is_dir() {
                bail!(format!("The file named {:?} exists", path));
            }
            fs::create_dir_all(path)?;
            Connection::open(path.join("task-fighter.db"))?
        }
    };
    const CREATE_TABLE_SQL: &str = include_str!("../assets/connect.sql");
    conn.execute(CREATE_TABLE_SQL, []).context(
        "Failed executing target master initialization schema table creation migrations",
    )?;
    info!("Database and target system schemas synchronized cleanly.");

    Ok(conn)
}

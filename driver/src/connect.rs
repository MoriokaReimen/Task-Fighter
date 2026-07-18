use crate::migrations;
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
            info!("Initializing File-based DuckDB database at: {path:?}");
            if path.exists() && !path.is_dir() {
                bail!(format!("The file named {} exists", path.display()));
            }
            fs::create_dir_all(path)?;
            Connection::open(path.join("task-fighter.db"))?
        }
    };

    migrations::migrate(&conn).context("Failed to migrate database schema")?;
    info!("Database connection established.");

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Tables ---

    fn exists_tasks_table(conn: &Connection) -> Result<bool> {
        info!("Checking if 'tasks' table exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM information_schema.tables 
                WHERE table_name = 'tasks'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_daily_tasks_table(conn: &Connection) -> Result<bool> {
        info!("Checking if 'daily_tasks' table exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM information_schema.tables 
                WHERE table_name = 'daily_tasks'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_weekly_tasks_table(conn: &Connection) -> Result<bool> {
        info!("Checking if 'weekly_tasks' table exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM information_schema.tables 
                WHERE table_name = 'weekly_tasks'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_monthly_tasks_table(conn: &Connection) -> Result<bool> {
        info!("Checking if 'monthly_tasks' table exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM information_schema.tables 
                WHERE table_name = 'monthly_tasks'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_relation_table(conn: &Connection) -> Result<bool> {
        info!("Checking if 'relation' table exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM information_schema.tables 
                WHERE table_name = 'relation'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_work_time_table(conn: &Connection) -> Result<bool> {
        info!("Checking if 'work_time' table exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM information_schema.tables 
                WHERE table_name = 'work_time'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    // --- Sequences ---

    fn exists_tasks_id_seq(conn: &Connection) -> Result<bool> {
        info!("Checking if 'tasks_id_seq' sequence exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM duckdb_sequences() 
                WHERE sequence_name = 'tasks_id_seq'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_daily_tasks_id_seq(conn: &Connection) -> Result<bool> {
        info!("Checking if 'daily_tasks_id_seq' sequence exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM duckdb_sequences() 
                WHERE sequence_name = 'daily_tasks_id_seq'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_weekly_tasks_id_seq(conn: &Connection) -> Result<bool> {
        info!("Checking if 'weekly_tasks_id_seq' sequence exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM duckdb_sequences() 
                WHERE sequence_name = 'weekly_tasks_id_seq'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_monthly_tasks_id_seq(conn: &Connection) -> Result<bool> {
        info!("Checking if 'monthly_tasks_id_seq' sequence exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM duckdb_sequences() 
                WHERE sequence_name = 'monthly_tasks_id_seq'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_seq_work_time_id(conn: &Connection) -> Result<bool> {
        info!("Checking if 'seq_work_time_id' sequence exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM duckdb_sequences() 
                WHERE sequence_name = 'seq_work_time_id'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    fn exists_schema_migrations_table(conn: &Connection) -> Result<bool> {
        info!("Checking if 'schema_migrations' table exists in the database.");
        let sql = "
            SELECT EXISTS (
                SELECT 1 
                FROM information_schema.tables 
                WHERE table_name = 'schema_migrations'
            );
        ";
        let exists: bool = conn.query_row(sql, [], |row| row.get(0))?;
        Ok(exists)
    }

    #[test]
    fn test_connect_in_memory() -> Result<()> {
        let path = DuckdbPath::InMemory;
        let conn = connect(&path);
        assert!(conn.is_ok(), "InMemory connection should succeed");
        let conn = conn.unwrap();

        // Tables Assertion
        assert!(
            exists_tasks_table(&conn)?,
            "'tasks' table should exist in database"
        );
        assert!(
            exists_daily_tasks_table(&conn)?,
            "'daily_tasks' table should exist in database"
        );
        assert!(
            exists_weekly_tasks_table(&conn)?,
            "'weekly_tasks' table should exist in database"
        );
        assert!(
            exists_monthly_tasks_table(&conn)?,
            "'monthly_tasks' table should exist in database"
        );
        assert!(
            exists_relation_table(&conn)?,
            "'relation' table should exist in database"
        );
        assert!(
            exists_work_time_table(&conn)?,
            "'work_time' table should exist in database"
        );
        assert!(
            exists_schema_migrations_table(&conn)?,
            "'schema_migrations' table should exist in database"
        );

        // Sequences Assertion
        assert!(
            exists_tasks_id_seq(&conn)?,
            "'tasks_id_seq' should exist in database"
        );
        assert!(
            exists_daily_tasks_id_seq(&conn)?,
            "'daily_tasks_id_seq' should exist in database"
        );
        assert!(
            exists_weekly_tasks_id_seq(&conn)?,
            "'weekly_tasks_id_seq' should exist in database"
        );
        assert!(
            exists_monthly_tasks_id_seq(&conn)?,
            "'monthly_tasks_id_seq' should exist in database"
        );
        assert!(
            exists_seq_work_time_id(&conn)?,
            "'seq_work_time_id' should exist in database"
        );

        Ok(())
    }

    #[test]
    fn test_connect_twice_is_idempotent() -> Result<()> {
        // Simulates an existing user re-opening an already-migrated database:
        // reconnecting must not fail or re-apply migrations.
        let dir = tempfile_dir()?;
        let path = DuckdbPath::InDirectory(dir.clone());

        connect(&path)?;
        let conn = connect(&path)?;

        let applied_count: u32 =
            conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        assert_eq!(applied_count as usize, migrations::MIGRATIONS.len());

        fs::remove_dir_all(dir)?;
        Ok(())
    }

    fn tempfile_dir() -> Result<PathBuf> {
        let mut dir = std::env::temp_dir();
        dir.push(format!("task-fighter-test-{}", uuid_like()));
        Ok(dir)
    }

    // Small dependency-free unique suffix so parallel test runs don't collide
    // on the same directory.
    fn uuid_like() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }
}

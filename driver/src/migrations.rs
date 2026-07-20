use anyhow::{Context, Result};
use duckdb::Connection;
use tracing::info;

pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
Migration {
    version: 1,
    name: "initial_schema",
    sql: include_str!("../assets/migrations/initial_schema.sql"),
},
Migration {
    version: 2,
    name: "second_schema",
    sql: include_str!("../assets/migrations/second_schema.sql"),
},
];

const CREATE_MIGRATIONS_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS schema_migrations (
        version     INTEGER PRIMARY KEY,
        name        VARCHAR NOT NULL,
        applied_at  TIMESTAMP NOT NULL DEFAULT current_timestamp
    );
";

/// Brings `conn`'s schema up to the latest version, applying any migration
/// from `MIGRATIONS` that hasn't been recorded as applied yet.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(CREATE_MIGRATIONS_TABLE_SQL)
        .context("Failed to create schema_migrations table")?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .context("Failed to read current schema version")?;

    let pending: Vec<&Migration> = MIGRATIONS
        .iter()
        .filter(|m| m.version > current_version)
        .collect();

    if pending.is_empty() {
        info!("Database schema up to date (version {current_version}).");
        return Ok(());
    }

    for migration in pending {
        info!(
            "Applying migration {}: {}",
            migration.version, migration.name
        );
        apply_migration(conn, migration).with_context(|| {
            format!(
                "Migration {} ({}) failed",
                migration.version, migration.name
            )
        })?;
    }

    Ok(())
}

/// Applies a single migration and records it, rolling back both the schema
/// change and the record together if either step fails.
fn apply_migration(conn: &Connection, migration: &Migration) -> Result<()> {
    conn.execute_batch("BEGIN TRANSACTION;")?;

    let result = (|| -> Result<()> {
        conn.execute_batch(migration.sql)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?, ?)",
            duckdb::params![migration.version, migration.name],
        )?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT;")?;
            Ok(())
        }
        Err(e) => {
            // Best-effort rollback; the original error is what we report.
            let _ = conn.execute_batch("ROLLBACK;");
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_fresh_database_applies_all_migrations() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        migrate(&conn)?;

        let applied_count: u32 =
            conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        assert_eq!(applied_count as usize, MIGRATIONS.len());

        let version: u32 =
            conn.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        assert_eq!(version, MIGRATIONS.last().unwrap().version);

        Ok(())
    }

    #[test]
    fn test_migrate_is_idempotent() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        migrate(&conn)?;
        // Running again should be a no-op, not re-apply or error (e.g. on
        // "table already exists").
        migrate(&conn)?;

        let applied_count: u32 =
            conn.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        assert_eq!(applied_count as usize, MIGRATIONS.len());

        Ok(())
    }
}

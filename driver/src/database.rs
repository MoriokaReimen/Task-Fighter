use crate::Task;
use crate::{Priority, TaskStatus};
use anyhow::{Context, Result, bail};
use duckdb::{Connection, params};
use jiff::Zoned;
use jiff::civil::Date;
use std::fs;
use std::path::Path;
use tracing::info;

/// Centralized mapper to convert a database row slice into a Task token instance,
/// significantly flattening nesting inside fetch functions.
impl<'a> TryFrom<&'a duckdb::Row<'a>> for Task {
    type Error = duckdb::Error;

    fn try_from(row: &'a duckdb::Row<'a>) -> Result<Self, Self::Error> {
        let status_raw: i32 = row.get(2)?;
        let priority_raw: i32 = row.get(8)?;

        let status = TaskStatus::try_from(status_raw).map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(2, duckdb::types::Type::Int, e.into())
        })?;
        let priority = Priority::try_from(priority_raw).map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(8, duckdb::types::Type::Int, e.into())
        })?;

        let start_date_str: String = row.get(6)?;
        let due_date_str: String = row.get(7)?;

        let start_date = start_date_str.parse::<Date>().map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(6, duckdb::types::Type::Text, e.into())
        })?;
        let due_date = due_date_str.parse::<Date>().map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(7, duckdb::types::Type::Text, e.into())
        })?;

        Ok(Task {
            id: row.get(0)?,
            active: row.get(1)?,
            status,
            project: row.get(3)?,
            title: row.get(4)?,
            detail: row.get(5)?,
            start_date,
            due_date,
            priority,
            progress: row.get(9)?,
            time_spent: row.get(10)?,
        })
    }
}

pub fn connect() -> Result<Connection> {
    let path = Path::new("runtime");
    if path.exists() && !path.is_dir() {
        bail!("'runtime' exists but is a file. Expected a directory context path target.");
    }
    fs::create_dir_all(path).context("Failed to safely initialize target 'runtime' directory")?;

    let conn = Connection::open("./runtime/task_fighter.db")
        .context("Failed to establish DuckDB database file handle stream connection")?;

    // 💡 1. 自動インクリメント用のシーケンス（SEQUENCE）を作成
    conn.execute("CREATE SEQUENCE IF NOT EXISTS tasks_id_seq START 1;", [])
        .context("Failed to create sequence for tasks id")?;

    // 💡 2. テーブル作成時に DEFAULT nextval('tasks_id_seq') を指定
    const CREATE_TABLE_SQL: &str = include_str!("../assets/create_table.sql");
    conn.execute(CREATE_TABLE_SQL, []).context(
        "Failed executing target master initialization schema table creation migrations",
    )?;
    info!("Database and target system schemas synchronized cleanly.");

    Ok(conn)
}

pub fn insert_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Inserting task record token: {:?}", task);

    const INSERT_TASK_SQL: &str = include_str!("../assets/insert_task.sql");
    let entry_date = Zoned::now().date();

    conn.execute(
        INSERT_TASK_SQL,
        params![
            task.active,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date.to_string(),
            task.due_date.to_string(),
            task.priority as i32,
            task.progress,
            task.time_spent,
            entry_date.to_string()
        ],
    )
    .context("Failed to commit novel dataset item to relational datastore row bounds")?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT 1 FROM tasks WHERE id = ?;")?;
    let exists = stmt
        .exists(duckdb::params![id])
        .context("Failed to check if task ID exists")?;

    Ok(exists)
}

pub fn upsert_task(conn: &Connection, task: &Task) -> Result<()> {
    let exists = exists_id(conn, task.id)?;
    if exists {
        info!("ID {} exists. Update task.", task.id);
        update_task(conn, task)?;
    } else {
        info!("ID {} not exists. Create new task.", task.id);
        insert_task(conn, task)?;
    }

    Ok(())
}

pub fn get_next_id(conn: &Connection) -> Result<i32> {
    let query = "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'tasks_id_seq';";
    let last_value: Option<i64> = conn
        .query_row(query, [], |row| row.get(0))
        .context("Failed to query next sequence value from DuckDB catalogs")?;
    let next_id = last_value.map(|val| (val + 1) as i32).unwrap_or(1);
    info!("Next task id is {}.", next_id);

    Ok(next_id)
}

pub fn fetch_active_tasks(conn: &Connection) -> Result<Vec<Task>> {
    info!("Querying active tasks");
    const FETCH_ACTIVE_TASK: &str = include_str!("../assets/fetch_active_task.sql");
    let mut stmt = conn
        .prepare(FETCH_ACTIVE_TASK)
        .context("Failed compiling relational parameter statements validations queries")?;

    let tasks = stmt
        .query_map([], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;
    info!("{} tasks queried.", tasks.len());

    Ok(tasks)
}

pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    const UPDATE_TASK_SQL: &str = include_str!("../assets/update_task.sql");
    let mut stmt = conn.prepare(UPDATE_TASK_SQL).context(
        "Failed compiling database structural mutations modification pipelines statements",
    )?;

    let rows_affected = stmt
        .execute(params![
            task.active,
            task.status as i32,
            task.project,
            task.title,
            task.detail,
            task.start_date.to_string(),
            task.due_date.to_string(),
            task.priority as i32,
            task.progress,
            task.time_spent,
            task.id
        ])
        .context("Failed executing datastore entity mutations pipelines updates states")?;

    if rows_affected == 0 {
        bail!(
            "Target modification bounds target identity token is non-existent: Identity [{}]",
            task.id
        );
    }

    if task.status == TaskStatus::Complete || task.status == TaskStatus::Canceled {
        let end_date = Zoned::now().date();
        let mut stmt = conn
            .prepare(
                "UPDATE tasks 
            SET end_date = ?2
            WHERE id = ?1",
            )
            .context(
                "Failed compiling database structural mutations modification pipelines statements",
            )?;

        let rows_affected = stmt
            .execute(params![task.id, end_date.to_string()])
            .context("Failed executing datastore entity mutations pipelines updates states")?;

        if rows_affected == 0 {
            bail!(
                "Target modification bounds target identity token is non-existent: Identity [{}]",
                task.id
            );
        }
    }

    info!("Task {} update success.", task.id);
    Ok(())
}

pub fn count_tasks_by_date(
    conn: &Connection,
    start_date: Date,
    end_date: Date,
) -> Result<Vec<(i32, i32, i32, i32)>> {
    const COUNT_TASK_BY_DATE_SQL: &str = include_str!("../assets/count_task_by_date.sql");
    let mut stmt = conn.prepare(COUNT_TASK_BY_DATE_SQL)?;
    let rows = stmt.query_map(
        params![start_date.to_string(), end_date.to_string()],
        |row| {
            Ok((
                row.get::<_, Option<i32>>(0)?.unwrap_or(0), // LEFT JOINでタスクがない日はNullになるため対策
                row.get::<_, Option<i32>>(1)?.unwrap_or(0),
                row.get::<_, Option<i32>>(2)?.unwrap_or(0),
                row.get::<_, Option<i32>>(3)?.unwrap_or(0),
            ))
        },
    )?;

    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn scan_tasks(conn: &Connection, pattern: &str, _only_active: bool) -> Result<Vec<Task>> {
    info!("Scanning tasks with regex pattern: {}", pattern);

    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    const SCAN_TASK_SQL: &str = include_str!("../assets/scan_task.sql");
    let mut stmt = conn
        .prepare(SCAN_TASK_SQL)
        .context("Failed to prepare regex match database query statement")?;
    let tasks = stmt
        .query_map([trimmed], |row| Task::try_from(row))?
        .collect::<Result<Vec<Task>, _>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;
    info!("{} tasks queried.", tasks.len());
    Ok(tasks)
}

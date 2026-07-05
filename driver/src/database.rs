use crate::Task;
use crate::TaskStatus;
use crate::duckdb_task::DuckdbTask;
use anyhow::{Context, Result, bail};
use duckdb::{Connection, params};
use jiff::Zoned;
use jiff::civil::Date;
use std::fs;
use std::path::Path;
use tracing::info;

pub fn connect() -> Result<Connection> {
    let path = Path::new("runtime");
    if path.exists() && !path.is_dir() {
        bail!("'runtime' exists but is a file. Expected a directory context path target.");
    }
    fs::create_dir_all(path).context("Failed to safely initialize target 'runtime' directory")?;

    let conn = Connection::open("./runtime/task_fighter.db")
        .context("Failed to establish DuckDB database file handle stream connection")?;
    const CREATE_TABLE_SQL: &str = include_str!("../assets/task_sql/connect.sql");
    conn.execute(CREATE_TABLE_SQL, []).context(
        "Failed executing target master initialization schema table creation migrations",
    )?;
    info!("Database and target system schemas synchronized cleanly.");

    Ok(conn)
}

pub fn insert_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Inserting task: {:?}", task);
    const INSERT_TASK_SQL: &str = include_str!("../assets/task_sql/insert_task.sql");
    let db_task: DuckdbTask = task.clone().into();
    let params = db_task.to_named_params();
    let mut stmt = conn.prepare(INSERT_TASK_SQL)?;
    stmt.execute(&params)?;

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

    let duckdb_tasks = stmt
        .query_map([], |row| DuckdbTask::try_from(row))?
        .collect::<Result<Vec<DuckdbTask>, duckdb::Error>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;
    let tasks = duckdb_tasks
        .into_iter()
        .map(Task::try_from)
        .collect::<Result<Vec<Task>>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;
    info!("{} tasks queried.", tasks.len());

    Ok(tasks)
}

pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Inserting task: {:?}", task);
    const UPDATE_TASK_SQL: &str = include_str!("../assets/task_sql/update_task.sql");
    let mut db_task: DuckdbTask = task.clone().into();
    db_task.end_date = if task.status == TaskStatus::Complete || task.status == TaskStatus::Canceled
    {
        Some(Zoned::now().date().to_string())
    } else {
        None
    };

    let params = db_task.to_named_params();
    let mut stmt = conn.prepare(UPDATE_TASK_SQL)?;
    stmt.execute(&params)?;

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
    let duckdb_tasks = stmt
        .query_map([trimmed], |row| DuckdbTask::try_from(row))?
        .collect::<Result<Vec<DuckdbTask>, duckdb::Error>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;
    let tasks = duckdb_tasks
        .into_iter()
        .map(Task::try_from)
        .collect::<Result<Vec<Task>>>()
        .context("Failed parsing query sequences lists mapping constraints rows")?;
    info!("{} tasks queried.", tasks.len());
    Ok(tasks)
}

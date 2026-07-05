use crate::duckdb_task::DuckdbTask;
use crate::task::{Task, TaskStatus};
use crate::task::{TaskFilterFlags, TaskOrderFlags, TaskSearchFlags};
use anyhow::{Context, Result, bail};
use duckdb::{Connection, params};
use jiff::Zoned;
use jiff::civil::Date;
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
    let mut params = db_task.to_named_params();
    let mut stmt = conn.prepare(INSERT_TASK_SQL)?;
    params.remove("end_date");
    params.remove("id");

    stmt.execute(&params)?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    if id <= 0 {
        bail!(format!("Invlid id: {}", id));
    }
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

pub fn update_task(conn: &Connection, task: &Task) -> Result<()> {
    info!("Updating task: {:?}", task);
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

pub fn get_plot_data(
    conn: &Connection,
    start_date: Date,
    end_date: Date,
) -> Result<Vec<(i32, i32, i32, i32)>> {
    const COUNT_TASK_BY_DATE_SQL: &str = include_str!("../assets/task_sql/get_plot_data.sql");
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

pub fn search_task(
    conn: &Connection,
    pattern: &str,
    search_flags: TaskSearchFlags,
    filter_flags: TaskFilterFlags,
    order_flags: TaskOrderFlags,
) -> Result<Vec<Task>> {
    info!("Searching tasks with pattern: '{}'", pattern);
    const SEARCH_SQL: &str = include_str!("../assets/task_sql/search_task.sql");
    let mut stmt = conn.prepare(SEARCH_SQL)?;

    let params = duckdb::named_params! {
        "pattern": pattern,
        "search_flags": search_flags.bits() as i32,
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_tasks = stmt
        .query_map(params, |row| DuckdbTask::try_from(row))?
        .collect::<Result<Vec<DuckdbTask>, duckdb::Error>>()?;

    let tasks = duckdb_tasks
        .into_iter()
        .map(Task::try_from)
        .collect::<Result<Vec<Task>>>()?;

    Ok(tasks)
}

pub fn fetch_one_task(conn: &Connection, id: i32) -> Result<Task> {
    info!("Querying task with id: {}", id);

    const FETCH_ONE_SQL: &str = include_str!("../assets/task_sql/fetch_one_task.sql");
    let mut stmt = conn.prepare(FETCH_ONE_SQL)?;

    let duckdb_task = stmt.query_row(duckdb::named_params! { ":id": id }, |row| {
        DuckdbTask::try_from(row)
    })?;

    let task = Task::try_from(duckdb_task)?;
    Ok(task)
}

pub fn fetch_all_task(
    conn: &Connection,
    filter_flags: TaskFilterFlags,
    order_flags: TaskOrderFlags,
) -> Result<Vec<Task>> {
    info!("Querying tasks");

    const FETCH_ALL_SQL: &str = include_str!("../assets/task_sql/fetch_all_task.sql");

    let mut stmt = conn.prepare(FETCH_ALL_SQL)?;

    let params = duckdb::named_params! {
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_tasks = stmt
        .query_map(params, |row| DuckdbTask::try_from(row))?
        .collect::<Result<Vec<DuckdbTask>, duckdb::Error>>()?;

    let tasks = duckdb_tasks
        .into_iter()
        .map(Task::try_from)
        .collect::<Result<Vec<Task>>>()?;

    Ok(tasks)
}

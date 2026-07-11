use crate::duckdb_weekly_task::DuckdbWeeklyTask;
use anyhow::{Context, Result, bail};
use domain::WeeklyTask;
use domain::{WeeklyTaskFilterFlags, WeeklyTaskOrderFlags, WeeklyTaskSearchFlags};
use duckdb::Connection;
use tracing::info;

pub fn insert_weekly_task(conn: &Connection, weekly_task: &WeeklyTask) -> Result<()> {
    const INSERT_TASK_SQL: &str = include_str!("../assets/weekly_task_sql/insert_weekly_task.sql");
    info!("Inserting weekly_task: {:?}", weekly_task);
    let duckdb_weekly_task: DuckdbWeeklyTask = weekly_task.clone().into();
    let mut params = duckdb_weekly_task.to_named_params();
    params.remove("id");

    let mut stmt = conn.prepare(INSERT_TASK_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    if id <= 0 {
        bail!(format!("Invalid id: {id}"));
    }
    let mut stmt = conn.prepare("SELECT 1 FROM weekly_tasks WHERE id = ?;")?;
    let exists = stmt
        .exists(duckdb::params![id])
        .context("Failed to check if weekly_task ID exists")?;

    Ok(exists)
}

pub fn upsert_weekly_task(conn: &Connection, weekly_task: &WeeklyTask) -> Result<()> {
    let exists = exists_id(conn, weekly_task.id)?;
    if exists {
        info!("ID {} exists. Update weekly_task.", weekly_task.id);
        update_weekly_task(conn, weekly_task)?;
    } else {
        info!("ID {} not exists. Create new weekly_task.", weekly_task.id);
        insert_weekly_task(conn, weekly_task)?;
    }

    Ok(())
}

pub fn get_next_weekly_task_id(conn: &Connection) -> Result<i32> {
    let query =
        "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'weekly_tasks_id_seq';";
    let last_value: Option<i64> = conn
        .query_row(query, [], |row| row.get(0))
        .context("Failed to query next sequence value from DuckDB catalogs")?;
    let next_id = last_value.map_or(1, |val| (val + 1) as i32);
    info!("Next weekly_task id is {}.", next_id);

    Ok(next_id)
}

pub fn update_weekly_task(conn: &Connection, weekly_task: &WeeklyTask) -> Result<()> {
    const UPDATE_TASK_SQL: &str = include_str!("../assets/weekly_task_sql/update_weekly_task.sql");
    info!("Updating weekly_task: {:?}", weekly_task);
    let duckdb_weekly_task: DuckdbWeeklyTask = weekly_task.clone().into();

    let params = duckdb_weekly_task.to_named_params();
    let mut stmt = conn.prepare(UPDATE_TASK_SQL)?;
    stmt.execute(&params)?;

    info!("WeeklyTask {} update success.", weekly_task.id);
    Ok(())
}

pub fn search_weekly_task(
    conn: &Connection,
    pattern: &str,
    search_flags: WeeklyTaskSearchFlags,
    filter_flags: WeeklyTaskFilterFlags,
    order_flags: WeeklyTaskOrderFlags,
) -> Result<Vec<WeeklyTask>> {
    const SEARCH_SQL: &str = include_str!("../assets/weekly_task_sql/search_weekly_task.sql");
    info!("Searching weekly_tasks with pattern: '{}'", pattern);
    let mut stmt = conn.prepare(SEARCH_SQL)?;

    let params = duckdb::named_params! {
        "pattern": pattern,
        "search_flags": search_flags.bits() as i32,
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_weekly_tasks = stmt
        .query_map(params, |row| DuckdbWeeklyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbWeeklyTask>, duckdb::Error>>()?;

    let weekly_tasks = duckdb_weekly_tasks
        .into_iter()
        .map(WeeklyTask::try_from)
        .collect::<Result<Vec<WeeklyTask>>>()?;

    Ok(weekly_tasks)
}

pub fn fetch_one_weekly_task(conn: &Connection, id: i32) -> Result<WeeklyTask> {
    const FETCH_ONE_SQL: &str = include_str!("../assets/weekly_task_sql/fetch_one_weekly_task.sql");
    info!("Querying weekly_task with id: {}", id);
    let mut stmt = conn.prepare(FETCH_ONE_SQL)?;

    let duckdb_weekly_task = stmt.query_row(duckdb::named_params! { ":id": id }, |row| {
        DuckdbWeeklyTask::try_from(row)
    })?;

    let weekly_task = WeeklyTask::try_from(duckdb_weekly_task)?;
    Ok(weekly_task)
}

pub fn fetch_all_weekly_task(
    conn: &Connection,
    filter_flags: WeeklyTaskFilterFlags,
    order_flags: WeeklyTaskOrderFlags,
) -> Result<Vec<WeeklyTask>> {
    const FETCH_ALL_SQL: &str = include_str!("../assets/weekly_task_sql/fetch_all_weekly_task.sql");
    info!("Querying weekly_tasks");
    let mut stmt = conn.prepare(FETCH_ALL_SQL)?;

    let params = duckdb::named_params! {
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_weekly_tasks = stmt
        .query_map(params, |row| DuckdbWeeklyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbWeeklyTask>, duckdb::Error>>()?;

    let weekly_tasks = duckdb_weekly_tasks
        .into_iter()
        .map(WeeklyTask::try_from)
        .collect::<Result<Vec<WeeklyTask>>>()?;

    Ok(weekly_tasks)
}

pub fn delete_weekly_task(conn: &Connection, id: i32) -> Result<()> {
    const DELETE_SQL: &str = include_str!("../assets/weekly_task_sql/delete_weekly_task.sql");
    info!("Delete weekly task: {}", id);
    let mut stmt = conn.prepare(DELETE_SQL)?;
    stmt.execute(duckdb::named_params! { ":id": id })?;

    Ok(())
}

use crate::duckdb_daily_task::DuckdbDailyTask;
use anyhow::{Context, Result, bail};
use domain::DailyTask;
use domain::{DailyTaskFilterFlags, DailyTaskOrderFlags, DailyTaskSearchFlags};
use duckdb::Connection;
use tracing::info;

pub fn insert_daily_task(conn: &Connection, daily_task: &DailyTask) -> Result<()> {
    info!("Inserting daily_task: {:?}", daily_task);
    const INSERT_TASK_SQL: &str = include_str!("../assets/daily_task_sql/insert_daily_task.sql");
    let duckdb_daily_task: DuckdbDailyTask = daily_task.clone().into();
    let params = duckdb_daily_task.to_named_params();
    let mut stmt = conn.prepare(INSERT_TASK_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    if id <= 0 {
        bail!(format!("Invlid id: {}", id));
    }
    let mut stmt = conn.prepare("SELECT 1 FROM daily_tasks WHERE id = ?;")?;
    let exists = stmt
        .exists(duckdb::params![id])
        .context("Failed to check if daily_task ID exists")?;

    Ok(exists)
}

pub fn upsert_daily_task(conn: &Connection, daily_task: &DailyTask) -> Result<()> {
    let exists = exists_id(conn, daily_task.id)?;
    if exists {
        info!("ID {} exists. Update daily_task.", daily_task.id);
        update_daily_task(conn, daily_task)?;
    } else {
        info!("ID {} not exists. Create new daily_task.", daily_task.id);
        insert_daily_task(conn, daily_task)?;
    }

    Ok(())
}

pub fn get_next_daily_task_id(conn: &Connection) -> Result<i32> {
    let query =
        "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'daily_tasks_id_seq';";
    let last_value: Option<i64> = conn
        .query_row(query, [], |row| row.get(0))
        .context("Failed to query next sequence value from DuckDB catalogs")?;
    let next_id = last_value.map(|val| (val + 1) as i32).unwrap_or(1);
    info!("Next daily_task id is {}.", next_id);

    Ok(next_id)
}

pub fn update_daily_task(conn: &Connection, daily_task: &DailyTask) -> Result<()> {
    info!("Updating daily_task: {:?}", daily_task);
    const UPDATE_TASK_SQL: &str = include_str!("../assets/daily_task_sql/update_daily_task.sql");
    let duckdb_daily_task: DuckdbDailyTask = daily_task.clone().into();

    let params = duckdb_daily_task.to_named_params();
    let mut stmt = conn.prepare(UPDATE_TASK_SQL)?;
    stmt.execute(&params)?;

    info!("DailyTask {} update success.", daily_task.id);
    Ok(())
}

pub fn search_daily_task(
    conn: &Connection,
    pattern: &str,
    search_flags: DailyTaskSearchFlags,
    filter_flags: DailyTaskFilterFlags,
    order_flags: DailyTaskOrderFlags,
) -> Result<Vec<DailyTask>> {
    info!("Searching daily_tasks with pattern: '{}'", pattern);
    const SEARCH_SQL: &str = include_str!("../assets/daily_task_sql/search_daily_task.sql");
    let mut stmt = conn.prepare(SEARCH_SQL)?;

    let params = duckdb::named_params! {
        "pattern": pattern,
        "search_flags": search_flags.bits() as i32,
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_daily_tasks = stmt
        .query_map(params, |row| DuckdbDailyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbDailyTask>, duckdb::Error>>()?;

    let daily_tasks = duckdb_daily_tasks
        .into_iter()
        .map(DailyTask::try_from)
        .collect::<Result<Vec<DailyTask>>>()?;

    Ok(daily_tasks)
}

pub fn fetch_one_daily_task(conn: &Connection, id: i32) -> Result<DailyTask> {
    info!("Querying daily_task with id: {}", id);

    const FETCH_ONE_SQL: &str = include_str!("../assets/daily_task_sql/fetch_one_daily_task.sql");
    let mut stmt = conn.prepare(FETCH_ONE_SQL)?;

    let duckdb_daily_task = stmt.query_row(duckdb::named_params! { ":id": id }, |row| {
        DuckdbDailyTask::try_from(row)
    })?;

    let daily_task = DailyTask::try_from(duckdb_daily_task)?;
    Ok(daily_task)
}

pub fn fetch_all_daily_task(
    conn: &Connection,
    filter_flags: DailyTaskFilterFlags,
    order_flags: DailyTaskOrderFlags,
) -> Result<Vec<DailyTask>> {
    info!("Querying daily_tasks");

    const FETCH_ALL_SQL: &str = include_str!("../assets/daily_task_sql/fetch_all_daily_task.sql");

    let mut stmt = conn.prepare(FETCH_ALL_SQL)?;

    let params = duckdb::named_params! {
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_daily_tasks = stmt
        .query_map(params, |row| DuckdbDailyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbDailyTask>, duckdb::Error>>()?;

    let daily_tasks = duckdb_daily_tasks
        .into_iter()
        .map(DailyTask::try_from)
        .collect::<Result<Vec<DailyTask>>>()?;

    Ok(daily_tasks)
}

pub fn delete_daily_task(conn: &Connection, id: i32) -> Result<()> {
    info!("Delete daily task: {}", id);
    const DELETE_SQL: &str = include_str!("../assets/daily_task_sql/delete_daily_task.sql");
    let mut stmt = conn.prepare(DELETE_SQL)?;
    stmt.execute(duckdb::named_params! { ":id": id })?;

    Ok(())
}

use crate::duckdb_monthly_task::DuckdbMonthlyTask;
use anyhow::{Context, Result};
use domain::MonthlyTask;
use domain::{MonthlyTaskFilterFlags, MonthlyTaskOrderFlags, MonthlyTaskSearchFlags};
use duckdb::Connection;
use tracing::info;
use uuid::Uuid;

pub fn insert_monthly_task(conn: &Connection, monthly_task: &MonthlyTask) -> Result<()> {
    const INSERT_TASK_SQL: &str =
        include_str!("../assets/monthly_task_sql/insert_monthly_task.sql");
    info!("Inserting monthly_task: {:?}", monthly_task);
    let duckdb_monthly_task: DuckdbMonthlyTask = monthly_task.clone().into();
    let mut params = duckdb_monthly_task.to_named_params();
    params.remove("uuid");
    let mut stmt = conn.prepare(INSERT_TASK_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

fn exists_uuid(conn: &Connection, uuid: Uuid) -> Result<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM monthly_tasks WHERE uuid = ?;",
            duckdb::params![uuid],
            |row| row.get(0),
        )
        .context("Failed to check if monthly_task ID exists")?;

    Ok(count > 0)
}

pub fn upsert_monthly_task(conn: &Connection, monthly_task: &MonthlyTask) -> Result<()> {
    let exists = exists_uuid(conn, monthly_task.uuid)?;
    if exists {
        info!("ID {} exists. Update monthly_task.", monthly_task.uuid);
        update_monthly_task(conn, monthly_task)?;
    } else {
        info!(
            "ID {} not exists. Create new monthly_task.",
            monthly_task.uuid
        );
        insert_monthly_task(conn, monthly_task)?;
    }

    Ok(())
}

pub fn update_monthly_task(conn: &Connection, monthly_task: &MonthlyTask) -> Result<()> {
    const UPDATE_TASK_SQL: &str =
        include_str!("../assets/monthly_task_sql/update_monthly_task.sql");
    info!("Updating monthly_task: {:?}", monthly_task);
    let duckdb_monthly_task: DuckdbMonthlyTask = monthly_task.clone().into();

    let params = duckdb_monthly_task.to_named_params();
    let mut stmt = conn.prepare(UPDATE_TASK_SQL)?;
    stmt.execute(&params)?;

    info!("MonthlyTask {} update success.", monthly_task.uuid);
    Ok(())
}

pub fn search_monthly_task(
    conn: &Connection,
    pattern: &str,
    search_flags: MonthlyTaskSearchFlags,
    filter_flags: MonthlyTaskFilterFlags,
    order_flags: MonthlyTaskOrderFlags,
) -> Result<Vec<MonthlyTask>> {
    const SEARCH_SQL: &str = include_str!("../assets/monthly_task_sql/search_monthly_task.sql");
    info!("Searching monthly_tasks with pattern: '{}'", pattern);
    let mut stmt = conn.prepare(SEARCH_SQL)?;

    let params = duckdb::named_params! {
        "pattern": pattern,
        "search_flags": search_flags.bits() as i32,
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_monthly_tasks = stmt
        .query_map(params, |row| DuckdbMonthlyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbMonthlyTask>, duckdb::Error>>()?;

    let monthly_tasks = duckdb_monthly_tasks
        .into_iter()
        .map(MonthlyTask::try_from)
        .collect::<Result<Vec<MonthlyTask>>>()?;

    Ok(monthly_tasks)
}

pub fn fetch_one_monthly_task(conn: &Connection, uuid: Uuid) -> Result<MonthlyTask> {
    const FETCH_ONE_SQL: &str =
        include_str!("../assets/monthly_task_sql/fetch_one_monthly_task.sql");
    info!("Querying monthly_task with id: {}", uuid);
    let mut stmt = conn.prepare(FETCH_ONE_SQL)?;

    let duckdb_monthly_task = stmt.query_row(duckdb::named_params! { ":uuid": uuid }, |row| {
        DuckdbMonthlyTask::try_from(row)
    })?;

    let monthly_task = MonthlyTask::try_from(duckdb_monthly_task)?;
    Ok(monthly_task)
}

pub fn fetch_all_monthly_task(
    conn: &Connection,
    filter_flags: MonthlyTaskFilterFlags,
    order_flags: MonthlyTaskOrderFlags,
) -> Result<Vec<MonthlyTask>> {
    const FETCH_ALL_SQL: &str =
        include_str!("../assets/monthly_task_sql/fetch_all_monthly_task.sql");
    info!("Querying monthly_tasks");
    let mut stmt = conn.prepare(FETCH_ALL_SQL)?;

    let params = duckdb::named_params! {
        "filter_flags": filter_flags.bits() as i32,
        "order_flags": order_flags.bits() as i32,
    };

    let duckdb_monthly_tasks = stmt
        .query_map(params, |row| DuckdbMonthlyTask::try_from(row))?
        .collect::<Result<Vec<DuckdbMonthlyTask>, duckdb::Error>>()?;

    let monthly_tasks = duckdb_monthly_tasks
        .into_iter()
        .map(MonthlyTask::try_from)
        .collect::<Result<Vec<MonthlyTask>>>()?;

    Ok(monthly_tasks)
}

pub fn delete_monthly_task(conn: &Connection, uuid: Uuid) -> Result<()> {
    const DELETE_SQL: &str = include_str!("../assets/monthly_task_sql/delete_monthly_task.sql");
    info!("Delete monthly task: {}", uuid);
    let mut stmt = conn.prepare(DELETE_SQL)?;
    stmt.execute(duckdb::named_params! { ":uuid": uuid })?;

    Ok(())
}

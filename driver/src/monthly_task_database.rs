use crate::duckdb_monthly_task::DuckdbMonthlyTask;
use anyhow::{Context, Result, bail};
use domain::MonthlyTask;
use domain::{MonthlyTaskFilterFlags, MonthlyTaskOrderFlags, MonthlyTaskSearchFlags};
use duckdb::Connection;
use tracing::info;

pub fn insert_monthly_task(conn: &Connection, monthly_task: &MonthlyTask) -> Result<()> {
    const INSERT_TASK_SQL: &str =
        include_str!("../assets/monthly_task_sql/insert_monthly_task.sql");
    info!("Inserting monthly_task: {:?}", monthly_task);
    let duckdb_monthly_task: DuckdbMonthlyTask = monthly_task.clone().into();
    let mut params = duckdb_monthly_task.to_named_params();
    params.remove("id");
    let mut stmt = conn.prepare_cached(INSERT_TASK_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

fn exists_id(conn: &Connection, id: i32) -> Result<bool> {
    if id <= 0 {
        bail!(format!("Invalid id: {id}"));
    }
    let mut stmt = conn.prepare_cached("SELECT 1 FROM monthly_tasks WHERE id = ?;")?;
    let exists = stmt
        .exists(duckdb::params![id])
        .context("Failed to check if monthly_task ID exists")?;

    Ok(exists)
}

pub fn upsert_monthly_task(conn: &Connection, monthly_task: &MonthlyTask) -> Result<()> {
    let exists = exists_id(conn, monthly_task.id)?;
    if exists {
        info!("ID {} exists. Update monthly_task.", monthly_task.id);
        update_monthly_task(conn, monthly_task)?;
    } else {
        info!(
            "ID {} not exists. Create new monthly_task.",
            monthly_task.id
        );
        insert_monthly_task(conn, monthly_task)?;
    }

    Ok(())
}

pub fn get_next_monthly_task_id(conn: &Connection) -> Result<i32> {
    const GET_NEXT_MONTHLY_TASK_ID_SQL: &str =
        "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'monthly_tasks_id_seq';";
    let mut stmt = conn.prepare_cached(GET_NEXT_MONTHLY_TASK_ID_SQL)?;
    let last_value: Option<i64> = stmt
        .query_row([], |row| row.get(0))
        .context("Failed to query next sequence value from DuckDB catalogs")?;
    let next_id = last_value.map_or(1, |val| (val + 1) as i32);
    info!("Next monthly_task id is {}.", next_id);

    Ok(next_id)
}

pub fn update_monthly_task(conn: &Connection, monthly_task: &MonthlyTask) -> Result<()> {
    const UPDATE_TASK_SQL: &str =
        include_str!("../assets/monthly_task_sql/update_monthly_task.sql");
    info!("Updating monthly_task: {:?}", monthly_task);
    let duckdb_monthly_task: DuckdbMonthlyTask = monthly_task.clone().into();

    let params = duckdb_monthly_task.to_named_params();
    let mut stmt = conn.prepare_cached(UPDATE_TASK_SQL)?;
    stmt.execute(&params)?;

    info!("MonthlyTask {} update success.", monthly_task.id);
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
    let mut stmt = conn.prepare_cached(SEARCH_SQL)?;

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

pub fn fetch_one_monthly_task(conn: &Connection, id: i32) -> Result<MonthlyTask> {
    const FETCH_ONE_SQL: &str =
        include_str!("../assets/monthly_task_sql/fetch_one_monthly_task.sql");
    info!("Querying monthly_task with id: {}", id);
    let mut stmt = conn.prepare_cached(FETCH_ONE_SQL)?;

    let duckdb_monthly_task = stmt.query_row(duckdb::named_params! { ":id": id }, |row| {
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
    let mut stmt = conn.prepare_cached(FETCH_ALL_SQL)?;

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

pub fn delete_monthly_task(conn: &Connection, id: i32) -> Result<()> {
    const DELETE_SQL: &str = include_str!("../assets/monthly_task_sql/delete_monthly_task.sql");
    info!("Delete monthly task: {}", id);
    let mut stmt = conn.prepare_cached(DELETE_SQL)?;
    stmt.execute(duckdb::named_params! { ":id": id })?;

    Ok(())
}

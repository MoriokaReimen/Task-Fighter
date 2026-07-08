use crate::duckdb_work_time::DuckdbWorkTime;
use anyhow::Result;
use domain::WorkTime;
use duckdb::Connection;
use jiff::civil::Date;
use tracing::info;

pub fn next_work_time_id(conn: &Connection) -> Result<i32> {
    let query =
        "SELECT last_value FROM duckdb_sequences() WHERE sequence_name = 'seq_work_time_id';";
    let last_value: i64 = conn.query_row(query, [], |row| row.get(0)).unwrap_or(0);
    let next_id = (last_value + 1) as i32;
    info!("Next work time id is {}.", next_id);

    Ok(next_id)
}

fn find_work_time(conn: &Connection, task_id: i32, date: &Date) -> Result<Option<WorkTime>> {
    info!("Find work_time for task_id: {}, date: {}", task_id, date);
    const FIND_WORK_TIME_SQL: &str = include_str!("../assets/work_time_sql/find_work_time.sql");
    let mut stmt = conn.prepare(FIND_WORK_TIME_SQL)?;
    let params = duckdb::named_params! {
        "task_id": task_id,
        "date": date.to_string(),
    };
    let mut rows = stmt.query(params)?;
    if let Some(row) = rows.next()? {
        let duckdb_work_time = DuckdbWorkTime::try_from(row)?;
        let work_time = WorkTime::try_from(duckdb_work_time)?;
        Ok(Some(work_time))
    } else {
        Ok(None)
    }
}

fn list_work_time_for_task(conn: &Connection, task_id: i32) -> Result<Vec<WorkTime>> {
    info!("List work_time for task_id: {}", task_id);

    const LIST_WORK_TIME_FOR_TASK_SQL: &str =
        include_str!("../assets/work_time_sql/list_work_time_by_task.sql");
    let mut stmt = conn.prepare(LIST_WORK_TIME_FOR_TASK_SQL)?;
    let params = duckdb::named_params! {
        "task_id": task_id,
    };
    let work_time_iter = stmt.query_map(params, |row| DuckdbWorkTime::try_from(row))?;

    let mut result = Vec::new();
    for item in work_time_iter {
        let duckdb_item = item?;
        let work_time = WorkTime::try_from(duckdb_item)?;
        result.push(work_time);
    }

    Ok(result)
}

pub fn insert_work_time(conn: &Connection, work_time: &WorkTime) -> Result<()> {
    info!("Inserting work time: {:?}", work_time);
    const INSERT_WORK_TIME_SQL: &str = include_str!("../assets/work_time_sql/insert_work_time.sql");
    let duckdb_work_time: DuckdbWorkTime = work_time.clone().into();
    let params = duckdb_work_time.to_named_params();
    let mut stmt = conn.prepare(INSERT_WORK_TIME_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

pub fn update_work_time(conn: &Connection, work_time: &WorkTime) -> Result<()> {
    info!("Update work time: {:?}", work_time);
    const UPDATE_WORK_TIME_SQL: &str = include_str!("../assets/work_time_sql/update_work_time.sql");
    let duckdb_work_time: DuckdbWorkTime = work_time.clone().into();
    let params = duckdb_work_time.to_named_params();
    let mut stmt = conn.prepare(UPDATE_WORK_TIME_SQL)?;
    stmt.execute(&params)?;

    Ok(())
}

pub fn upsert_work_time(conn: &Connection, work_time: &WorkTime) -> Result<()> {
    let exists = exists_work_time_id(conn, work_time.id)?;
    if exists {
        update_work_time(conn, work_time)
    } else {
        insert_work_time(conn, work_time)
    }
}

fn exists_work_time_id(conn: &Connection, id: i32) -> Result<bool> {
    info!("Check id {} exists in work_time table", id);
    const EXISTS_WORK_TIME_ID_SQL: &str =
        include_str!("../assets/work_time_sql/exists_work_time_id.sql");

    let params = duckdb::named_params! {
        "id": id,
    };

    let exists: bool = conn.query_row(EXISTS_WORK_TIME_ID_SQL, params, |row| row.get(0))?;

    Ok(exists)
}

pub fn get_total_work_time_by_task(conn: &Connection, task_id: i32) -> Result<f32> {
    info!("Get total work time for: {}", task_id);
    const GET_TOTAL_WORK_TIME_BY_TASK_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_by_task.sql");
    let params = duckdb::named_params! {"task_id": task_id};
    let total_time: f32 = conn.query_row(
        GET_TOTAL_WORK_TIME_BY_TASK_SQL,
        params,
        |row| row.get(0), // 最初のカラム（SUMの結果）を取得
    )?;

    Ok(total_time)
}

pub fn get_total_work_time_by_date(conn: &Connection, date: &Date) -> Result<f32> {
    info!("Get total work time for: {}", date);
    const GET_TOTAL_WORK_TIME_BY_DATE_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_by_date.sql");
    let date_str = date.to_string();
    let params = duckdb::named_params! {"date": date_str};
    let total_time: f32 = conn.query_row(
        GET_TOTAL_WORK_TIME_BY_DATE_SQL,
        params,
        |row| row.get(0), // 最初のカラム（SUMの結果）を取得
    )?;

    Ok(total_time)
}

pub fn get_total_work_time_history(
    conn: &Connection,
    start_date: &Date,
    end_date: &Date,
) -> Result<Vec<(Date, f32)>> {
    info!(
        "Get total work time history from {} to {}",
        start_date, end_date
    );
    const GET_TOTAL_WORK_TIME_HISTORY_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_history.sql");

    let start_date_str = start_date.to_string();
    let end_date_str = end_date.to_string();

    let params = duckdb::named_params! {
        "start_date": start_date_str,
        "end_date": end_date_str
    };

    let mut stmt = conn.prepare(GET_TOTAL_WORK_TIME_HISTORY_SQL)?;
    let mut rows = stmt.query(params)?;

    let mut history = Vec::new();
    while let Some(row) = rows.next()? {
        let date_str: String = row.get(0)?;
        let date = date_str.parse::<Date>()?;
        let time_spent: f32 = row.get(1)?;
        history.push((date, time_spent));
    }

    Ok(history)
}

pub fn get_total_work_time_ratio(
    conn: &Connection,
    start_date: &Date,
    end_date: &Date,
) -> Result<Vec<(i32, f32)>> {
    info!(
        "Get total work time ratio from {} to {}",
        start_date, end_date
    );
    const GET_TOTAL_WORK_TIME_RATIO_SQL: &str =
        include_str!("../assets/work_time_sql/get_total_work_time_ratio.sql");

    let start_date_str = start_date.to_string();
    let end_date_str = end_date.to_string();

    let params = duckdb::named_params! {
        "start_date": start_date_str,
        "end_date": end_date_str
    };

    let mut stmt = conn.prepare(GET_TOTAL_WORK_TIME_RATIO_SQL)?;
    let mut rows = stmt.query(params)?;

    let mut ratios = Vec::new();
    while let Some(row) = rows.next()? {
        let task_id: i32 = row.get(0)?;
        let ratio: f32 = row.get(1)?;

        ratios.push((task_id, ratio));
    }

    Ok(ratios)
}

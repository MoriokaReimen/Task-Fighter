use crate::duckdb_work_time::DuckdbWorkTime;
use anyhow::Result;
use domain::WorkTime;
use duckdb::Connection;
use jiff::civil::Date;
use tracing::info;

pub fn insert_work_time(conn: &Connection, work_time: &WorkTime) -> Result<()> {
    info!("Inserting work time: {:?}", work_time);
    const INSERT_WORK_TIME_SQL: &str = include_str!("../assets/work_time_sql/insert_work_time.sql");
    let duckdb_work_time: DuckdbWorkTime = work_time.clone().into();
    let params = duckdb_work_time.to_named_params();
    let mut stmt = conn.prepare(INSERT_WORK_TIME_SQL)?;
    stmt.execute(&params)?;

    Ok(())
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

    // 複数行を取得するため、prepare から rows をループ処理する
    let mut stmt = conn.prepare(GET_TOTAL_WORK_TIME_HISTORY_SQL)?;
    let mut rows = stmt.query(params)?;

    let mut history = Vec::new();
    while let Some(row) = rows.next()? {
        // 1. カラム0から日付の文字列（"YYYY-MM-DD"）を取得してJiffのDate型に変換
        let date_str: String = row.get(0)?;
        let date = date_str.parse::<Date>()?; // 💡 .parse() メソッドで安全にDate型に

        // 2. カラム1から合計作業時間（f32）を取得
        let time_spent: f32 = row.get(1)?;

        // タプルのペアにして配列に追加
        history.push((date, time_spent));
    }

    Ok(history)
}
